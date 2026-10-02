mod job;
mod native;
mod runtime;
use crate::terminal::{TerminalConfig, TerminalError, TerminalResult};
use native::Session;
use std::{
    io::{self, Read, Write},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
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
        let mut child = Session::new(config)?;
        child.launch(config)?;
        let child_id = child.child_id();
        let (sender, receiver) = mpsc::sync_channel(16);
        let (output_sender, output_receiver) = mpsc::sync_channel(16);
        let stop = Arc::new(AtomicBool::new(false));
        let outcome = Arc::new(Mutex::new(Outcome::default()));
        let worker_stop = stop.clone();
        let worker_outcome = outcome.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("terminal-pty".into())
            .spawn(move || {
                run(
                    child,
                    receiver,
                    output_sender,
                    worker_stop,
                    worker_outcome,
                    started_tx,
                );
            })
            .map_err(error)?;
        if started_rx.recv().is_err() {
            worker
                .join()
                .map_err(|_| error("PTY startup worker panicked"))?;
            let outcome = outcome
                .lock()
                .map_err(|_| error("PTY outcome lock poisoned"))?;
            return Err(error(
                outcome.error.as_deref().unwrap_or("PTY IO startup failed"),
            ));
        }
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
    /// Return the actual Windows exit code after process and IO cleanup.
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
    native::validate_size(width, height)
}
fn error(message: impl std::fmt::Display) -> TerminalError {
    TerminalError::Pty(message.to_string())
}

fn record_error(outcome: &Mutex<Outcome>, failure: impl std::fmt::Display) {
    if let Ok(mut outcome) = outcome.lock() {
        outcome.error.get_or_insert_with(|| failure.to_string());
    }
}

fn run(
    mut child: Session,
    commands: Receiver<Request>,
    output: SyncSender<Vec<u8>>,
    stop: Arc<AtomicBool>,
    outcome: Arc<Mutex<Outcome>>,
    started: mpsc::Sender<()>,
) {
    let result = run_io(&mut child, commands, output, &stop, &outcome, started);
    stop.store(true, Ordering::Release);
    let closed = child.close();
    if let Err(failure) = result {
        record_error(&outcome, failure);
    }
    match closed {
        Ok(exit) => {
            if let Ok(mut outcome) = outcome.lock() {
                outcome.exit = Some(exit);
            }
        }
        Err(failure) => record_error(&outcome, failure),
    }
}

fn run_io(
    child: &mut Session,
    commands: Receiver<Request>,
    output: SyncSender<Vec<u8>>,
    stop: &Arc<AtomicBool>,
    outcome: &Arc<Mutex<Outcome>>,
    started: mpsc::Sender<()>,
) -> TerminalResult<()> {
    let mut reader = child.take_output();
    let reader_stop = stop.clone();
    let reader_outcome = outcome.clone();
    let reader_worker = thread::Builder::new()
        .name("terminal-conpty-output".into())
        .spawn(move || {
            if let Err(failure) = read_output(&mut reader, output, &reader_stop) {
                record_error(&reader_outcome, failure);
                reader_stop.store(true, Ordering::Release);
            }
        })
        .map_err(error)?;
    // Every subsequent error closes ConPTY while this reader keeps draining.
    let result = (|| {
        let (input_tx, input_rx) = mpsc::sync_channel::<Vec<u8>>(16);
        let pending = Arc::new(AtomicUsize::new(0));
        let mut writer = child.take_input();
        let writer_stop = stop.clone();
        let writer_outcome = outcome.clone();
        let writer_pending = pending.clone();
        let writer_worker = thread::Builder::new()
            .name("terminal-conpty-input".into())
            .spawn(move || {
                while !writer_stop.load(Ordering::Acquire) {
                    match input_rx.recv_timeout(Duration::from_millis(10)) {
                        Ok(bytes) => {
                            let result = writer.write_all(&bytes);
                            writer_pending.fetch_sub(bytes.len(), Ordering::AcqRel);
                            if let Err(failure) = result {
                                if !native::is_closed_pipe(&failure)
                                    && !writer_stop.load(Ordering::Acquire)
                                {
                                    record_error(&writer_outcome, failure);
                                }
                                writer_stop.store(true, Ordering::Release);
                                break;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .map_err(error)?;
        let _ = started.send(());
        let result = supervise(child, commands, &input_tx, &pending, stop);
        stop.store(true, Ordering::Release);
        drop(input_tx);
        let closed = child.close();
        let joined = writer_worker
            .join()
            .map_err(|_| error("PTY input worker panicked"));
        result?;
        joined?;
        closed.map(|_| ())
    })();
    stop.store(true, Ordering::Release);
    let closed = child.close();
    let joined = reader_worker
        .join()
        .map_err(|_| error("PTY output worker panicked"));
    result?;
    closed?;
    joined
}

fn supervise(
    child: &Session,
    commands: Receiver<Request>,
    input: &SyncSender<Vec<u8>>,
    pending: &AtomicUsize,
    stop: &AtomicBool,
) -> TerminalResult<()> {
    while !stop.load(Ordering::Acquire) && child.try_wait()?.is_none() {
        match commands.recv_timeout(Duration::from_millis(2)) {
            Ok(Request::Resize(width, height, reply)) => {
                let _ = reply.send(child.resize(width, height));
            }
            Ok(Request::Input(bytes, reply)) => {
                let count = bytes.len();
                let result = if pending
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                        n.checked_add(count).filter(|&n| n <= MAX_INPUT)
                    })
                    .is_err()
                {
                    Err(error(
                        "PTY pending input exceeds 64 KiB; retry after the child reads",
                    ))
                } else if input.try_send(bytes).is_err() {
                    pending.fetch_sub(count, Ordering::AcqRel);
                    Err(error("PTY input queue is full or stopped"))
                } else {
                    Ok(())
                };
                let _ = reply.send(result);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}

fn read_output(
    reader: &mut impl Read,
    output: SyncSender<Vec<u8>>,
    stop: &AtomicBool,
) -> io::Result<()> {
    let mut buffer = [0; super::OUTPUT_CHUNK_SIZE];
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => count,
            Err(failure) if native::is_closed_pipe(&failure) => return Ok(()),
            Err(failure) if failure.kind() == io::ErrorKind::Interrupted => continue,
            Err(failure) => return Err(failure),
        };
        let mut bytes = buffer[..count].to_vec();
        loop {
            match output.try_send(bytes) {
                Ok(()) => break,
                Err(mpsc::TrySendError::Full(returned)) if !stop.load(Ordering::Acquire) => {
                    bytes = returned;
                    thread::sleep(Duration::from_millis(2));
                }
                // ClosePseudoConsole may write a final frame. Keep draining if
                // the consumer stopped or its bounded output queue is full.
                Err(_) => break,
            }
        }
    }
}
