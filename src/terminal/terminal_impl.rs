//! Main terminal emulator implementation

use super::{
    AnsiParser, PseudoTerminal, TerminalConfig, TerminalError, TerminalEvent, TerminalResult,
    VirtualScreen,
};
use std::time::{Duration, Instant};

/// Terminal emulator instance managing PTY, screen buffer, and event processing
#[derive(Debug)]
pub struct Terminal {
    config: TerminalConfig,
    screen: VirtualScreen,
    pty: PseudoTerminal,
    parser: AnsiParser,
    running: bool,
    last_activity: Instant,
    last_error: Option<String>,
    exit_reported: bool,
    revision: u64,
    resize_events: Vec<TerminalEvent>,
    resize_output_bytes: usize,
}

impl Terminal {
    /// Create an unstarted terminal emulator.
    ///
    /// # Panics
    /// Panics for zero dimensions or more than 262144 cells. Use [`Self::try_new`]
    /// when configuration comes from input that may be invalid.
    pub fn new(config: TerminalConfig) -> Self {
        Self::try_new(config).expect("invalid terminal dimensions")
    }

    /// Validate screen dimensions before allocating an unstarted terminal.
    pub fn try_new(config: TerminalConfig) -> TerminalResult<Self> {
        let screen = VirtualScreen::try_new(config.size.0, config.size.1, config.scrollback_size)?;

        Ok(Self {
            config,
            screen,
            pty: PseudoTerminal::new(),
            parser: AnsiParser::new(),
            running: false,
            last_activity: Instant::now(),
            last_error: None,
            exit_reported: false,
            revision: 0,
            resize_events: Vec::new(),
            resize_output_bytes: 0,
        })
    }

    /// Start the terminal emulator, spawning the shell process
    pub fn start(&mut self) -> TerminalResult<()> {
        if self.running {
            return Ok(());
        }
        self.pty.kill()?;
        self.pty.spawn(&self.config)?;
        self.last_error = None;
        self.resize_events.clear();
        self.resize_output_bytes = 0;
        self.exit_reported = false;
        self.running = true;
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
        use crate::event::types::{KeyCode, KeyEvent};
        let code = match key {
            "Enter" => KeyCode::Enter,
            "Tab" => KeyCode::Tab,
            "Backspace" => KeyCode::Backspace,
            "Escape" => KeyCode::Escape,
            "Up" => KeyCode::Up,
            "Down" => KeyCode::Down,
            "Right" => KeyCode::Right,
            "Left" => KeyCode::Left,
            "Home" => KeyCode::Home,
            "End" => KeyCode::End,
            "PageUp" => KeyCode::PageUp,
            "PageDown" => KeyCode::PageDown,
            "Insert" => KeyCode::Insert,
            "Delete" => KeyCode::Delete,
            _ => {
                let mut chars = key.chars();
                match (chars.next(), chars.next()) {
                    (Some(character), None) => KeyCode::Char(character),
                    _ => return Err(TerminalError::Parse(format!("Unknown key: {key}"))),
                }
            }
        };
        let bytes = super::keyboard::encode(
            &KeyEvent::new(code),
            self.screen.input_modes().application_cursor_keys,
        );
        self.write_input(&bytes)
    }

    /// Poll for terminal events (output, resize, process exit)
    pub fn poll_events(&mut self) -> Vec<TerminalEvent> {
        let mut events = std::mem::take(&mut self.resize_events);
        self.resize_output_bytes = 0;
        events.extend(self.read_pty_events(16));
        events
    }

    fn read_pty_events(&mut self, max_chunks: usize) -> Vec<TerminalEvent> {
        let mut events = Vec::new();
        // Drain bounded output even after the exit status becomes visible.
        for _ in 0..max_chunks {
            match self.pty.read_output(Some(Duration::ZERO)) {
                Ok(Some(data)) => {
                    events.extend(self.process_output(&data));
                    events.push(TerminalEvent::Output(data));
                }
                Ok(None) => break,
                Err(error) => {
                    self.last_error = Some(error.to_string());
                    self.running = false;
                    break;
                }
            }
        }
        if !self.exit_reported {
            match self.pty.try_wait() {
                Ok(Some(code)) => {
                    events.push(TerminalEvent::ProcessExited(code));
                    self.running = false;
                    self.exit_reported = true;
                    self.revision = self.revision.wrapping_add(1);
                }
                Ok(None) => {}
                Err(error) => {
                    self.last_error = Some(error.to_string());
                    self.running = false;
                }
            }
        }
        events
    }

    /// Latest PTY failure, distinct from a normal child exit code.
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Revision of screen, resize and process-exit updates.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// PID of the directly owned child.
    pub fn child_id(&self) -> Option<u32> {
        self.pty.child_id()
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
                                terminal_events
                                    .push(TerminalEvent::WorkingDirectoryChanged(path.clone()));
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
        self.revision = self.revision.wrapping_add(1);
        terminal_events
    }

    /// Consume queued output at the old dimensions, then resize the PTY and screen.
    /// Output events remain available from [`Self::poll_events`]. Repeated resizes
    /// without polling can return backpressure rather than retain more than 64 KiB.
    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        VirtualScreen::validate_size(width, height)?;

        const MAX_RESIZE_OUTPUT: usize = 64 * 1024;
        let chunks = (MAX_RESIZE_OUTPUT.saturating_sub(self.resize_output_bytes)
            / super::pty::OUTPUT_CHUNK_SIZE)
            .min(16);
        if chunks == 0 {
            return Err(TerminalError::Pty(
                "Poll terminal events before resizing again: the bounded output event buffer is full".into(),
            ));
        }
        let events = self.read_pty_events(chunks);
        self.resize_output_bytes += events
            .iter()
            .map(|event| match event {
                TerminalEvent::Output(bytes) => bytes.len(),
                _ => 0,
            })
            .sum::<usize>();
        self.resize_events.extend(events);

        self.pty.resize(width, height)?;
        self.screen.resize(width, height)?;
        self.config.size = (width, height);
        self.revision = self.revision.wrapping_add(1);

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
        self.pty.kill()
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
        self.apply_screen_control(b"\x1b[2J\x1b[H")
    }

    /// Clear the current line
    pub fn clear_line(&mut self) -> TerminalResult<()> {
        self.apply_screen_control(b"\x1b[2K")
    }

    /// Move the cursor to the specified position
    pub fn move_cursor(&mut self, col: u16, row: u16) -> TerminalResult<()> {
        let command = format!("\x1b[{};{}H", row.saturating_add(1), col.saturating_add(1));
        self.apply_screen_control(command.as_bytes())
    }

    /// Save the current cursor position
    pub fn save_cursor(&mut self) -> TerminalResult<()> {
        self.apply_screen_control(b"\x1b[s")
    }

    /// Restore the previously saved cursor position
    pub fn restore_cursor(&mut self) -> TerminalResult<()> {
        self.apply_screen_control(b"\x1b[u")
    }

    /// Set the terminal window title. Control characters and titles longer than
    /// 8190 UTF-8 bytes are rejected before changing the terminal.
    pub fn set_title(&mut self, title: &str) -> TerminalResult<()> {
        if title.len() > AnsiParser::MAX_STRING_BUFFER - 2 || title.chars().any(char::is_control) {
            return Err(TerminalError::Parse(
                "Terminal title must contain no control characters and fit in 8190 UTF-8 bytes"
                    .into(),
            ));
        }
        let command = format!("\x1b]0;{}\x07", title);
        self.apply_screen_control(command.as_bytes())
    }

    /// Ring the terminal bell
    pub fn bell(&mut self) -> TerminalResult<()> {
        self.apply_screen_control(b"\x07")
    }

    /// Reset the terminal to its initial state
    pub fn reset(&mut self) -> TerminalResult<()> {
        self.apply_screen_control(b"\x1bc")
    }
    fn apply_screen_control(&mut self, bytes: &[u8]) -> TerminalResult<()> {
        self.process_output(bytes);
        Ok(())
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

    #[cfg(unix)]
    #[test]
    fn repeated_resizes_apply_backpressure_until_output_events_are_polled() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let shell = directory.path().join("resize-backpressure");
        std::fs::write(&shell, format!(
            "#!/bin/sh\nstty -echo -onlcr\ni=0\nwhile IFS= read -r value; do\ni=$((i+1))\nprintf '%s\\033]0;step-%s\\007' '{}' \"$i\"\ndone\n",
            "x".repeat(4096)
        )).unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut terminal = Terminal::new(TerminalConfig {
            shell: Some(shell.to_str().unwrap().into()),
            size: (8, 3),
            ..Default::default()
        });
        terminal.start().unwrap();
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut backpressure = false;
        'bursts: for step in 1..=20 {
            terminal.write_string("\n").unwrap();
            loop {
                if let Err(error) = terminal.resize(8, 4) {
                    assert!(
                        error.to_string().contains("Poll terminal events"),
                        "{error}"
                    );
                    backpressure = true;
                    break 'bursts;
                }
                if terminal.title() == format!("step-{step}") {
                    break;
                }
                assert!(Instant::now() < deadline, "child output did not arrive");
                std::thread::sleep(Duration::from_millis(2));
            }
        }
        assert!(
            backpressure,
            "resizes exceeded the 64 KiB event budget without backpressure"
        );
        assert!(terminal.resize_output_bytes <= 64 * 1024);
        assert!(!terminal.poll_events().is_empty());
        terminal.resize(9, 4).unwrap();
        terminal.stop().unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn resize_consumes_queued_output_at_old_height_and_preserves_events() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let shell = directory.path().join("queued-output");
        std::fs::write(&shell, "#!/bin/sh\nstty -onlcr\nprintf 'one\\r\\ntwo\\r\\nthree\\r\\nfour\\033]0;queued title\\007'\n").unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut terminal = Terminal::new(TerminalConfig {
            shell: Some(shell.to_str().unwrap().into()),
            size: (8, 3),
            ..Default::default()
        });
        terminal.start().unwrap();
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        while terminal.pty.try_wait().unwrap().is_none() {
            assert!(Instant::now() < deadline, "output owner did not finish");
            std::thread::sleep(Duration::from_millis(2));
        }
        // The reaped PTY owner has queued every output byte. Do not poll it yet.
        terminal.stop().unwrap();
        terminal.resize(8, 5).unwrap();
        assert_eq!(terminal.screen().scrollback_len(), 1);
        let first = (0..8)
            .map(|x| terminal.cell_at(x, 0).unwrap().character.as_str())
            .collect::<String>();
        assert_eq!(first.trim(), "two");
        let events = terminal.poll_events();
        let raw = events
            .iter()
            .filter_map(|event| match event {
                TerminalEvent::Output(bytes) => Some(bytes.as_slice()),
                _ => None,
            })
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(raw, b"one\r\ntwo\r\nthree\r\nfour\x1b]0;queued title\x07");
        assert_eq!(events.iter().filter(|event| matches!(event, TerminalEvent::TitleChanged(title) if title == "queued title")).count(), 1);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, TerminalEvent::ProcessExited(0)))
                .count(),
            1
        );
        assert!(terminal.poll_events().is_empty());
    }

    #[cfg(windows)]
    #[test]
    fn windows_command_shell_paints_its_prompt_and_calculated_result() {
        for _ in 0..4 {
            assert_eq!(
                windows_command_shell_resize(false, (31, 7), (35, 9)),
                (35, 9),
                "terminal reports the resized dimensions"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_command_shell_resize_after_complete_prompt() {
        assert_eq!(
            windows_command_shell_resize(true, (43, 11), (47, 13)),
            (47, 13),
            "terminal reports the resized dimensions"
        );
    }

    #[cfg(windows)]
    /// Drive the command shell through a resize and return the terminal's final size.
    fn windows_command_shell_resize(
        wait_for_prompt: bool,
        size: (u16, u16),
        resized: (u16, u16),
    ) -> (u16, u16) {
        let directory = tempfile::tempdir().unwrap();
        let mut terminal = Terminal::new(TerminalConfig {
            shell: Some(std::env::var("COMSPEC").unwrap()),
            working_directory: Some(directory.path().to_str().unwrap().into()),
            env: vec![("PROMPT".into(), "READY$G".into())],
            size,
            ..Default::default()
        });
        terminal.start().unwrap();
        let mut raw = Vec::new();
        for (expected, input, resize) in [
            ("READY>", Some("@set /a 41+1\r"), None),
            ("42", Some("@set /a 52+1\r"), Some(resized)),
            ("53", None, None),
        ] {
            // A hang guard, not a timing check.
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                for event in terminal.poll_events() {
                    if let TerminalEvent::Output(bytes) = event {
                        raw.extend(bytes);
                    }
                }
                let (width, height) = terminal.size();
                let screen = (0..height)
                    .map(|y| {
                        (0..width)
                            .map(|x| terminal.cell_at(x, y).unwrap().character.as_str())
                            .collect::<String>()
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                let complete = if expected == "READY>" {
                    screen.contains(expected)
                } else {
                    screen.lines().any(|line| line.trim() == expected)
                };
                let prompt_complete = expected != "42"
                    || !wait_for_prompt
                    || screen
                        .lines()
                        .nth(usize::from(terminal.cursor_position().1))
                        .is_some_and(|line| line.trim() == "READY>");
                if complete && prompt_complete {
                    eprintln!(
                        "stage {expected}; wait_for_prompt={wait_for_prompt}; raw={:?}; screen={screen:?}; cursor={:?}",
                        String::from_utf8_lossy(&raw), terminal.cursor_position()
                    );
                    break;
                }
                assert!(
                    Instant::now() < deadline,
                    "missing {expected}; raw={:?}; screen={screen:?}; running={}; error={:?}",
                    String::from_utf8_lossy(&raw),
                    terminal.is_running(),
                    terminal.last_error()
                );
                assert!(
                    raw.len() < 256 * 1024,
                    "unexpected command-shell output flood"
                );
                std::thread::sleep(Duration::from_millis(10));
            }
            if let Some((width, height)) = resize {
                terminal.resize(width, height).unwrap();
            }
            if let Some(input) = input {
                terminal.write_string(input).unwrap();
            }
        }
        let final_size = terminal.size();
        terminal.stop().unwrap();
        final_size
    }

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
    fn oversized_resize_preserves_pty_screen_and_revision() {
        let mut terminal = Terminal::new(TerminalConfig::default());
        terminal.process_output(b"KEEP");
        let revision = terminal.revision();
        assert!(terminal.resize(1024, 257).is_err());
        assert_eq!(terminal.pty.size(), (80, 24));
        assert_eq!(terminal.size(), (80, 24));
        assert_eq!(terminal.screen().size(), (80, 24));
        assert_eq!(terminal.revision(), revision);
        assert_eq!(terminal.cell_at(0, 0).unwrap().character, "K");
    }

    #[test]
    fn test_key_sequences() {
        let config = TerminalConfig::default();
        let mut terminal = Terminal::new(config);

        // Unstarted input must report an error instead of a successful no-op.
        assert!(terminal.send_key("a").is_err());
        assert!(terminal.send_key("Enter").is_err());
        assert!(terminal.send_key("Up").is_err());
        assert!(terminal.send_key("InvalidKey").is_err());
    }

    #[test]
    fn cursor_helper_clamps_extreme_coordinates() {
        let mut terminal = Terminal::new(TerminalConfig::default());
        terminal.move_cursor(u16::MAX, u16::MAX).unwrap();
        assert_eq!(terminal.cursor_position(), (79, 23));
    }

    #[test]
    fn title_helper_rejects_control_injection_without_changing_screen() {
        let mut terminal = Terminal::new(TerminalConfig::default());
        terminal.process_output(b"KEEP");
        terminal.set_title("original").unwrap();
        assert!(terminal.set_title("bad\x07\x1b[2J").is_err());
        assert!(terminal.set_title(&"x".repeat(8191)).is_err());
        assert_eq!(terminal.screen().title(), "original");
        assert_eq!(terminal.screen().cell_at(0, 0).unwrap().character, "K");
    }

    #[test]
    fn title_keeps_semicolons_and_unicode() {
        let mut terminal = Terminal::new(TerminalConfig::default());
        terminal.set_title("é; workspace; 2").unwrap();
        assert_eq!(terminal.screen().title(), "é; workspace; 2");
    }

    #[cfg(unix)]
    #[test]
    fn named_keys_follow_application_cursor_mode_over_pty() {
        use std::os::unix::fs::PermissionsExt;
        let fixture = tempfile::tempdir().unwrap();
        let shell = fixture.path().join("keys");
        std::fs::write(
            &shell,
            "#!/bin/sh\nstty raw -echo\nprintf READY\ndd bs=1 count=36 2>/dev/null | od -An -tx1\n",
        )
        .unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut terminal = Terminal::new(TerminalConfig {
            shell: Some(shell.to_str().unwrap().into()),
            ..Default::default()
        });
        terminal.start().unwrap();
        // A hang guard, not a timing check: generous so a busy machine
        // cannot fail a correct test by running it slowly.
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut output = Vec::new();
        while Instant::now() < deadline {
            if let Some(bytes) = terminal
                .pty
                .read_output(Some(Duration::from_millis(20)))
                .unwrap()
            {
                output.extend(bytes);
            }
            if output.ends_with(b"READY") {
                break;
            }
        }
        assert!(output.ends_with(b"READY"));
        for mode in [b"\x1b[?1h", b"\x1b[?1l"] {
            terminal.process_output(mode);
            for key in ["Up", "Down", "Right", "Left", "Home", "End"] {
                terminal.send_key(key).unwrap();
            }
        }
        output.clear();
        while Instant::now() < deadline {
            if let Some(bytes) = terminal
                .pty
                .read_output(Some(Duration::from_millis(20)))
                .unwrap()
            {
                output.extend(bytes);
            }
            if String::from_utf8_lossy(&output).split_whitespace().count() == 36 {
                break;
            }
        }
        let actual = String::from_utf8_lossy(&output)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(actual, "1b 4f 41 1b 4f 42 1b 4f 43 1b 4f 44 1b 4f 48 1b 4f 46 1b 5b 41 1b 5b 42 1b 5b 43 1b 5b 44 1b 5b 48 1b 5b 46");
        terminal.stop().unwrap();
    }
}
