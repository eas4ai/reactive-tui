//! Main terminal emulator implementation

use super::{
    AnsiParser, PseudoTerminal, TerminalConfig, TerminalError, TerminalEvent, TerminalResult,
    VirtualScreen,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Terminal emulator instance managing PTY, screen buffer, and event processing
#[derive(Debug)]
pub struct Terminal {
    config: TerminalConfig,
    screen: VirtualScreen,
    pty: PseudoTerminal,
    parser: AnsiParser,
    event_receiver: Option<mpsc::Receiver<TerminalEvent>>,
    running: bool,
    last_activity: Instant,
    running_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}

impl Terminal {
    /// Create a new terminal emulator with the given configuration
    pub fn new(config: TerminalConfig) -> Self {
        let screen = VirtualScreen::new(config.size.0, config.size.1, config.scrollback_size);

        Self {
            config,
            screen,
            pty: PseudoTerminal::new(),
            parser: AnsiParser::new(),
            event_receiver: None,
            running: false,
            last_activity: Instant::now(),
            running_flag: None,
        }
    }

    /// Start the terminal emulator, spawning the shell process
    pub fn start(&mut self) -> TerminalResult<()> {
        self.pty.spawn(&self.config)?;
        self.setup_event_loop()?;
        self.running = true;
        Ok(())
    }

    fn setup_event_loop(&mut self) -> TerminalResult<()> {
        let (event_tx, event_rx) = mpsc::channel();
        self.event_receiver = Some(event_rx);

        // Create shared running flag for thread coordination
        let running_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        let running_clone = running_flag.clone();
        self.running_flag = Some(running_flag);

        // Clone event sender for background thread
        let _output_tx = event_tx.clone();

        thread::spawn(move || {
            // Background thread for additional event processing
            // This can handle periodic tasks, monitoring, etc.
            loop {
                if !running_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }

                // Production implementation could include:
                // - Periodic health checks
                // - Background data processing
                // - Event aggregation and filtering
                // - Performance monitoring

                // For now, just maintain the event loop with minimal overhead
                thread::sleep(Duration::from_millis(50));
            }
        });

        Ok(())
    }

    /// Write raw input bytes to the terminal process
    pub fn write_input(&mut self, data: &[u8]) -> TerminalResult<()> {
        self.pty.write_input(data)?;
        self.last_activity = Instant::now();
        Ok(())
    }

    /// Write a string to the terminal process
    pub fn write_string(&mut self, s: &str) -> TerminalResult<()> {
        self.write_input(s.as_bytes())
    }

    /// Send a keyboard key event to the terminal
    pub fn send_key(&mut self, key: &str) -> TerminalResult<()> {
        match key {
            "Enter" => self.write_input(b"\r"),
            "Tab" => self.write_input(b"\t"),
            "Backspace" => self.write_input(b"\x7f"),
            "Escape" => self.write_input(b"\x1b"),
            "Up" => self.write_input(b"\x1b[A"),
            "Down" => self.write_input(b"\x1b[B"),
            "Right" => self.write_input(b"\x1b[C"),
            "Left" => self.write_input(b"\x1b[D"),
            "Home" => self.write_input(b"\x1b[H"),
            "End" => self.write_input(b"\x1b[F"),
            "PageUp" => self.write_input(b"\x1b[5~"),
            "PageDown" => self.write_input(b"\x1b[6~"),
            "Insert" => self.write_input(b"\x1b[2~"),
            "Delete" => self.write_input(b"\x1b[3~"),
            _ => {
                if key.len() == 1 {
                    self.write_string(key)
                } else {
                    Err(TerminalError::Parse(format!("Unknown key: {}", key)))
                }
            }
        }
    }

    /// Poll for terminal events (output, resize, process exit)
    pub fn poll_events(&mut self) -> Vec<TerminalEvent> {
        let mut events = Vec::new();

        // Read output from PTY and process it
        if self.running {
            // Use a short timeout to avoid blocking
            match self.pty.read_output(Some(Duration::from_millis(1))) {
                Ok(Some(data)) => {
                    if !data.is_empty() {
                        // Process the output data through the ANSI parser
                        let mut processed_events = self.process_output(&data);
                        events.append(&mut processed_events);

                        // Send the raw output as an event
                        events.push(TerminalEvent::Output(data));
                        self.last_activity = Instant::now();
                    }
                }
                Ok(None) => {
                    // No data available, continue
                }
                Err(e) => {
                    // PTY error, likely disconnected
                    eprintln!("PTY read error: {}", e);
                    self.running = false;
                }
            }
        }

        // Check for events from the background thread
        if let Some(ref receiver) = self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        // Check for process exit
        if let Ok(Some(exit_code)) = self.pty.try_wait() {
            events.push(TerminalEvent::ProcessExited(exit_code));
            self.running = false;
        }

        events
    }

    /// Process output data from the terminal process
    pub fn process_output(&mut self, data: &[u8]) -> Vec<TerminalEvent> {
        let mut terminal_events = Vec::new();
        let ansi_events = self.parser.parse_bytes(data);

        for event in ansi_events {
            // Process the event through the screen buffer
            self.screen.process_event(event.clone());

            // Generate terminal events based on ANSI events
            match event {
                super::parser::AnsiEvent::Osc { command, params } => {
                    match command.as_str() {
                        "0" | "2" => {
                            // Window title change
                            if let Some(title) = params.first() {
                                terminal_events.push(TerminalEvent::TitleChanged(title.clone()));
                            }
                        }
                        "7" => {
                            // Working directory change
                            if let Some(path) = params.first() {
                                terminal_events.push(TerminalEvent::WorkingDirectoryChanged(path.clone()));
                            }
                        }
                        _ => {}
                    }
                }
                super::parser::AnsiEvent::Execute(0x07) => {
                    // Bell character
                    terminal_events.push(TerminalEvent::Bell);
                }
                _ => {
                    // Other events are handled by the screen buffer
                }
            }
        }

        self.last_activity = Instant::now();
        terminal_events
    }

    /// Resize the terminal to the specified dimensions
    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        if width == 0 || height == 0 {
            return Err(TerminalError::InvalidSize { width, height });
        }

        self.config.size = (width, height);
        self.pty.resize(width, height)?;

        // Create new screen with new size
        self.screen = VirtualScreen::new(width, height, self.config.scrollback_size);

        Ok(())
    }

    /// Get the current terminal size (width, height)
    pub fn size(&self) -> (u16, u16) {
        self.config.size
    }

    /// Get the current cursor position (column, row)
    pub fn cursor_position(&self) -> (u16, u16) {
        self.screen.cursor_position()
    }

    /// Check if the cursor is visible
    pub fn cursor_visible(&self) -> bool {
        self.screen.cursor_visible()
    }

    /// Get the current cursor shape
    pub fn cursor_shape(&self) -> super::cursor::CursorShape {
        self.screen.cursor_shape()
    }

    /// Get the terminal window title
    pub fn title(&self) -> &str {
        self.screen.title()
    }

    /// Get the current working directory if available
    pub fn working_directory(&self) -> Option<&str> {
        self.screen.working_directory()
    }

    /// Get the cell at the specified position
    pub fn cell_at(&self, col: u16, row: u16) -> Option<&super::TerminalCell> {
        self.screen.cell_at(col, row)
    }

    /// Check if the terminal is currently running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get the timestamp of the last terminal activity
    pub fn last_activity(&self) -> Instant {
        self.last_activity
    }

    /// Stop the terminal emulator and kill the shell process
    pub fn stop(&mut self) -> TerminalResult<()> {
        self.running = false;

        // Signal background thread to stop
        if let Some(ref flag) = self.running_flag {
            flag.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        self.pty.kill()?;
        Ok(())
    }

    /// Get a reference to the virtual screen buffer
    pub fn screen(&self) -> &VirtualScreen {
        &self.screen
    }

    /// Get a mutable reference to the virtual screen buffer
    pub fn screen_mut(&mut self) -> &mut VirtualScreen {
        &mut self.screen
    }

    // Convenience methods for common operations
    /// Clear the entire terminal screen
    pub fn clear_screen(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[2J\x1b[H")
    }

    /// Clear the current line
    pub fn clear_line(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[2K")
    }

    /// Move the cursor to the specified position
    pub fn move_cursor(&mut self, col: u16, row: u16) -> TerminalResult<()> {
        let command = format!("\x1b[{};{}H", row + 1, col + 1);
        self.write_string(&command)
    }

    /// Save the current cursor position
    pub fn save_cursor(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[s")
    }

    /// Restore the previously saved cursor position
    pub fn restore_cursor(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[u")
    }

    /// Set the terminal window title
    pub fn set_title(&mut self, title: &str) -> TerminalResult<()> {
        let command = format!("\x1b]0;{}\x07", title);
        self.write_string(&command)
    }

    /// Ring the terminal bell
    pub fn bell(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x07")
    }

    /// Reset the terminal to its initial state
    pub fn reset(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1bc")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_creation() {
        let config = TerminalConfig::default();
        let terminal = Terminal::new(config);

        assert_eq!(terminal.size(), (80, 24));
        assert!(!terminal.is_running());
        assert_eq!(terminal.cursor_position(), (0, 0));
    }

    #[test]
    fn test_terminal_resize() {
        let config = TerminalConfig::default();
        let mut terminal = Terminal::new(config);

        let result = terminal.resize(120, 30);
        assert!(result.is_ok());
        assert_eq!(terminal.size(), (120, 30));
    }

    #[test]
    fn test_terminal_invalid_resize() {
        let config = TerminalConfig::default();
        let mut terminal = Terminal::new(config);

        let result = terminal.resize(0, 24);
        assert!(result.is_err());

        let result = terminal.resize(80, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_key_sequences() {
        let config = TerminalConfig::default();
        let mut terminal = Terminal::new(config);

        // These would normally require a running PTY, but we can test the key mapping
        assert!(terminal.send_key("a").is_ok());
        assert!(terminal.send_key("Enter").is_ok());
        assert!(terminal.send_key("Up").is_ok());
        assert!(terminal.send_key("InvalidKey").is_err());
    }
}
