//! Terminal state tracking and restoration
//! 
//! Ensures terminal is properly restored even on panic or abnormal exit

use std::sync::Mutex;
use std::io::{self, Write};
use crossterm::execute;

/// Global terminal state for panic recovery
static GLOBAL_STATE: Mutex<Option<TerminalStateGuard>> = Mutex::new(None);

/// Terminal features that need to be tracked and restored
#[derive(Debug, Default, Clone)]
pub struct TerminalState {
    /// Original termios settings (Unix only)
    #[cfg(unix)]
    pub original_termios: Option<libc::termios>,
    
    /// Whether raw mode is enabled
    pub raw_mode: bool,
    
    /// Whether we're in alternate screen
    pub alternate_screen: bool,
    
    /// Whether mouse capture is enabled
    pub mouse_capture: bool,
    
    /// Whether bracketed paste is enabled
    pub bracketed_paste: bool,
    
    /// Whether synchronized output is enabled
    pub synchronized_output: bool,
    
    /// Whether cursor is hidden
    pub cursor_hidden: bool,
    
    /// Original cursor position (row, col)
    pub original_cursor_pos: Option<(u16, u16)>,
    
    /// Whether we modified terminal title
    pub title_changed: bool,
    
    /// Whether kitty keyboard protocol is enabled
    pub kitty_keyboard: bool,
    
    /// Whether we're using SGR mouse mode
    pub sgr_mouse: bool,
}

impl TerminalState {
    /// Create a new terminal state tracker
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Capture current terminal state before modifications
    #[cfg(unix)]
    pub fn capture_original(&mut self) -> io::Result<()> {
        use std::os::unix::io::AsRawFd;
        
        let stdin_fd = io::stdin().as_raw_fd();
        let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
        
        unsafe {
            if libc::tcgetattr(stdin_fd, termios.as_mut_ptr()) == 0 {
                self.original_termios = Some(termios.assume_init());
            } else {
                return Err(io::Error::last_os_error());
            }
        }
        
        // Try to get cursor position
        if let Ok(pos) = crossterm::cursor::position() {
            self.original_cursor_pos = Some(pos);
        }
        
        Ok(())
    }
    
    #[cfg(not(unix))]
    pub fn capture_original(&mut self) -> io::Result<()> {
        // On Windows, crossterm handles this internally
        if let Ok(pos) = crossterm::cursor::position() {
            self.original_cursor_pos = Some(pos);
        }
        Ok(())
    }
    
    /// Reset all terminal state to original
    pub fn reset_all(&self, writer: &mut impl Write) {
        // Order matters: reverse of how we enabled things
        
        // Reset keyboard protocols
        if self.kitty_keyboard {
            let _ = writer.write_all(b"\x1b[<u");  // Pop kitty keyboard
        }
        
        // Reset mouse modes
        if self.sgr_mouse {
            let _ = writer.write_all(b"\x1b[?1006l");  // Disable SGR mouse
        }
        if self.mouse_capture {
            let _ = execute!(writer, crossterm::event::DisableMouseCapture);
        }
        
        // Reset paste mode
        if self.bracketed_paste {
            let _ = writer.write_all(b"\x1b[?2004l");  // Disable bracketed paste
        }
        
        // Reset synchronized output
        if self.synchronized_output {
            let _ = writer.write_all(b"\x1b[?2026l");  // Disable synchronized output
        }
        
        // Show cursor if hidden
        if self.cursor_hidden {
            let _ = execute!(writer, crossterm::cursor::Show);
        }
        
        // Reset SGR attributes
        let _ = writer.write_all(b"\x1b[0m");  // SGR reset
        
        // Exit alternate screen
        if self.alternate_screen {
            // Clear screen first
            let _ = execute!(writer, 
                crossterm::cursor::MoveTo(0, 0),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            );
            let _ = execute!(writer, crossterm::terminal::LeaveAlternateScreen);
        } else if let Some((_, row)) = self.original_cursor_pos {
            // If not in alt screen, move cursor to bottom and clear below
            let _ = writer.write_all(b"\r");  // Carriage return
            for _ in 0..row {
                let _ = writer.write_all(b"\x1b[A");  // Move up
            }
            let _ = writer.write_all(b"\x1b[J");  // Clear below cursor
        }
        
        // Restore raw mode
        if self.raw_mode {
            let _ = crossterm::terminal::disable_raw_mode();
        }
        
        // Restore original termios on Unix
        #[cfg(unix)]
        if let Some(termios) = &self.original_termios {
            use std::os::unix::io::AsRawFd;
            let stdin_fd = io::stdin().as_raw_fd();
            unsafe {
                let _ = libc::tcsetattr(stdin_fd, libc::TCSAFLUSH, termios);
            }
        }
        
        // Force flush
        let _ = writer.flush();
    }
}

/// RAII guard for terminal state restoration
pub struct TerminalStateGuard {
    state: TerminalState,
    armed: bool,
}

impl TerminalStateGuard {
    /// Create a new guard that will restore terminal on drop
    pub fn new(state: TerminalState) -> Self {
        Self {
            state,
            armed: true,
        }
    }
    
    /// Register this guard globally for panic handling
    pub fn register_global(self) -> Self {
        if let Ok(mut global) = GLOBAL_STATE.lock() {
            *global = Some(self.clone());
        }
        self
    }
    
    /// Disarm the guard (prevent restoration on drop)
    pub fn disarm(&mut self) {
        self.armed = false;
    }
    
    /// Get a reference to the state
    pub fn state(&self) -> &TerminalState {
        &self.state
    }
    
    /// Get a mutable reference to the state
    pub fn state_mut(&mut self) -> &mut TerminalState {
        &mut self.state
    }
    
    /// Manually trigger restoration
    pub fn restore(&self) {
        let mut stdout = io::stdout();
        self.state.reset_all(&mut stdout);
    }
}

impl Clone for TerminalStateGuard {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            armed: false,  // Only the original guard is armed
        }
    }
}

impl Drop for TerminalStateGuard {
    fn drop(&mut self) {
        if self.armed {
            self.restore();
        }
    }
}

/// Panic handler that restores terminal state
pub fn panic_handler(info: &std::panic::PanicInfo) {
    // Try to restore terminal state
    if let Ok(global) = GLOBAL_STATE.lock() {
        if let Some(ref guard) = *global {
            guard.restore();
        }
    }
    
    // Call the default panic handler
    if let Some(location) = info.location() {
        eprintln!("\nPanic occurred at {}:{}:{}", 
            location.file(), 
            location.line(), 
            location.column()
        );
    }
    if let Some(msg) = info.payload().downcast_ref::<&str>() {
        eprintln!("Message: {}", msg);
    } else if let Some(msg) = info.payload().downcast_ref::<String>() {
        eprintln!("Message: {}", msg);
    }
}

/// Install the panic handler
pub fn install_panic_handler() {
    std::panic::set_hook(Box::new(panic_handler));
}

/// Uninstall our panic handler and restore the default
pub fn uninstall_panic_handler() {
    let _ = std::panic::take_hook();
}