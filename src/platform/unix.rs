//! Unix/POSIX TTY implementation for direct terminal access
//!
//! Native POSIX terminal interface with signal handling

use super::{input_receiver::InputWorker, InputReceiver, PlatformTty};
use crate::error::Result;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::{Arc, Weak};
use std::thread::JoinHandle;
use std::time::Duration;

static SIGNAL_HANDLERS: Mutex<Vec<SignalHandler>> = Mutex::new(Vec::new());
static SIGNAL_LIFECYCLE: Mutex<SignalLifecycle> = Mutex::new(SignalLifecycle {
    owners: 0,
    action: None,
    stop: None,
    worker: None,
});
// FIX: Use OnceLock for thread-safe initialization instead of unsafe global
static GLOBAL_TTY: OnceLock<Mutex<Option<Weak<TtyState>>>> = OnceLock::new();

/// Signal handler for Unix systems
#[derive(Clone, Copy)]
pub struct SignalHandler {
    /// Context pointer passed to the callback
    pub context: *mut std::ffi::c_void,
    /// Callback function to handle the signal
    pub callback: extern "C" fn(*mut std::ffi::c_void),
}

unsafe impl Send for SignalHandler {}
unsafe impl Sync for SignalHandler {}

/// Unix TTY implementation using direct POSIX calls
pub struct UnixTty {
    /// File descriptor for /dev/tty
    fd: RawFd,
    /// Original terminal settings to restore on exit
    state: Arc<TtyState>,
}

struct TtyState {
    fd: OwnedFd,
    original_termios: libc::termios,
    restored: AtomicBool,
    workers: Mutex<Vec<Weak<InputWorker>>>,
    signal_owner: AtomicBool,
}

struct SignalLifecycle {
    owners: usize,
    action: Option<signal_hook::SigId>,
    stop: Option<UnixStream>,
    worker: Option<JoinHandle<()>>,
}

impl TtyState {
    fn restore(&self) -> Result<()> {
        if self.restored.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let result = unsafe {
            libc::tcsetattr(self.fd.as_raw_fd(), libc::TCSAFLUSH, &self.original_termios)
        };
        if result != 0 {
            self.restored.store(false, Ordering::Release);
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }
}

impl Drop for TtyState {
    fn drop(&mut self) {
        let workers = self
            .workers
            .get_mut()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        for worker in &workers {
            worker.cancel();
        }
        for worker in workers {
            worker.stop();
        }
        if self.signal_owner.swap(false, Ordering::AcqRel) {
            release_signal_handler();
        }
        let _ = self.restore();
    }
}

impl PlatformTty for UnixTty {
    fn write(&self, data: &[u8]) -> Result<usize> {
        let result =
            unsafe { libc::write(self.fd, data.as_ptr() as *const libc::c_void, data.len()) };

        if result < 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(result as usize)
        }
    }

    fn size(&self) -> Result<(u16, u16)> {
        let mut winsize = std::mem::MaybeUninit::<libc::winsize>::uninit();
        let result = unsafe { libc::ioctl(self.fd, libc::TIOCGWINSZ, winsize.as_mut_ptr()) };

        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        let winsize = unsafe { winsize.assume_init() };
        Ok((winsize.ws_col, winsize.ws_row))
    }

    fn restore(&self) -> Result<()> {
        self.state.restore()
    }
}

impl UnixTty {
    /// Initialize TTY by opening /dev/tty and setting raw mode
    pub fn init() -> Result<Self> {
        // Open /dev/tty for direct terminal access
        let fd = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDWR | libc::O_CLOEXEC) };

        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        // Get current terminal settings
        let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
        let result = unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) };
        if result != 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error().into());
        }

        let original_termios = unsafe { termios.assume_init() };

        // Set raw mode
        let mut raw_termios = original_termios;
        unsafe { libc::cfmakeraw(&mut raw_termios) };

        let result = unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &raw_termios) };
        if result != 0 {
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error().into());
        }

        // The shared owner restores raw mode and closes its descriptor once.
        // Construct it before fallible setup so every error releases the session.
        let state = Arc::new(TtyState {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
            original_termios,
            restored: AtomicBool::new(false),
            workers: Mutex::new(Vec::new()),
            signal_owner: AtomicBool::new(false),
        });
        install_signal_handlers()?;
        state.signal_owner.store(true, Ordering::Release);
        let tty = UnixTty { fd, state };

        // Store global reference for panic recovery (thread-safe)
        let global_tty = GLOBAL_TTY.get_or_init(|| Mutex::new(None));
        if let Ok(mut guard) = global_tty.lock() {
            *guard = Some(Arc::downgrade(&tty.state));
        }

        Ok(tty)
    }

    /// Write raw bytes to terminal (duplicate method for compatibility)
    pub fn write_raw(&self, data: &[u8]) -> Result<usize> {
        self.write(data)
    }

    /// Set non-blocking mode
    pub fn set_nonblocking(&self, nonblocking: bool) -> Result<()> {
        let flags = unsafe { libc::fcntl(self.fd, libc::F_GETFL) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error().into());
        }

        let new_flags = if nonblocking {
            flags | libc::O_NONBLOCK
        } else {
            flags & !libc::O_NONBLOCK
        };

        let result = unsafe { libc::fcntl(self.fd, libc::F_SETFL, new_flags) };
        if result < 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }

    /// Read raw bytes from terminal with timeout
    pub fn read(&self, buf: &mut [u8], timeout: Option<Duration>) -> Result<usize> {
        if let Some(timeout) = timeout {
            // Use select() for timeout
            let mut fd_set = std::mem::MaybeUninit::<libc::fd_set>::uninit();
            unsafe {
                libc::FD_ZERO(fd_set.as_mut_ptr());
                libc::FD_SET(self.fd, fd_set.as_mut_ptr());
            }

            let mut timeval = libc::timeval {
                tv_sec: timeout.as_secs() as libc::time_t,
                tv_usec: timeout.subsec_micros() as libc::suseconds_t,
            };

            let result = unsafe {
                libc::select(
                    self.fd + 1,
                    fd_set.as_mut_ptr(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut timeval,
                )
            };

            if result < 0 {
                return Err(std::io::Error::last_os_error().into());
            } else if result == 0 {
                // Timeout
                return Ok(0);
            }
        }

        // Read data
        let result =
            unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };

        if result < 0 {
            Err(std::io::Error::last_os_error().into())
        } else {
            Ok(result as usize)
        }
    }

    /// Register a signal handler for window resize events
    pub fn register_winch_handler(
        context: *mut std::ffi::c_void,
        callback: extern "C" fn(*mut std::ffi::c_void),
    ) -> Result<()> {
        let handler = SignalHandler { context, callback };

        let mut handlers = SIGNAL_HANDLERS.lock().unwrap_or_else(|e| e.into_inner());
        handlers.push(handler);

        Ok(())
    }

    /// Start a bounded raw input stream with at most 64 chunks of 4096 bytes.
    /// Dropping the receiver or final cloned terminal owner joins the reader.
    pub fn spawn_input_thread(&self) -> Result<InputReceiver<Vec<u8>>> {
        self.spawn_input(|bytes| vec![bytes.to_vec()])
    }

    pub(super) fn spawn_input<T, F>(&self, parse: F) -> Result<InputReceiver<T>>
    where
        T: Send + 'static,
        F: FnMut(&[u8]) -> Vec<T> + Send + 'static,
    {
        // A separate open description keeps nonblocking flags private to the
        // worker. A duplicated fd would still share flags with synchronous IO.
        let fd = unsafe {
            libc::open(
                c"/dev/tty".as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NONBLOCK | libc::O_NOCTTY,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let descriptor = unsafe { OwnedFd::from_raw_fd(fd) };
        // Reject a caller whose controlling terminal has changed since init.
        // A session can have only one controlling terminal; compare both owned
        // descriptors before starting a reader for this owner.
        let session = unsafe { libc::tcgetsid(self.fd) };
        if session < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let reader_session = unsafe { libc::tcgetsid(descriptor.as_raw_fd()) };
        if reader_session < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if session != reader_session {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Input reader and terminal owner belong to different controlling sessions",
            )
            .into());
        }
        let receiver = InputWorker::spawn(descriptor, parse)?;
        let mut workers = self.state.workers.lock().unwrap_or_else(|e| e.into_inner());
        workers.retain(|worker| worker.strong_count() != 0);
        workers.push(InputWorker::registration(&receiver));
        Ok(receiver)
    }
}

impl Clone for UnixTty {
    fn clone(&self) -> Self {
        Self {
            fd: self.fd,
            state: Arc::clone(&self.state),
        }
    }
}

impl AsRawFd for UnixTty {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

/// Install a SIGWINCH self-pipe and dispatch callbacks outside signal context.
fn install_signal_handlers() -> Result<()> {
    use std::io::Read;

    let mut lifecycle = SIGNAL_LIFECYCLE.lock().unwrap_or_else(|e| e.into_inner());
    if lifecycle.owners != 0 {
        lifecycle.owners += 1;
        return Ok(());
    }

    let (mut reader, writer) = UnixStream::pair()?;
    let stop = reader.try_clone()?;
    let action = signal_hook::low_level::pipe::register(libc::SIGWINCH, writer)?;
    let worker = match std::thread::Builder::new()
        .name("reactive-tui-sigwinch".into())
        .spawn(move || {
            let mut byte = [0_u8; 64];
            loop {
                match reader.read(&mut byte) {
                    Ok(0) => break,
                    Ok(_) => {
                        let handlers = SIGNAL_HANDLERS
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .clone();
                        for handler in handlers {
                            (handler.callback)(handler.context);
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        }) {
        Ok(worker) => worker,
        Err(error) => {
            signal_hook::low_level::unregister(action);
            return Err(error.into());
        }
    };
    lifecycle.owners = 1;
    lifecycle.action = Some(action);
    lifecycle.stop = Some(stop);
    lifecycle.worker = Some(worker);
    Ok(())
}

fn release_signal_handler() {
    let resources = {
        let mut lifecycle = SIGNAL_LIFECYCLE.lock().unwrap_or_else(|e| e.into_inner());
        if lifecycle.owners == 0 {
            return;
        }
        lifecycle.owners -= 1;
        if lifecycle.owners != 0 {
            return;
        }
        (
            lifecycle.action.take(),
            lifecycle.stop.take(),
            lifecycle.worker.take(),
        )
    };
    stop_signal_handler(resources);
}

fn stop_signal_handler(
    (action, stop, worker): (
        Option<signal_hook::SigId>,
        Option<UnixStream>,
        Option<JoinHandle<()>>,
    ),
) {
    if let Some(action) = action {
        signal_hook::low_level::unregister(action);
    }
    if let Some(stop) = stop {
        let _ = stop.shutdown(std::net::Shutdown::Both);
    }
    if let Some(worker) = worker {
        let _ = worker.join();
    }
    SIGNAL_HANDLERS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clear();
}

/// Stop the library's resize dispatcher and release registered callbacks.
pub fn reset_signal_handlers() {
    let resources = {
        let mut lifecycle = SIGNAL_LIFECYCLE.lock().unwrap_or_else(|e| e.into_inner());
        lifecycle.owners = 0;
        (
            lifecycle.action.take(),
            lifecycle.stop.take(),
            lifecycle.worker.take(),
        )
    };
    stop_signal_handler(resources);
}

/// Panic handler to restore terminal state
pub fn install_panic_handler() {
    std::panic::set_hook(Box::new(|_| {
        // Thread-safe access to global TTY
        if let Some(global_tty) = GLOBAL_TTY.get() {
            if let Ok(guard) = global_tty.lock() {
                if let Some(tty) = guard.as_ref().and_then(Weak::upgrade) {
                    let _ = tty.restore();
                }
            }
        }
        reset_signal_handlers();
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::atomic::AtomicUsize;

    static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
    static PRIOR_COUNT: AtomicUsize = AtomicUsize::new(0);
    static CALLBACK_LOCK: Mutex<()> = Mutex::new(());

    extern "C" fn counted_callback(_context: *mut std::ffi::c_void) {
        let _guard = CALLBACK_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    extern "C" fn prior_handler(_signal: libc::c_int) {
        PRIOR_COUNT.fetch_add(1, Ordering::SeqCst);
    }

    fn wait_for(counter: &AtomicUsize, expected: usize) {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while counter.load(Ordering::SeqCst) < expected && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(counter.load(Ordering::SeqCst), expected);
    }

    #[test]
    #[serial]
    fn api019_sigwinch_dispatches_outside_signal_context_and_preserves_prior_handler() {
        reset_signal_handlers();
        CALLBACK_COUNT.store(0, Ordering::SeqCst);
        PRIOR_COUNT.store(0, Ordering::SeqCst);

        let mut original = std::mem::MaybeUninit::<libc::sigaction>::uninit();
        let mut prior: libc::sigaction = unsafe { std::mem::zeroed() };
        prior.sa_sigaction = prior_handler as *const () as usize;
        unsafe {
            libc::sigemptyset(&mut prior.sa_mask);
            assert_eq!(
                libc::sigaction(libc::SIGWINCH, &prior, original.as_mut_ptr()),
                0
            );
        }

        install_signal_handlers().unwrap();
        UnixTty::register_winch_handler(std::ptr::null_mut(), counted_callback).unwrap();
        let held = CALLBACK_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe { libc::raise(libc::SIGWINCH) };
        wait_for(&PRIOR_COUNT, 1);
        assert_eq!(CALLBACK_COUNT.load(Ordering::SeqCst), 0);
        drop(held);
        wait_for(&CALLBACK_COUNT, 1);

        release_signal_handler();
        unsafe { libc::raise(libc::SIGWINCH) };
        wait_for(&PRIOR_COUNT, 2);
        std::thread::sleep(Duration::from_millis(20));
        assert_eq!(CALLBACK_COUNT.load(Ordering::SeqCst), 1);

        let original = unsafe { original.assume_init() };
        unsafe { libc::sigaction(libc::SIGWINCH, &original, std::ptr::null_mut()) };
    }

    #[test]
    fn test_tty_init() {
        // Skip test if no controlling terminal is available
        let tty = match UnixTty::init() {
            Ok(tty) => tty,
            Err(_) => {
                eprintln!("Skipping test_tty_init: No controlling terminal available");
                return;
            }
        };
        // Test terminal size - skip if not available
        let (width, height) = match tty.size() {
            Ok(size) => size,
            Err(_) => {
                eprintln!("Skipping terminal size test: Terminal operations not available");
                return;
            }
        };
        assert!(width > 0);
        assert!(height > 0);
    }

    #[test]
    fn test_write_read() {
        // Skip test if no controlling terminal is available
        let tty = match UnixTty::init() {
            Ok(tty) => tty,
            Err(_) => {
                eprintln!("Skipping test_write_read: No controlling terminal available");
                return;
            }
        };

        // Write a simple escape sequence - skip if not available
        let written = match tty.write(b"\x1b[6n") {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Skipping write test: Terminal write operations not available");
                return;
            }
        };
        assert_eq!(written, 4);

        // Try to read response (cursor position report) - skip if not available
        let mut buf = [0u8; 32];
        let timeout = Duration::from_millis(100);
        let _read = match tty.read(&mut buf, Some(timeout)) {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Skipping read test: Terminal read operations not available");
                return;
            }
        };
        // Note: This might timeout if terminal doesn't support cursor position report
    }
}
