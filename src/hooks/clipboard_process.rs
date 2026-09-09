//! Synchronous clipboard commands with bounded I/O and explicit child ownership.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

#[cfg(not(windows))]
const TIMEOUT: Duration = Duration::from_secs(2);
#[cfg(windows)]
const TIMEOUT: Duration = Duration::from_secs(5);
const MAX_BYTES: u64 = 64 * 1024 * 1024;
const POLL: Duration = Duration::from_millis(5);

struct Deadline<F> {
    end: Instant,
    cancelled: F,
}

impl<F: Fn() -> bool> Deadline<F> {
    fn check(&self) -> Result<(), String> {
        if (self.cancelled)() {
            Err("clipboard operation cancelled: hook owner has closed".into())
        } else if Instant::now() >= self.end {
            Err(format!(
                "clipboard operation timed out after {} seconds",
                TIMEOUT.as_secs()
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
        {
            // The child starts in its own group. Stop descendants that still
            // belong to that group before reaping the direct child.
            let status = unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) };
            if status < 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error);
                }
            }
        }
        match self.child.kill() {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::InvalidInput => {}
            Err(error) => return Err(error),
        }
        self.child.wait()?;
        self.released = true;
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.released {
            if let Err(error) = self.stop() {
                log::error!("Could not stop an owned clipboard process: {error}");
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

    fn check_size(&self) -> Result<(), String> {
        if self
            .output
            .metadata()
            .map_err(|error| format!("clipboard output metadata: {error}"))?
            .len()
            > MAX_BYTES
        {
            return Err("clipboard output exceeds the 64 MiB transfer limit".into());
        }
        Ok(())
    }
}

pub(super) fn run(
    mut command: Command,
    text: Option<&str>,
    cancelled: impl Fn() -> bool,
) -> Result<Vec<u8>, String> {
    let deadline = Deadline {
        end: Instant::now() + TIMEOUT,
        cancelled,
    };
    deadline.check()?;
    if text.is_some_and(|text| text.len() as u64 > MAX_BYTES) {
        return Err("clipboard input exceeds the 64 MiB transfer limit".into());
    }
    let mut streams =
        Streams::new().map_err(|error| format!("clipboard temporary I/O: {error}"))?;
    for chunk in text.unwrap_or_default().as_bytes().chunks(64 * 1024) {
        deadline.check()?;
        streams
            .input
            .write_all(chunk)
            .map_err(|error| format!("clipboard input: {error}"))?;
    }
    streams
        .input
        .rewind()
        .map_err(|error| format!("clipboard input seek: {error}"))?;
    command
        .stdin(Stdio::from(
            streams
                .input
                .try_clone()
                .map_err(|error| error.to_string())?,
        ))
        .stdout(if text.is_some() {
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
        if text.is_some() {
            return Ok(Vec::new());
        }
        read_output(&mut streams.output, &deadline)
    });
    match result {
        Ok(output) => {
            // The direct child was reaped. Successful desktop tools can leave
            // a daemon that serves the clipboard after their parent exits.
            if text.is_some() {
                owned.released = true;
            } else {
                // Paste has no clipboard-serving background owner.
                owned
                    .stop()
                    .map_err(|error| format!("clipboard paste cleanup: {error}"))?;
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
        streams.check_size()?;
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("clipboard child status: {error}"))?
        {
            streams.check_size()?;
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
        .map_err(|error| format!("clipboard output seek: {error}"))?;
    let mut output = Vec::new();
    let mut chunk = [0; 64 * 1024];
    loop {
        deadline.check()?;
        let count = file
            .read(&mut chunk)
            .map_err(|error| format!("clipboard output read: {error}"))?;
        if count == 0 {
            return Ok(output);
        }
        if (output.len() + count) as u64 > MAX_BYTES {
            return Err("clipboard output exceeds the 64 MiB transfer limit".into());
        }
        output.extend_from_slice(&chunk[..count]);
    }
}

#[cfg(all(test, any(unix, windows)))]
mod tests {
    use super::*;

    #[test]
    #[ignore = "child fixture invoked only by the process lifecycle tests"]
    fn sleeping_child_fixture() {
        let Some(path) = std::env::var_os("RTUI_PID_PATH") else {
            return;
        };
        std::fs::write(path, std::process::id().to_string()).unwrap();
        std::thread::sleep(Duration::from_secs(30));
    }

    fn exercise_stop(cancel: bool) {
        let directory = tempfile::tempdir().unwrap();
        let pid_path = directory.path().join("pid");
        // A native child isolates process ownership from shell startup latency.
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--ignored",
            "--exact",
            "hooks::clipboard_process::tests::sleeping_child_fixture",
        ]);
        #[cfg(windows)]
        let process = std::cell::RefCell::new(None::<std::os::windows::io::OwnedHandle>);
        command.env("RTUI_PID_PATH", &pid_path);
        let start = Instant::now();
        let error = run(command, None, || {
            let ready = std::fs::read_to_string(&pid_path)
                .ok()
                .and_then(|pid| pid.trim().parse::<u32>().ok());
            #[cfg(windows)]
            if let Some(pid) = ready {
                use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
                use windows_sys::Win32::{
                    Foundation::WAIT_TIMEOUT,
                    System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_SYNCHRONIZE},
                };
                if process.borrow().is_none() {
                    // Retain the exact live process identity across cancellation;
                    // querying a PID after exit could observe a reused identifier.
                    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
                    assert!(
                        !handle.is_null(),
                        "open child process: {}",
                        std::io::Error::last_os_error()
                    );
                    let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
                    assert_eq!(
                        unsafe { WaitForSingleObject(handle.as_raw_handle(), 0) },
                        WAIT_TIMEOUT,
                        "fixture must be live before stop"
                    );
                    *process.borrow_mut() = Some(handle);
                }
            }
            cancel && ready.is_some()
        })
        .unwrap_err();
        assert!(
            error.contains(if cancel { "cancelled" } else { "timed out" }),
            "{error}"
        );
        #[cfg(unix)]
        assert!(start.elapsed() < Duration::from_secs(3));
        #[cfg(windows)]
        assert!(start.elapsed() < Duration::from_secs(6));
        #[cfg(unix)]
        {
            let pid: i32 = std::fs::read_to_string(&pid_path)
                .unwrap()
                .trim()
                .parse()
                .unwrap();
            assert_eq!(
                unsafe { libc::kill(pid, 0) },
                -1,
                "child remains live or unreaped"
            );
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::ESRCH)
            );
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows_sys::Win32::{
                Foundation::WAIT_OBJECT_0, System::Threading::WaitForSingleObject,
            };
            let process = process.into_inner().expect("observed the live child");
            assert_eq!(
                unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
                WAIT_OBJECT_0,
                "owned Windows process must have exited"
            );
        }
    }

    #[test]
    fn deadline_kills_and_reaps_child() {
        exercise_stop(false);
    }

    #[test]
    fn cancellation_kills_and_reaps_child() {
        exercise_stop(true);
    }
}
