use super::{
    keyboard,
    pty::PtyChild,
    snapshot::{capture, native_error},
    SessionCommand, SharedState,
};
use crate::error::{ReactiveError, Result};
use libghostty_vt::{key, RenderState, Terminal};
use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{self, Read, Write},
    process::{Command, ExitStatus},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    time::Duration,
};

const IO_BUDGET: usize = 65_536;
const INPUT_LIMIT: usize = 65_536;

#[derive(Default)]
struct PendingInput {
    bytes: VecDeque<u8>,
    overflow: bool,
}
impl PendingInput {
    fn append(&mut self, bytes: &[u8]) {
        if bytes.len() > INPUT_LIMIT.saturating_sub(self.bytes.len()) {
            self.overflow = true;
        } else {
            self.bytes.extend(bytes);
        }
    }
    fn check(&self) -> Result<()> {
        if self.overflow {
            Err(ReactiveError::resource(
                "embedded terminal pending input exceeded 64 KiB",
            ))
        } else {
            Ok(())
        }
    }
}

pub(super) fn run(
    command: Command,
    size: (u16, u16),
    receiver: mpsc::Receiver<SessionCommand>,
    shared: Arc<Mutex<SharedState>>,
    stop: Arc<AtomicBool>,
    ready: mpsc::Sender<Result<u32>>,
) {
    let pending = RefCell::new(PendingInput::default());
    let setup = (|| {
        let mut terminal = Terminal::new(size.0, size.1).map_err(native_error)?;
        terminal
            .set_default_mode(libghostty_vt::terminal::Mode::GRAPHEME_CLUSTER, true)
            .map_err(native_error)?;
        terminal
            .set_mode(libghostty_vt::terminal::Mode::GRAPHEME_CLUSTER, true)
            .map_err(native_error)?;
        terminal
            .set_scrollback_max_bytes(Some(1_048_576))
            .map_err(native_error)?;
        terminal
            .set_scrollback_max_lines(Some(2000))
            .map_err(native_error)?;
        terminal
            .set_apc_max_bytes(Some(IO_BUDGET))
            .map_err(native_error)?;
        terminal
            .set_glyph_protocol_enabled(false)
            .map_err(native_error)?;
        terminal
            .on_pty_write(|_, data| pending.borrow_mut().append(data))
            .map_err(native_error)?;
        let mut state = RenderState::new().map_err(native_error)?;
        let encoder = key::Encoder::new().map_err(native_error)?;
        let child = PtyChild::spawn(command, size.0, size.1)
            .map_err(|error| io_error("spawn PTY child", error))?;
        publish(&shared, &terminal, &mut state)?;
        Ok::<_, ReactiveError>((terminal, state, encoder, child))
    })();
    let (mut terminal, mut state, mut encoder, mut child) = match setup {
        Ok(setup) => setup,
        Err(error) => {
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(child.id())).is_err() {
        return;
    }
    let result = drive(
        &mut child,
        &mut terminal,
        &mut state,
        &mut encoder,
        &pending,
        &receiver,
        &shared,
        &stop,
    );
    let cleanup = child.stop();
    if let Ok(mut shared) = shared.lock() {
        shared.exit_status = cleanup.as_ref().ok().copied().or(shared.exit_status);
        shared.error = match (result, cleanup) {
            (Err(error), Err(cleanup)) => {
                Some(format!("{error}; child cleanup also failed: {cleanup}"))
            }
            (Err(error), _) => Some(error.to_string()),
            (_, Err(error)) => Some(format!("child cleanup: {error}")),
            _ => None,
        };
        shared.stopped = true;
        shared.revision = shared.revision.saturating_add(1);
        let wake = shared.wake.clone();
        drop(shared);
        if let Some(wake) = wake {
            wake.wake();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn drive<'a>(
    child: &mut PtyChild,
    terminal: &mut Terminal<'a, '_>,
    state: &mut RenderState<'a>,
    encoder: &mut key::Encoder<'_>,
    pending: &RefCell<PendingInput>,
    receiver: &mpsc::Receiver<SessionCommand>,
    shared: &Mutex<SharedState>,
    stop: &AtomicBool,
) -> Result<()> {
    let mut eof = false;
    loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut consumed = false;
        for _ in 0..16 {
            match receiver.try_recv() {
                Ok(SessionCommand::Key(key)) => {
                    consumed = true;
                    pending
                        .borrow_mut()
                        .append(&keyboard::encode(terminal, encoder, &key)?);
                }
                Ok(SessionCommand::Resize(width, height, reply)) => {
                    consumed = true;
                    let result = child
                        .resize(width, height)
                        .map_err(|error| io_error("resize PTY", error))
                        .and_then(|()| terminal.resize(width, height, 0, 0).map_err(native_error))
                        .and_then(|()| publish(shared, terminal, state));
                    match result {
                        Ok(()) => {
                            let _ = reply.send(Ok(()));
                        }
                        Err(error) => {
                            let _ = reply.send(Err(ReactiveError::terminal(error.to_string())));
                            return Err(error);
                        }
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
            }
        }
        if consumed {
            let wake = shared.lock().map_err(|_| super::closed())?.wake.clone();
            if let Some(wake) = wake {
                wake.wake();
            }
        }
        let read = if eof {
            0
        } else {
            read_output(child, terminal, &mut eof)?
        };
        pending.borrow().check()?;
        flush_input(&mut child.master, &mut pending.borrow_mut())?;
        if read > 0 {
            publish(shared, terminal, state)?;
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| io_error("reap PTY child", error))?
        {
            // Output may arrive between the earlier nonblocking read and wait.
            // Once exit is observed, perform a final drain before publishing it.
            let trailing = if eof {
                0
            } else {
                read_output(child, terminal, &mut eof)?
            };
            pending.borrow().check()?;
            if trailing > 0 {
                publish(shared, terminal, state)?;
            }
            publish_exit(shared, status)?;
            if trailing < IO_BUDGET {
                return Ok(());
            }
        }
        if eof {
            std::thread::sleep(Duration::from_millis(10));
        } else {
            child
                .wait_ready(!pending.borrow().bytes.is_empty())
                .map_err(|error| io_error("poll PTY", error))?;
        }
    }
}

fn read_output(
    child: &mut PtyChild,
    terminal: &mut Terminal<'_, '_>,
    eof: &mut bool,
) -> Result<usize> {
    let mut bytes = [0u8; 8192];
    let mut total = 0;
    while total < IO_BUDGET {
        match child.master.read(&mut bytes) {
            Ok(0) => {
                *eof = true;
                break;
            }
            Ok(count) => {
                terminal.vt_write(&bytes[..count]);
                total += count;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => break,
            // Linux reports slave closure as EIO rather than a zero-byte read.
            Err(error) if cfg!(target_os = "linux") && error.raw_os_error() == Some(libc::EIO) => {
                *eof = true;
                break;
            }
            Err(error) => return Err(io_error("read PTY", error)),
        }
    }
    Ok(total)
}

fn flush_input(writer: &mut impl Write, pending: &mut PendingInput) -> Result<()> {
    pending.check()?;
    while !pending.bytes.is_empty() {
        let (first, _) = pending.bytes.as_slices();
        match writer.write(first) {
            Ok(0) => return Err(ReactiveError::terminal("write PTY made no progress")),
            Ok(count) => {
                pending.bytes.drain(..count);
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => break,
            Err(error) => return Err(io_error("write PTY", error)),
        }
    }
    Ok(())
}

fn publish<'a>(
    shared: &Mutex<SharedState>,
    terminal: &Terminal<'a, '_>,
    state: &mut RenderState<'a>,
) -> Result<()> {
    let frame = Arc::new(capture(terminal, state)?);
    let mut shared = shared.lock().map_err(|_| super::closed())?;
    shared.frame = Some(frame);
    shared.revision = shared.revision.saturating_add(1);
    let wake = shared.wake.clone();
    drop(shared);
    if let Some(wake) = wake {
        wake.wake();
    }
    Ok(())
}

fn publish_exit(shared: &Mutex<SharedState>, status: ExitStatus) -> Result<()> {
    let mut shared = shared.lock().map_err(|_| super::closed())?;
    if shared.exit_status.is_none() {
        shared.exit_status = Some(status);
        shared.revision = shared.revision.saturating_add(1);
    }
    let wake = shared.wake.clone();
    drop(shared);
    if let Some(wake) = wake {
        wake.wake();
    }
    Ok(())
}

fn io_error(operation: &str, error: io::Error) -> ReactiveError {
    ReactiveError::terminal(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pending_input_is_bounded_and_io_failure_is_not_success() {
        let mut pending = PendingInput::default();
        pending.append(&vec![b'x'; INPUT_LIMIT]);
        assert!(pending.check().is_ok());
        pending.append(b"x");
        assert_eq!(pending.bytes.len(), INPUT_LIMIT);
        assert!(pending.check().is_err());
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "controlled failure",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut pending = PendingInput::default();
        pending.append(b"hello");
        assert!(flush_input(&mut Broken, &mut pending)
            .unwrap_err()
            .to_string()
            .contains("write PTY"));
        assert_eq!(pending.bytes.len(), 5);
        let mut output = Vec::new();
        flush_input(&mut output, &mut pending).unwrap();
        assert_eq!(output, b"hello");
        assert!(pending.bytes.is_empty());
    }
}
