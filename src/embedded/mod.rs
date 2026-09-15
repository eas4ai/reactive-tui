//! Optional Unix shell sessions interpreted by libghostty on an owned worker.
//!
//! The worker uses nonblocking PTY IO, a 16-command queue, at most 64 KiB of
//! pending input/replies, and one latest owned frame. Callers may retain their
//! own snapshots; the session never queues old output frames.

mod keyboard;
mod pty;
mod snapshot;
mod worker;

use crate::app::{AppWaker, RootComponent, RootUpdate};
use crate::backend::CellFrame;
use crate::component::Element;
use crate::error::{ReactiveError, Result};
use crate::event::{
    router::EventResult,
    types::{Event, KeyEvent},
};
use std::process::{Command, ExitStatus};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, SyncSender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Latest terminal state and process outcome. Errors remain observable after exit.
#[derive(Clone, Debug)]
pub struct SessionSnapshot {
    /// Monotonic change counter for frame, exit, and error updates.
    pub revision: u64,
    /// Complete visible screen; owns no references to native terminal memory.
    pub frame: Arc<CellFrame>,
    /// Direct child's reaped exit status, when available.
    pub exit_status: Option<ExitStatus>,
    /// Worker or IO failure with operation context.
    pub error: Option<String>,
    /// Whether the worker has completed cleanup.
    pub stopped: bool,
}

#[derive(Default)]
struct SharedState {
    wake: Option<AppWaker>,
    revision: u64,
    frame: Option<Arc<CellFrame>>,
    exit_status: Option<ExitStatus>,
    error: Option<String>,
    stopped: bool,
}

enum SessionCommand {
    Key(KeyEvent),
    Resize(u16, u16, mpsc::Sender<Result<()>>),
}

/// A real child PTY and its libghostty interpreter. Drop stops and joins the worker.
pub struct EmbeddedSession {
    commands: SyncSender<SessionCommand>,
    shared: Arc<Mutex<SharedState>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    child_id: u32,
}

impl EmbeddedSession {
    /// Spawn the requested command with its arguments, environment and directory.
    /// Stdin, stdout, and stderr are attached to a new controlling PTY.
    pub fn spawn(command: Command, width: u16, height: u16) -> Result<Self> {
        CellFrame::validate_size(width, height)?;
        let (commands, receiver) = mpsc::sync_channel(16);
        let (ready, initialized) = mpsc::channel();
        let shared = Arc::new(Mutex::new(SharedState::default()));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_shared = Arc::clone(&shared);
        let worker_stop = Arc::clone(&stop);
        let worker = thread::Builder::new()
            .name("embedded-terminal".into())
            .spawn(move || {
                worker::run(
                    command,
                    (width, height),
                    receiver,
                    worker_shared,
                    worker_stop,
                    ready,
                );
            })?;
        let mut session = Self {
            commands,
            shared,
            stop,
            worker: Some(worker),
            child_id: 0,
        };
        match initialized.recv().map_err(|_| closed())? {
            Ok(pid) => {
                session.child_id = pid;
                Ok(session)
            }
            Err(error) => Err(error),
        }
    }

    /// Notify one App when a frame, exit status or error is published.
    pub fn attach_waker(&self, wake: AppWaker) -> Result<()> {
        self.shared.lock().map_err(|_| closed())?.wake = Some(wake.clone());
        wake.wake();
        Ok(())
    }

    /// PID of the directly owned executable, for diagnostics.
    pub fn child_id(&self) -> u32 {
        self.child_id
    }

    /// Read the latest complete frame and process outcome without waiting for IO.
    pub fn snapshot(&self) -> Result<SessionSnapshot> {
        let state = self.shared.lock().map_err(|_| closed())?;
        if !state.stopped && self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            return Err(closed());
        }
        Ok(SessionSnapshot {
            revision: state.revision,
            frame: Arc::clone(state.frame.as_ref().ok_or_else(closed)?),
            exit_status: state.exit_status,
            error: state.error.clone(),
            stopped: state.stopped,
        })
    }

    /// Queue one key. A full queue returns backpressure instead of blocking App.
    /// Later IO errors appear in snapshot().error and TerminalView::update.
    pub fn send_key(&self, key: KeyEvent) -> Result<()> {
        if self.try_key(key)?.is_some() {
            return Err(ReactiveError::resource(
                "embedded terminal command queue is full",
            ));
        }
        Ok(())
    }

    // Return the unqueued key so TerminalView can pause input and retry on wake.
    fn try_key(&self, key: KeyEvent) -> Result<Option<KeyEvent>> {
        self.ensure_running()?;
        match self.commands.try_send(SessionCommand::Key(key)) {
            Ok(()) => Ok(None),
            Err(mpsc::TrySendError::Full(SessionCommand::Key(key))) => Ok(Some(key)),
            Err(_) => Err(closed()),
        }
    }

    /// Resize both the PTY and interpreter, publishing the new frame before return.
    /// A zero dimension retains the previous valid size.
    pub fn resize(&self, width: u16, height: u16) -> Result<()> {
        if width == 0 || height == 0 || self.snapshot()?.frame.size() == (width, height) {
            return Ok(());
        }
        CellFrame::validate_size(width, height)?;
        let (reply, result) = mpsc::channel();
        self.send(SessionCommand::Resize(width, height, reply))?;
        result
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| ReactiveError::terminal("embedded terminal resize did not complete"))?
    }

    fn ensure_running(&self) -> Result<()> {
        let snapshot = self.snapshot()?;
        if let Some(error) = snapshot.error {
            return Err(ReactiveError::terminal(error));
        }
        if snapshot.stopped || snapshot.exit_status.is_some() {
            return Err(closed());
        }
        Ok(())
    }

    fn send(&self, command: SessionCommand) -> Result<()> {
        self.ensure_running()?;
        self.commands
            .try_send(command)
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => {
                    ReactiveError::resource("embedded terminal command queue is full")
                }
                mpsc::TrySendError::Disconnected(_) => closed(),
            })
    }

    /// Kill/reap the owned child and join the worker. Safe to call repeatedly.
    pub fn shutdown(&mut self) -> Result<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| closed())?;
        }
        let shared = self.shared.lock().map_err(|_| closed())?;
        match &shared.error {
            Some(error) => Err(ReactiveError::terminal(error.clone())),
            None => Ok(()),
        }
    }
}

impl Drop for EmbeddedSession {
    fn drop(&mut self) {
        if self.worker.is_some() {
            if let Err(error) = self.shutdown() {
                log::warn!("Embedded session cleanup: {error}");
            }
        }
    }
}

/// A root component that displays one terminal screen through App.
pub struct TerminalView {
    session: EmbeddedSession,
    revision: u64,
    exit_shown: AtomicBool,
    wake: Option<AppWaker>,
    pending_key: Option<KeyEvent>,
}

impl TerminalView {
    /// Wrap an owned session. App's viewport becomes the terminal's pane size.
    pub fn new(session: EmbeddedSession) -> Self {
        Self {
            session,
            revision: 0,
            exit_shown: AtomicBool::new(false),
            wake: None,
            pending_key: None,
        }
    }
}

impl RootComponent for TerminalView {
    fn render(&self) -> Element {
        Element::empty()
    }
    fn attach_waker(&mut self, wake: AppWaker) {
        // Shared-state poisoning is reported by the next snapshot/update call.
        if let Ok(mut shared) = self.session.shared.lock() {
            shared.wake = Some(wake.clone());
        }
        self.wake = Some(wake);
    }
    fn accepts_input(&self) -> bool {
        self.pending_key.is_none()
    }
    fn wake_driven(&self) -> bool {
        true
    }
    fn cell_frame(&self) -> Result<Option<Arc<CellFrame>>> {
        let snapshot = self.session.snapshot()?;
        if snapshot.exit_status.is_some() {
            self.exit_shown.store(true, Ordering::Release);
            if let Some(wake) = &self.wake {
                wake.wake();
            }
        }
        Ok(Some(snapshot.frame))
    }
    fn update(&mut self) -> Result<RootUpdate> {
        let snapshot = self.session.snapshot()?;
        if let Some(error) = snapshot.error {
            return Err(ReactiveError::terminal(error));
        }
        if self.exit_shown.load(Ordering::Acquire) {
            return Ok(RootUpdate::Exit);
        }
        if snapshot.exit_status.is_some() {
            self.pending_key = None;
        } else if let Some(key) = self.pending_key.take() {
            self.pending_key = self.session.try_key(key)?;
        }
        if snapshot.revision != self.revision {
            self.revision = snapshot.revision;
            Ok(RootUpdate::Redraw)
        } else {
            Ok(RootUpdate::Unchanged)
        }
    }
    fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.session.resize(width, height)
    }
    fn try_handle_event(&mut self, event: &Event) -> Result<EventResult> {
        match event {
            Event::Key(key) => {
                if self.pending_key.is_some() {
                    return Err(ReactiveError::resource(
                        "embedded terminal input is paused; retry after update",
                    ));
                }
                if self.session.snapshot()?.exit_status.is_some() {
                    return Ok(EventResult::Ignored);
                }
                self.pending_key = self.session.try_key(key.clone())?;
                Ok(EventResult::Handled)
            }
            _ => Ok(EventResult::Ignored),
        }
    }
}

fn closed() -> ReactiveError {
    ReactiveError::terminal("embedded terminal worker is closed or failed")
}
