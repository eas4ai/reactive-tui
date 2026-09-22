//! Terminal management and capability detection

use std::io::{stdout, Stdout, Write};
use std::process::Command;

use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, BeginSynchronizedUpdate, Clear, ClearType,
        EndSynchronizedUpdate, EnterAlternateScreen, LeaveAlternateScreen,
    },
};

use crate::core::capabilities::{TerminalCapabilities, TerminalQuery};
use crate::error::{ReactiveError, Result};
use crate::widgets::display::image::ImageCapabilities;

/// Statistics for terminal write operations
#[derive(Debug, Default, Clone)]
pub struct TerminalWriteStats {
    /// Total bytes written to terminal
    pub total_bytes: usize,
    /// Total number of write operations
    pub total_writes: usize,
    /// Buffer utilization percentage (0.0 to 1.0)
    pub buffer_utilization: f32,
}

/// Terminal abstraction that handles initialization and cleanup
pub struct Terminal {
    stdout: Stdout,
    /// Original terminal size for restoration
    original_size: Option<(u16, u16)>,
    /// Whether we're in raw mode
    raw_mode: bool,
    /// Whether we're in alternate screen
    alternate_screen: bool,
    /// Detected terminal capabilities
    capabilities: TerminalCapabilities,
    /// Write buffer for high-performance mode
    write_buffer: Vec<u8>,
    /// Whether buffered mode is enabled
    buffered_mode: bool,
    /// Write statistics
    write_stats: TerminalWriteStats,
}

impl Terminal {
    /// Create a new terminal instance
    pub fn new() -> Result<Self> {
        let stdout = stdout();

        // Detect capabilities using our new query system
        let capabilities = TerminalQuery::detect_from_env();

        Ok(Terminal {
            stdout,
            original_size: None,
            raw_mode: false,
            alternate_screen: false,
            capabilities,
            write_buffer: Vec::with_capacity(65536), // 64KB default buffer
            buffered_mode: false,
            write_stats: TerminalWriteStats::default(),
        })
    }

    /// Gate startup on modern terminal assumptions.
    pub fn capability_gate(&mut self) -> Result<()> {
        // Allow bypassing capability checks via environment variable
        if std::env::var("REACTIVE_TUI_FORCE_ENABLE").is_ok() {
            // Force enable 256 color support when bypassing
            self.capabilities.color_depth = crate::core::capabilities::ColorDepth::Colors256;
            return Ok(());
        }

        // Warn about limited color support but don't fail
        if !self.capabilities.color_depth.supports(256) {
            log::warn!(
                "Terminal may not support 256 colors; some features may look degraded. \
                 Set REACTIVE_TUI_FORCE_ENABLE=1 to bypass this check."
            );
            // Continue anyway - let it degrade gracefully
        }
        Ok(())
    }

    /// Enter modern terminal mode (raw mode + alternate screen)
    pub fn enter_modern_mode(&mut self) -> Result<()> {
        enable_raw_mode()
            .map_err(|e| ReactiveError::terminal(format!("Failed to enable raw mode: {}", e)))?;
        self.raw_mode = true;

        execute!(self.stdout, EnterAlternateScreen).map_err(|e| {
            ReactiveError::terminal(format!("Failed to enter alternate screen: {}", e))
        })?;
        self.alternate_screen = true;

        execute!(self.stdout, EnableBracketedPaste).map_err(|e| {
            ReactiveError::terminal(format!("Failed to enable bracketed paste: {}", e))
        })?;

        execute!(self.stdout, EnableMouseCapture).map_err(|e| {
            ReactiveError::terminal(format!("Failed to enable mouse capture: {}", e))
        })?;

        Ok(())
    }

    /// Exit modern terminal mode
    pub fn exit_modern_mode(&mut self) -> Result<()> {
        let _ = execute!(self.stdout, DisableMouseCapture);
        let _ = execute!(self.stdout, DisableBracketedPaste);

        if self.alternate_screen {
            let _ = execute!(self.stdout, LeaveAlternateScreen);
            self.alternate_screen = false;
        }

        if self.raw_mode {
            let _ = disable_raw_mode();
            self.raw_mode = false;
        }

        Ok(())
    }

    /// Begin synchronized update
    pub fn begin_sync(&mut self) -> Result<()> {
        execute!(self.stdout, BeginSynchronizedUpdate)
            .map_err(|e| ReactiveError::terminal(format!("Failed to begin sync: {}", e)))
    }

    /// End synchronized update
    pub fn end_sync(&mut self) -> Result<()> {
        execute!(self.stdout, EndSynchronizedUpdate)
            .map_err(|e| ReactiveError::terminal(format!("Failed to end sync: {}", e)))
    }

    /// Enable high-performance buffered write mode
    pub fn enable_buffered_mode(&mut self) -> Result<()> {
        self.buffered_mode = true;
        self.write_buffer.clear();
        Ok(())
    }

    /// Disable buffered write mode
    pub fn disable_buffered_mode(&mut self) -> Result<()> {
        if self.buffered_mode {
            self.flush_buffered()?;
        }
        self.buffered_mode = false;
        Ok(())
    }

    /// Write all data to buffer or directly
    pub fn write_all_buffered(&mut self, data: &[u8]) -> Result<()> {
        if self.buffered_mode {
            self.write_buffer.extend_from_slice(data);
            self.write_stats.total_bytes += data.len();
            self.write_stats.buffer_utilization =
                (self.write_buffer.len() as f32) / (self.write_buffer.capacity() as f32);
            Ok(())
        } else {
            self.stdout
                .write_all(data)
                .map_err(|e| ReactiveError::terminal(format!("Failed to write: {}", e)))?;
            self.write_stats.total_bytes += data.len();
            self.write_stats.total_writes += 1;
            Ok(())
        }
    }

    /// Flush buffered data to stdout
    pub fn flush_buffered(&mut self) -> Result<()> {
        if !self.write_buffer.is_empty() {
            self.stdout
                .write_all(&self.write_buffer)
                .map_err(|e| ReactiveError::terminal(format!("Failed to flush buffer: {}", e)))?;
            self.stdout
                .flush()
                .map_err(|e| ReactiveError::terminal(format!("Failed to flush: {}", e)))?;
            self.write_stats.total_writes += 1;
            self.write_buffer.clear();
        }
        Ok(())
    }

    /// Get write statistics
    pub fn write_stats(&self) -> &TerminalWriteStats {
        &self.write_stats
    }

    /// A failed frame is retried in full by its owner. Do not prepend bytes
    /// retained after a partial buffered write to that replacement frame.
    pub(crate) fn discard_buffered(&mut self) {
        self.write_buffer.clear();
        self.write_stats.buffer_utilization = 0.0;
    }

    /// Reset write statistics
    pub fn reset_write_stats(&mut self) {
        self.write_stats = TerminalWriteStats::default();
    }

    /// Static write method for non-buffered writes
    #[cfg(not(test))]
    pub fn write_all(data: &[u8]) -> Result<()> {
        stdout()
            .write_all(data)
            .map_err(|e| ReactiveError::terminal(format!("Failed to write: {}", e)))
    }

    /// Static write method for test mode
    #[cfg(test)]
    pub fn write_all(data: &[u8]) -> Result<()> {
        test_io_capture::capture_write(data);
        Ok(())
    }

    /// Poll for terminal events
    pub fn poll_event(timeout_ms: Option<u64>) -> Result<Option<Event>> {
        let dur = std::time::Duration::from_millis(timeout_ms.unwrap_or(0));
        if !event::poll(dur)
            .map_err(|e| ReactiveError::terminal(format!("Failed to poll event: {}", e)))?
        {
            return Ok(None);
        }
        Ok(Some(event::read().map_err(|e| {
            ReactiveError::terminal(format!("Failed to read event: {}", e))
        })?))
    }

    /// Get detected capabilities
    pub fn capabilities(&self) -> &TerminalCapabilities {
        &self.capabilities
    }

    /// Initialize the terminal for TUI rendering
    pub fn init(&mut self) -> Result<()> {
        // Store original size
        let (cols, rows) = crossterm::terminal::size()
            .map_err(|e| ReactiveError::terminal(format!("Failed to get terminal size: {}", e)))?;
        self.original_size = Some((cols, rows));

        // Enter raw mode
        enable_raw_mode()
            .map_err(|e| ReactiveError::terminal(format!("Failed to enable raw mode: {}", e)))?;
        self.raw_mode = true;

        // Enter alternate screen
        execute!(self.stdout, EnterAlternateScreen).map_err(|e| {
            ReactiveError::terminal(format!("Failed to enter alternate screen: {}", e))
        })?;
        self.alternate_screen = true;

        // Enable mouse capture
        execute!(self.stdout, EnableMouseCapture).map_err(|e| {
            ReactiveError::terminal(format!("Failed to enable mouse capture: {}", e))
        })?;

        // Hide cursor
        execute!(self.stdout, cursor::Hide)
            .map_err(|e| ReactiveError::terminal(format!("Failed to hide cursor: {}", e)))?;

        // Clear screen
        execute!(self.stdout, Clear(ClearType::All))
            .map_err(|e| ReactiveError::terminal(format!("Failed to clear screen: {}", e)))?;

        Ok(())
    }

    /// Restore the terminal to its original state
    pub fn restore(&mut self) -> Result<()> {
        // Show cursor
        let _ = execute!(self.stdout, cursor::Show);

        // Disable mouse capture
        let _ = execute!(self.stdout, DisableMouseCapture);

        // Leave alternate screen if we entered it
        if self.alternate_screen {
            let _ = execute!(self.stdout, LeaveAlternateScreen);
            self.alternate_screen = false;
        }

        // Disable raw mode if we enabled it
        if self.raw_mode {
            let _ = disable_raw_mode();
            self.raw_mode = false;
        }

        Ok(())
    }

    /// Flush the stdout buffer
    pub fn flush(&mut self) -> Result<()> {
        self.stdout
            .flush()
            .map_err(|e| ReactiveError::terminal(format!("Failed to flush stdout: {}", e)))
    }

    /// Get the current terminal size
    pub fn size(&self) -> Result<(u16, u16)> {
        crossterm::terminal::size()
            .map_err(|e| ReactiveError::terminal(format!("Failed to get terminal size: {}", e)))
    }

    /// Get the current terminal size (static version)
    pub fn get_size() -> Result<(u16, u16)> {
        crossterm::terminal::size()
            .map_err(|e| ReactiveError::terminal(format!("Failed to get terminal size: {}", e)))
    }

    /// Check if the terminal meets minimum requirements
    pub fn check_requirements() -> Result<()> {
        // Using our new capabilities detection
        let caps = TerminalQuery::detect_from_env();

        // Check for color support
        if !caps.color_depth.supports(256) {
            return Err(ReactiveError::terminal(
                "Terminal must support at least 256 colors",
            ));
        }

        // Check terminal size
        let (width, height) = Self::get_size()?;
        if width < 80 || height < 24 {
            return Err(ReactiveError::terminal(format!(
                "Terminal too small: {}x{} (minimum 80x24)",
                width, height
            )));
        }

        // Check for modern terminal features
        let term = std::env::var("TERM").unwrap_or_default();
        let is_modern = term.contains("256color")
            || term.contains("truecolor")
            || term.contains("kitty")
            || term.contains("alacritty")
            || term.contains("wezterm")
            || term.contains("iterm");

        if !is_modern {
            log::warn!(
                "Terminal '{term}' may have limited features; use a 24-bit-color terminal such as \
                 WezTerm, Kitty, Alacritty, or iTerm2. Detected color depth: {:?}; Unicode: {}; \
                 graphics: sixel={}, kitty={}, iterm2={}",
                caps.color_depth,
                caps.unicode,
                caps.sixel,
                caps.kitty_graphics,
                caps.iterm2_graphics
            );
        }

        Ok(())
    }

    /// Detect image capabilities using our new system
    pub fn detect_image_capabilities() -> ImageCapabilities {
        let caps = TerminalQuery::detect_from_env();

        ImageCapabilities {
            sixel: caps.sixel,
            kitty_graphics: caps.kitty_graphics,
            iterm2_inline: caps.iterm2_graphics,
            chafa_available: Self::has_external_tool("chafa"),
            viu_available: Self::has_external_tool("viu"),
        }
    }

    /// Check if an external tool is available
    fn has_external_tool(tool: &str) -> bool {
        let mut command = Command::new("which");
        command.arg(tool);
        crate::core::owned_process::run(
            command,
            None,
            crate::core::owned_process::Options {
                purpose: "terminal tool discovery",
                timeout: crate::core::owned_process::TERMINAL_HELPER_TIMEOUT,
                max_input: 0,
                max_output: 0,
                capture_output: false,
                allow_background_after_success: false,
            },
            || false,
        )
        .is_ok()
    }

    /// Set the terminal title
    pub fn set_title(&mut self, title: &str) -> Result<()> {
        if title.chars().any(char::is_control) {
            return Err(ReactiveError::terminal(
                "Terminal title must contain no control characters",
            ));
        }
        write!(self.stdout, "\x1b]2;{}\x07", title)
            .map_err(|e| ReactiveError::terminal(format!("Failed to set terminal title: {}", e)))?;
        self.flush()
    }

    /// Move cursor to position
    pub fn move_cursor_to(&mut self, x: u16, y: u16) -> Result<()> {
        execute!(self.stdout, cursor::MoveTo(x, y))
            .map_err(|e| ReactiveError::terminal(format!("Failed to move cursor: {}", e)))
    }

    /// Write raw bytes to terminal
    pub fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        self.stdout
            .write_all(data)
            .map_err(|e| ReactiveError::terminal(format!("Failed to write to terminal: {}", e)))
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // Best effort cleanup
        let _ = self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        // This test just verifies we can create a terminal instance
        // without actually initializing it (which would mess up the test terminal)
        let terminal = Terminal::new();
        assert!(terminal.is_ok());
    }

    #[test]
    fn test_image_capabilities_detection() {
        // Test that we can detect capabilities without terminal I/O
        let caps = Terminal::detect_image_capabilities();
        // Image capabilities must preserve the detected terminal protocol flags.
        let detected = TerminalQuery::detect_from_env();
        assert_eq!(caps.sixel, detected.sixel);
        assert_eq!(caps.kitty_graphics, detected.kitty_graphics);
        assert_eq!(caps.iterm2_inline, detected.iterm2_graphics);
    }

    #[test]
    fn test_size_detection() {
        // In a test environment this may fail; either way detection is stable
        let first = Terminal::get_size().ok();
        let second = Terminal::get_size().ok();
        assert_eq!(
            first, second,
            "terminal size detection is stable across calls"
        );
    }
}

#[cfg(test)]
/// Test utilities for capturing terminal output
pub mod test_io_capture {
    use std::sync::{Mutex, OnceLock};

    static TEST_OUT: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();
    fn out() -> &'static Mutex<Vec<u8>> {
        TEST_OUT.get_or_init(|| Mutex::new(Vec::new()))
    }

    /// Capture output written to the terminal for testing
    pub fn capture_write(buf: &[u8]) {
        if let Ok(mut guard) = out().lock() {
            guard.extend_from_slice(buf);
        } else {
            log::warn!("Terminal output lock poisoned during capture_write");
        }
    }

    /// Take all captured output and clear the buffer
    pub fn take_output() -> Vec<u8> {
        match out().lock() {
            Ok(mut guard) => std::mem::take(&mut *guard),
            Err(_) => {
                log::warn!("Terminal output lock poisoned during take_output");
                Vec::new()
            }
        }
    }
}
