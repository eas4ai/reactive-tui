//! Main terminal emulator implementation

use super::{
    AnsiParser, PseudoTerminal, TerminalConfig, TerminalError, TerminalEvent, TerminalResult,
    VirtualScreen,
};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

pub struct Terminal {
    config: TerminalConfig,
    screen: VirtualScreen,
    pty: PseudoTerminal,
    parser: AnsiParser,
    event_receiver: Option<mpsc::Receiver<TerminalEvent>>,
    running: bool,
    last_activity: Instant,
}

impl Terminal {
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
        }
    }

    pub fn start(&mut self) -> TerminalResult<()> {
        self.pty.spawn(&self.config)?;
        self.setup_event_loop()?;
        self.running = true;
        Ok(())
    }

    fn setup_event_loop(&mut self) -> TerminalResult<()> {
        let (event_tx, event_rx) = mpsc::channel();
        self.event_receiver = Some(event_rx);

        let _pty_output_tx = event_tx.clone();
        thread::spawn(move || {
            loop {
                // This would be implemented with actual PTY reading
                thread::sleep(Duration::from_millis(10));
                // Placeholder - in real implementation, read from PTY and send events
            }
        });

        Ok(())
    }

    pub fn write_input(&mut self, data: &[u8]) -> TerminalResult<()> {
        self.pty.write_input(data)?;
        self.last_activity = Instant::now();
        Ok(())
    }

    pub fn write_string(&mut self, s: &str) -> TerminalResult<()> {
        self.write_input(s.as_bytes())
    }

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

    pub fn poll_events(&mut self) -> Vec<TerminalEvent> {
        let mut events = Vec::new();

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

    pub fn process_output(&mut self, data: &[u8]) {
        let events = self.parser.parse_bytes(data);
        for event in events {
            self.screen.process_event(event);
        }
        self.last_activity = Instant::now();
    }

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

    pub fn size(&self) -> (u16, u16) {
        self.config.size
    }

    pub fn cursor_position(&self) -> (u16, u16) {
        self.screen.cursor_position()
    }

    pub fn cursor_visible(&self) -> bool {
        self.screen.cursor_visible()
    }

    pub fn cursor_shape(&self) -> super::cursor::CursorShape {
        self.screen.cursor_shape()
    }

    pub fn title(&self) -> &str {
        self.screen.title()
    }

    pub fn working_directory(&self) -> Option<&str> {
        self.screen.working_directory()
    }

    pub fn cell_at(&self, col: u16, row: u16) -> Option<&super::TerminalCell> {
        self.screen.cell_at(col, row)
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn last_activity(&self) -> Instant {
        self.last_activity
    }

    pub fn stop(&mut self) -> TerminalResult<()> {
        self.running = false;
        self.pty.kill()?;
        Ok(())
    }

    pub fn screen(&self) -> &VirtualScreen {
        &self.screen
    }

    pub fn screen_mut(&mut self) -> &mut VirtualScreen {
        &mut self.screen
    }

    // Convenience methods for common operations
    pub fn clear_screen(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[2J\x1b[H")
    }

    pub fn clear_line(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[2K")
    }

    pub fn move_cursor(&mut self, col: u16, row: u16) -> TerminalResult<()> {
        let command = format!("\x1b[{};{}H", row + 1, col + 1);
        self.write_string(&command)
    }

    pub fn save_cursor(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[s")
    }

    pub fn restore_cursor(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x1b[u")
    }

    pub fn set_title(&mut self, title: &str) -> TerminalResult<()> {
        let command = format!("\x1b]0;{}\x07", title);
        self.write_string(&command)
    }

    pub fn bell(&mut self) -> TerminalResult<()> {
        self.write_input(b"\x07")
    }

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
