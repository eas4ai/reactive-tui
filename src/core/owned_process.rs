//! Bounded command I/O with cancellation and explicit child ownership.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

const POLL: Duration = Duration::from_millis(5);
pub(crate) const TERMINAL_HELPER_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Clone, Copy)]
pub(crate) struct Options {
    pub purpose: &'static str,
    pub timeout: Duration,
    pub max_input: u64,
    pub max_output: u64,
    pub capture_output: bool,
    pub allow_background_after_success: bool,
}

fn limit_error(purpose: &str, stream: &str, limit: u64) -> String {
    let size = if limit > 0 && limit.is_multiple_of(1024 * 1024) {
        format!("{} MiB", limit / (1024 * 1024))
    } else {
        format!("{limit} byte")
    };
    format!("{purpose} {stream} exceeds the {size} transfer limit")
}

struct Deadline<F> {
    end: Instant,
    options: Options,
    cancelled: F,
}

impl<F: Fn() -> bool> Deadline<F> {
    fn check(&self) -> Result<(), String> {
        if (self.cancelled)() {
            Err(format!(
                "{} operation cancelled: owner has closed",
                self.options.purpose
            ))
        } else if Instant::now() >= self.end {
            Err(format!(
                "{} operation timed out after {} seconds",
                self.options.purpose,
                self.options.timeout.as_secs_f64()
            ))
        } else {
            Ok(())
        }
    }
}

struct OwnedChild {
    child: Child,
    released: bool,
}

impl OwnedChild {
    fn stop(&mut self) -> std::io::Result<()> {
        #[cfg(unix)]
        let mut group_error = None;
        #[cfg(unix)]
        {
            // The child starts in its own group. Stop descendants that still
            // belong to that group before reaping the direct child.
            let status = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
            if status < 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    group_error = Some(error);
                }
            }
        }
        if self.child.try_wait()?.is_none() {
            if let Err(error) = self.child.kill() {
                // A child may finish between the status query and kill. A failed
                // signal alone does not establish liveness. Accept the error only
                // after observing and reaping its exit, with a bounded race wait.
                let deadline = Instant::now() + Duration::from_millis(100);
                loop {
                    if self.child.try_wait()?.is_some() {
                        break;
                    }
                    if Instant::now() >= deadline {
                        return Err(std::io::Error::new(
                            error.kind(),
                            format!("Stop direct child {}: {error}", self.child.id()),
                        ));
                    }
                    std::thread::sleep(POLL);
                }
            }
        }
        self.child.wait()?;
        #[cfg(unix)]
        if let Some(error) = group_error {
            // A group containing only an exiting child may reject the signal.
            // Do not hide a real descendant-cleanup failure: require that the
            // group is absent after reaping the direct child.
            if unsafe { libc::kill(-(self.child.id() as i32), 0) } == 0
                || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
            {
                return Err(std::io::Error::new(
                    error.kind(),
                    format!("Stop child process group {}: {error}", self.child.id()),
                ));
            }
        }
        self.released = true;
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.released {
            if let Err(error) = self.stop() {
                log::error!("Could not stop an owned process: {error}");
            }
        }
    }
}

struct Streams {
    input: File,
    output: File,
}

impl Streams {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            input: tempfile::tempfile()?,
            output: tempfile::tempfile()?,
        })
    }

    fn check_size(&self, options: Options) -> Result<(), String> {
        if self
            .output
            .metadata()
            .map_err(|error| format!("{} output metadata: {error}", options.purpose))?
            .len()
            > options.max_output
        {
            return Err(limit_error(options.purpose, "output", options.max_output));
        }
        Ok(())
    }
}

pub(crate) fn run(
    mut command: Command,
    input: Option<&[u8]>,
    options: Options,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    let deadline = Deadline {
        end: Instant::now()
            .checked_add(options.timeout)
            .ok_or_else(|| format!("{} timeout exceeds the clock range", options.purpose))?,
        options,
        cancelled,
    };
    deadline.check()?;
    if input.is_some_and(|input| input.len() as u64 > options.max_input) {
        return Err(limit_error(options.purpose, "input", options.max_input));
    }
    let mut streams =
        Streams::new().map_err(|error| format!("{} temporary I/O: {error}", options.purpose))?;
    for chunk in input.unwrap_or_default().chunks(64 * 1024) {
        deadline.check()?;
        streams
            .input
            .write_all(chunk)
            .map_err(|error| format!("{} input: {error}", options.purpose))?;
    }
    streams
        .input
        .rewind()
        .map_err(|error| format!("{} input seek: {error}", options.purpose))?;
    command
        .stdin(Stdio::from(
            streams
                .input
                .try_clone()
                .map_err(|error| error.to_string())?,
        ))
        .stdout(if !options.capture_output {
            Stdio::null()
        } else {
            Stdio::from(
                streams
                    .output
                    .try_clone()
                    .map_err(|error| error.to_string())?,
            )
        })
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    deadline.check()?;
    let program = command.get_program().to_string_lossy().into_owned();
    let child = command
        .spawn()
        .map_err(|error| format!("Could not start {program}: {error}"))?;
    let mut owned = OwnedChild {
        child,
        released: false,
    };
    let result = wait(&mut owned.child, &streams, &deadline).and_then(|status| {
        if !status.success() {
            return Err(format!("{program} exited with {status}"));
        }
        if !options.capture_output {
            return Ok(Vec::new());
        }
        read_output(&mut streams.output, &deadline)
    });
    match result {
        Ok(output) => {
            // The direct child was reaped. Successful desktop tools can leave
            // a daemon that serves the clipboard after their parent exits.
            if options.allow_background_after_success {
                owned.released = true;
            } else {
                // This command has no permitted background owner.
                owned
                    .stop()
                    .map_err(|error| format!("{} cleanup: {error}", options.purpose))?;
            }
            Ok(output)
        }
        Err(error) => {
            owned
                .stop()
                .map_err(|cleanup| format!("{error}; child cleanup failed: {cleanup}"))?;
            Err(error)
        }
    }
}

fn wait<F: Fn() -> bool>(
    child: &mut Child,
    streams: &Streams,
    deadline: &Deadline<F>,
) -> Result<ExitStatus, String> {
    loop {
        deadline.check()?;
        streams.check_size(deadline.options)?;
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("{} child status: {error}", deadline.options.purpose))?
        {
            streams.check_size(deadline.options)?;
            return Ok(status);
        }
        std::thread::sleep(POLL.min(deadline.end.saturating_duration_since(Instant::now())));
    }
}

fn read_output<F: Fn() -> bool>(
    file: &mut File,
    deadline: &Deadline<F>,
) -> Result<Vec<u8>, String> {
    file.seek(SeekFrom::Start(0))
        .map_err(|error| format!("{} output seek: {error}", deadline.options.purpose))?;
    let mut output = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        deadline.check()?;
        let count = file
            .read(&mut chunk)
            .map_err(|error| format!("{} output read: {error}", deadline.options.purpose))?;
        if count == 0 {
            return Ok(output);
        }
        if (output.len() + count) as u64 > deadline.options.max_output {
            return Err(limit_error(
                deadline.options.purpose,
                "output",
                deadline.options.max_output,
            ));
        }
        output.extend_from_slice(&chunk[..count]);
    }
}
