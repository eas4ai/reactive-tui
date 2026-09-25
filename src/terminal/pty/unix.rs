use crate::terminal::{owned_pty::PtyChild, TerminalConfig, TerminalError, TerminalResult};
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    os::unix::process::ExitStatusExt,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use super::MAX_INPUT;
type Reply = mpsc::Sender<TerminalResult<()>>;
enum Request {
    Input(Vec<u8>, Reply),
    Resize(u16, u16, Reply),
}
#[derive(Debug, Default)]
struct Outcome {
    exit: Option<i32>,
    error: Option<String>,
}

/// A real controlling PTY with bounded queues and joined child/IO ownership.
#[derive(Debug)]
pub struct PseudoTerminal {
    size: (u16, u16),
    child_id: Option<u32>,
    commands: Option<SyncSender<Request>>,
    output: Option<Receiver<Vec<u8>>>,
    stop: Arc<AtomicBool>,
    outcome: Arc<Mutex<Outcome>>,
    worker: Option<JoinHandle<()>>,
}
impl Default for PseudoTerminal {
    fn default() -> Self {
        Self::new()
    }
}
impl PseudoTerminal {
    /// Create an unstarted PTY handle with an 80x24 initial size.
    pub fn new() -> Self {
        Self {
            size: (80, 24),
            child_id: None,
            commands: None,
            output: None,
            stop: Arc::new(AtomicBool::new(false)),
            outcome: Arc::new(Mutex::new(Outcome::default())),
            worker: None,
        }
    }
    /// Start the configured executable with a controlling PTY on all three streams.
    pub fn spawn(&mut self, config: &TerminalConfig) -> TerminalResult<()> {
        if self.worker.is_some() {
            return Err(error("PTY already owns a child; stop it before spawning"));
        }
        validate_size(config.size.0, config.size.1)?;
        let shell = config
            .shell
            .clone()
            .or_else(|| std::env::var("SHELL").ok())
            .unwrap_or_else(|| "/bin/sh".into());
        let mut command = Command::new(shell);
        command
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .envs(config.env.iter().cloned());
        if let Some(directory) = &config.working_directory {
            command.current_dir(directory);
        }
        let child = PtyChild::spawn(command, config.size.0, config.size.1).map_err(error)?;
        let child_id = child.id();
        let (sender, receiver) = mpsc::sync_channel(16);
        let (output_sender, output_receiver) = mpsc::sync_channel(16);
        let stop = Arc::new(AtomicBool::new(false));
        let outcome = Arc::new(Mutex::new(Outcome::default()));
        let worker_stop = stop.clone();
        let worker_outcome = outcome.clone();
        let worker = thread::Builder::new()
            .name("terminal-pty".into())
            .spawn(move || {
                run(child, receiver, output_sender, worker_stop, worker_outcome);
            })
            .map_err(error)?;
        self.size = config.size;
        self.child_id = Some(child_id);
        self.commands = Some(sender);
        self.output = Some(output_receiver);
        self.stop = stop;
        self.outcome = outcome;
        self.worker = Some(worker);
        Ok(())
    }
    /// Queue at most 64 KiB of input; report backpressure or a stopped child.
    pub fn write_input(&self, data: &[u8]) -> TerminalResult<()> {
        if data.len() > MAX_INPUT {
            return Err(error("PTY input exceeds 64 KiB; send bounded chunks"));
        }
        let (sender, reply) = mpsc::channel();
        self.request(Request::Input(data.to_vec(), sender), reply)
    }
    fn request(&self, request: Request, reply: Receiver<TerminalResult<()>>) -> TerminalResult<()> {
        self.commands
            .as_ref()
            .ok_or_else(|| error("PTY is not running"))?
            .try_send(request)
            .map_err(|e| match e {
                mpsc::TrySendError::Full(_) => {
                    error("PTY command queue is full; retry after draining output")
                }
                mpsc::TrySendError::Disconnected(_) => error("PTY worker has stopped"),
            })?;
        reply
            .recv_timeout(Duration::from_secs(1))
            .map_err(|_| error("PTY command was not acknowledged"))?
    }
    /// Receive a bounded output chunk; None waits until output or worker exit.
    pub fn read_output(&self, timeout: Option<Duration>) -> TerminalResult<Option<Vec<u8>>> {
        let Some(output) = &self.output else {
            return Ok(None);
        };
        let result = match timeout {
            Some(timeout) => output.recv_timeout(timeout),
            None => output
                .recv()
                .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };
        match result {
            Ok(bytes) => Ok(Some(bytes)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let outcome = self
                    .outcome
                    .lock()
                    .map_err(|_| error("PTY outcome lock poisoned"))?;
                match &outcome.error {
                    Some(message) => Err(error(message)),
                    None => Ok(None),
                }
            }
        }
    }
    /// Return the reaped exit code, or 128 plus the terminating Unix signal.
    pub fn try_wait(&self) -> TerminalResult<Option<i32>> {
        let outcome = self
            .outcome
            .lock()
            .map_err(|_| error("PTY outcome lock poisoned"))?;
        if let Some(message) = &outcome.error {
            return Err(error(message));
        }
        Ok(outcome.exit)
    }
    /// Update the actual PTY window size; zero and excessive sizes are rejected.
    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        validate_size(width, height)?;
        if self.commands.is_some() {
            let (sender, reply) = mpsc::channel();
            self.request(Request::Resize(width, height, sender), reply)?;
        }
        self.size = (width, height);
        Ok(())
    }
    /// Last successfully applied size in columns and rows.
    pub fn size(&self) -> (u16, u16) {
        self.size
    }
    /// PID of the directly owned executable, including after its exit.
    pub fn child_id(&self) -> Option<u32> {
        self.child_id
    }
    /// Stop IO, terminate/reap the owned child and join the worker. Idempotent.
    pub fn kill(&mut self) -> TerminalResult<()> {
        self.stop.store(true, Ordering::Release);
        self.commands.take();
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| error("PTY worker panicked"))?;
        }
        let outcome = self
            .outcome
            .lock()
            .map_err(|_| error("PTY outcome lock poisoned"))?;
        match &outcome.error {
            Some(message) => Err(error(message)),
            None => Ok(()),
        }
    }
}
impl Drop for PseudoTerminal {
    fn drop(&mut self) {
        if self.worker.is_some() {
            if let Err(error) = self.kill() {
                log::warn!("Terminal PTY cleanup: {error}");
            }
        }
    }
}

fn validate_size(width: u16, height: u16) -> TerminalResult<()> {
    if width == 0 || height == 0 || usize::from(width) * usize::from(height) > 262_144 {
        Err(TerminalError::InvalidSize { width, height })
    } else {
        Ok(())
    }
}
fn error(message: impl std::fmt::Display) -> TerminalError {
    TerminalError::Pty(message.to_string())
}

fn run(
    mut child: PtyChild,
    commands: Receiver<Request>,
    output: SyncSender<Vec<u8>>,
    stop: Arc<AtomicBool>,
    outcome: Arc<Mutex<Outcome>>,
) {
    let result = run_io(&mut child, commands, output, &stop);
    let stopped = child.stop();
    if let Ok(mut outcome) = outcome.lock() {
        if let Err(error) = result {
            outcome.error = Some(error.to_string());
        }
        match stopped {
            Ok(status) => {
                outcome.exit = Some(
                    status
                        .code()
                        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
                )
            }
            Err(error) => {
                outcome.error.get_or_insert_with(|| error.to_string());
            }
        }
    }
}
fn run_io(
    child: &mut PtyChild,
    commands: Receiver<Request>,
    output: SyncSender<Vec<u8>>,
    stop: &AtomicBool,
) -> io::Result<()> {
    let mut input = VecDeque::new();
    let mut pending = None;
    let mut eof = false;
    let mut after_exit = 0usize;
    while !stop.load(Ordering::Acquire) {
        let exited = child.try_wait()?.is_some();
        for _ in 0..16 {
            match commands.try_recv() {
                Ok(Request::Input(bytes, reply)) => {
                    let result = if input.len() + bytes.len() > MAX_INPUT {
                        Err(error(
                            "PTY pending input exceeds 64 KiB; retry after the child reads",
                        ))
                    } else {
                        input.extend(bytes);
                        Ok(())
                    };
                    let _ = reply.send(result);
                }
                Ok(Request::Resize(width, height, reply)) => {
                    let _ = reply.send(child.resize(width, height).map_err(error));
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
            }
        }
        if !input.is_empty() {
            match child.master.write(input.as_slices().0) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "PTY input write returned zero",
                    ))
                }
                Ok(count) => {
                    input.drain(..count);
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) => {}
                Err(error) if error.raw_os_error() == Some(libc::EIO) => {
                    input.clear();
                    eof = true;
                }
                Err(error) => return Err(error),
            }
        }
        if let Some(bytes) = pending.take() {
            match output.try_send(bytes) {
                Ok(()) => {}
                Err(mpsc::TrySendError::Full(bytes)) => pending = Some(bytes),
                Err(mpsc::TrySendError::Disconnected(_)) => return Ok(()),
            }
        }
        if pending.is_none() && !eof {
            let mut bytes = [0; super::OUTPUT_CHUNK_SIZE];
            match child.master.read(&mut bytes) {
                Ok(0) => eof = true,
                Ok(count) => {
                    pending = Some(bytes[..count].to_vec());
                    if exited {
                        after_exit += count;
                        eof = after_exit >= 64 * 1024;
                    }
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                    ) =>
                {
                    if exited {
                        eof = true;
                    }
                }
                Err(error) if error.raw_os_error() == Some(libc::EIO) => eof = true,
                Err(error) => return Err(error),
            }
        }
        if eof && pending.is_none() {
            return Ok(());
        }
        // A full output queue stops reads but must not spin on a readable master.
        if pending.is_some() {
            thread::sleep(Duration::from_millis(2));
        } else {
            child.wait_ready(!input.is_empty())?;
        }
    }
    Ok(())
}
