//! Unix/POSIX TTY implementation for direct terminal access
//!
//! Native POSIX terminal interface with signal handling

use super::PlatformTty;
use crate::error::Result;
use std::os::unix::io::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::{Arc, Weak};
use std::time::Duration;

static SIGNAL_HANDLERS: Mutex<Vec<SignalHandler>> = Mutex::new(Vec::new());
static HANDLER_INSTALLED: AtomicBool = AtomicBool::new(false);
// FIX: Use OnceLock for thread-safe initialization instead of unsafe global
static GLOBAL_TTY: OnceLock<Mutex<Option<Weak<TtyState>>>> = OnceLock::new();

/// Signal handler for Unix systems
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
        });
        install_signal_handlers()?;
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

        let mut handlers = SIGNAL_HANDLERS.lock().unwrap();
        handlers.push(handler);

        Ok(())
    }

    /// Spawn a background thread for reading input
    pub fn spawn_input_thread(&self) -> Result<std::sync::mpsc::Receiver<Vec<u8>>> {
        use std::sync::mpsc;
        use std::thread;

        let (tx, rx) = mpsc::channel();
        let fd = self.fd;

        thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            // Set non-blocking mode for the thread
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            if flags >= 0 {
                unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) };
            }

            loop {
                // Use select to wait for input with timeout
                let mut fd_set = std::mem::MaybeUninit::<libc::fd_set>::uninit();
                unsafe {
                    libc::FD_ZERO(fd_set.as_mut_ptr());
                    libc::FD_SET(fd, fd_set.as_mut_ptr());
                }

                let mut timeout = libc::timeval {
                    tv_sec: 0,
                    tv_usec: 100_000, // 100ms timeout
                };

                let select_result = unsafe {
                    libc::select(
                        fd + 1,
                        fd_set.as_mut_ptr(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &mut timeout,
                    )
                };

                match select_result {
                    1 => {
                        // Data available
                        match unsafe {
                            libc::read(fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len())
                        } {
                            n if n > 0 => {
                                let data = buffer[..n as usize].to_vec();
                                if tx.send(data).is_err() {
                                    break; // Receiver dropped
                                }
                            }
                            0 => break, // EOF
                            _ => {
                                // Error reading
                                if std::io::Error::last_os_error().kind()
                                    != std::io::ErrorKind::WouldBlock
                                {
                                    break; // Real error
                                }
                            }
                        }
                    }
                    0 => {
                        // Timeout - continue loop
                        continue;
                    }
                    _ => {
                        // Error in select
                        break;
                    }
                }
            }

            // Restore blocking mode
            if flags >= 0 {
                unsafe { libc::fcntl(fd, libc::F_SETFL, flags) };
            }
        });

        Ok(rx)
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

/// Install SIGWINCH handler for window resize detection
fn install_signal_handlers() -> Result<()> {
    if HANDLER_INSTALLED.load(Ordering::Relaxed) {
        return Ok(());
    }

    let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
    unsafe {
        let action_ptr = action.as_mut_ptr();
        (*action_ptr).sa_sigaction = handle_winch as *const () as usize;

        #[cfg(target_os = "macos")]
        {
            (*action_ptr).sa_mask = 0;
        }
        #[cfg(not(target_os = "macos"))]
        {
            libc::sigemptyset(&mut (*action_ptr).sa_mask);
        }

        (*action_ptr).sa_flags = libc::SA_SIGINFO;

        let result = libc::sigaction(libc::SIGWINCH, action_ptr, std::ptr::null_mut());
        if result != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    HANDLER_INSTALLED.store(true, Ordering::Relaxed);
    Ok(())
}

/// Reset signal handlers to default
pub fn reset_signal_handlers() {
    if !HANDLER_INSTALLED.load(Ordering::Relaxed) {
        return;
    }

    let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
    unsafe {
        let action_ptr = action.as_mut_ptr();
        (*action_ptr).sa_sigaction = libc::SIG_DFL;

        #[cfg(target_os = "macos")]
        {
            (*action_ptr).sa_mask = 0;
        }
        #[cfg(not(target_os = "macos"))]
        {
            libc::sigemptyset(&mut (*action_ptr).sa_mask);
        }

        (*action_ptr).sa_flags = 0;

        let _ = libc::sigaction(libc::SIGWINCH, action_ptr, std::ptr::null_mut());
    }

    HANDLER_INSTALLED.store(false, Ordering::Relaxed);
}

/// SIGWINCH signal handler
extern "C" fn handle_winch(
    _sig: libc::c_int,
    _info: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    // Call all registered handlers
    if let Ok(handlers) = SIGNAL_HANDLERS.lock() {
        for handler in handlers.iter() {
            (handler.callback)(handler.context);
        }
    }
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
