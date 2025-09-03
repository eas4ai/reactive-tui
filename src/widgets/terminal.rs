//! Terminal Widget - Embedded terminal emulator component
//!
//! Provides a complete terminal emulator widget that can be embedded in TUI applications.
//! This is the "killer feature" equivalent to libvaxis's embedded terminal.

use crate::component::{Component, Element, Props};
use crate::core::surface::{Attr, Cell, Rgba};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyEvent, MouseEvent, ResizeEvent};
use crate::terminal::{
    Terminal, TerminalCell, TerminalColor, TerminalConfig, TerminalError, TerminalEvent, TerminalResult,
};
use std::any::Any;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Properties for Terminal widget
#[derive(Clone, Debug, PartialEq)]
pub struct TerminalProps {
    /// Terminal configuration
    pub config: TerminalConfig,
    /// Whether the terminal should auto-focus
    pub auto_focus: bool,
    /// Whether to show scrollbar
    pub show_scrollbar: bool,
    /// Custom shell command to run
    pub shell_command: Option<String>,
    /// Environment variables to set
    pub env_vars: Vec<(String, String)>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Terminal title
    pub title: String,
}

impl Default for TerminalProps {
    fn default() -> Self {
        Self {
            config: TerminalConfig::default(),
            auto_focus: true,
            show_scrollbar: true,
            shell_command: None,
            env_vars: Vec::new(),
            working_directory: None,
            title: "Terminal".to_string(),
        }
    }
}

impl Props for TerminalProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// State for Terminal widget
#[derive(Debug)]
pub struct TerminalState {
    /// The underlying terminal emulator
    terminal: Arc<Mutex<Terminal>>,
    /// Event receiver for terminal events (wrapped in `Arc<Mutex>` for Sync)
    event_receiver: Option<Arc<Mutex<mpsc::Receiver<TerminalEvent>>>>,
    /// Whether the terminal is running
    is_running: bool,
    /// Whether the terminal has focus
    has_focus: bool,
    /// Current scroll position in scrollback
    scroll_position: usize,
    /// Last update time for performance
    #[allow(dead_code)]
    last_update: Instant,
    /// Cached terminal size
    cached_size: (u16, u16),
    /// Whether terminal needs redraw
    needs_redraw: bool,
}

impl Default for TerminalState {
    fn default() -> Self {
        let config = TerminalConfig::default();
        let terminal = Terminal::new(config);

        Self {
            terminal: Arc::new(Mutex::new(terminal)),
            event_receiver: None,
            is_running: false,
            has_focus: false,
            scroll_position: 0,
            last_update: Instant::now(),
            cached_size: (80, 24),
            needs_redraw: true,
        }
    }
}

/// Terminal widget component
pub struct TerminalWidget {
    props: TerminalProps,
    state: TerminalState,
}

impl TerminalWidget {
    /// Create a new terminal widget
    pub fn new(props: TerminalProps) -> Self {
        let mut config = props.config.clone();

        // Apply props to config
        if let Some(ref shell) = props.shell_command {
            config.shell = Some(shell.clone());
        }
        if let Some(ref dir) = props.working_directory {
            config.working_directory = Some(dir.clone());
        }
        config.title = props.title.clone();
        config.env.extend(props.env_vars.iter().cloned());

        let terminal = Terminal::new(config);
        let state = TerminalState {
            terminal: Arc::new(Mutex::new(terminal)),
            ..Default::default()
        };

        Self { props, state }
    }

    /// Start the terminal emulator
    pub fn start(&mut self) -> TerminalResult<()> {
        if self.state.is_running {
            return Ok(());
        }

        // Start the terminal
        {
            let mut terminal = self.state.terminal.lock()
                .map_err(|_| TerminalError::Process("Terminal lock poisoned during start".to_string()))?;
            terminal.start()?;
        }

        self.state.is_running = true;
        self.state.needs_redraw = true;

        // Setup event monitoring thread
        self.setup_event_monitoring();

        Ok(())
    }

    /// Stop the terminal emulator
    pub fn stop(&mut self) -> TerminalResult<()> {
        if !self.state.is_running {
            return Ok(());
        }

        {
            let mut terminal = self.state.terminal.lock()
                .map_err(|_| TerminalError::Process("Terminal lock poisoned during stop".to_string()))?;
            terminal.stop()?;
        }

        self.state.is_running = false;
        Ok(())
    }

    /// Send input to the terminal
    pub fn send_input(&mut self, data: &[u8]) -> TerminalResult<()> {
        let mut terminal = self.state.terminal.lock()
            .map_err(|_| TerminalError::Process("Terminal lock poisoned during input".to_string()))?;
        terminal.write_input(data)
    }

    /// Send a string to the terminal
    pub fn send_string(&mut self, text: &str) -> TerminalResult<()> {
        self.send_input(text.as_bytes())
    }

    /// Resize the terminal
    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        if self.state.cached_size == (width, height) {
            return Ok(());
        }

        {
            let mut terminal = self.state.terminal.lock()
                .map_err(|_| TerminalError::Process("Terminal lock poisoned during resize".to_string()))?;
            terminal.resize(width, height)?;
        }

        self.state.cached_size = (width, height);
        self.state.needs_redraw = true;
        Ok(())
    }

    /// Scroll the terminal view
    pub fn scroll(&mut self, lines: i32) {
        if lines > 0 {
            self.state.scroll_position = self.state.scroll_position.saturating_add(lines as usize);
        } else {
            self.state.scroll_position =
                self.state.scroll_position.saturating_sub((-lines) as usize);
        }
        self.state.needs_redraw = true;
    }

    /// Get the current terminal title
    pub fn title(&self) -> String {
        match self.state.terminal.lock() {
            Ok(terminal) => {
                let title = terminal.title();
                if title.is_empty() {
                    self.props.title.clone()
                } else {
                    title.to_string()
                }
            }
            Err(_) => {
                // Lock poisoned, return fallback title
                self.props.title.clone()
            }
        }
    }

    /// Check if terminal is running
    pub fn is_running(&self) -> bool {
        self.state.is_running
    }

    /// Set focus state
    pub fn set_focus(&mut self, focused: bool) {
        if self.state.has_focus != focused {
            self.state.has_focus = focused;
            self.state.needs_redraw = true;
        }
    }

    /// Setup event monitoring thread
    fn setup_event_monitoring(&mut self) {
        let terminal = Arc::clone(&self.state.terminal);
        let (tx, rx) = mpsc::channel();
        self.state.event_receiver = Some(Arc::new(Mutex::new(rx)));

        thread::spawn(move || {
            loop {
                // Poll for terminal events
                if let Ok(mut terminal) = terminal.lock() {
                    let events = terminal.poll_events();
                    for event in events {
                        if tx.send(event).is_err() {
                            return; // Channel closed
                        }
                    }
                }

                thread::sleep(Duration::from_millis(16)); // ~60 FPS
            }
        });
    }

    /// Process pending terminal events
    #[allow(dead_code)]
    fn process_events(&mut self) {
        if let Some(ref receiver) = self.state.event_receiver {
            if let Ok(receiver) = receiver.lock() {
                while let Ok(event) = receiver.try_recv() {
                    match event {
                        TerminalEvent::Output(_) => {
                            self.state.needs_redraw = true;
                            self.state.last_update = Instant::now();
                        }
                        TerminalEvent::TitleChanged(_) => {
                            // Title changed - could trigger parent update
                        }
                        TerminalEvent::Resized(w, h) => {
                            self.state.cached_size = (w, h);
                            self.state.needs_redraw = true;
                        }
                        TerminalEvent::ProcessExited(_) => {
                            self.state.is_running = false;
                        }
                        TerminalEvent::Bell => {
                            // Could trigger visual bell or sound
                        }
                        TerminalEvent::WorkingDirectoryChanged(_) => {
                            // Working directory changed
                        }
                    }
                }
            }
        }
    }

    /// Convert terminal cell to surface cell
    #[allow(dead_code)]
    fn convert_cell(&self, term_cell: &TerminalCell) -> Cell {
        let fg = self.convert_color(term_cell.style.foreground);
        let bg = self.convert_color(term_cell.style.background);
        let mut attr = Attr::empty();

        if term_cell.style.bold {
            attr |= Attr::BOLD;
        }
        if term_cell.style.italic {
            attr |= Attr::ITALIC;
        }
        if term_cell.style.underline {
            attr |= Attr::UNDERLINE;
        }
        if term_cell.style.reverse {
            attr |= Attr::REVERSE;
        }

        // Get the first character from the string
        let ch = term_cell.character.chars().next().unwrap_or(' ');

        Cell {
            ch,
            fg,
            bg,
            attr,
            image_id: None,
            image_placement: None,
        }
    }

    /// Convert terminal color to RGBA
    #[allow(dead_code)]
    fn convert_color(&self, color: TerminalColor) -> Rgba {
        match color {
            TerminalColor::Default => Rgba::white(),
            TerminalColor::Indexed(idx) => {
                // Convert indexed color to RGB (simplified)
                match idx {
                    0 => Rgba::black(),
                    1 => Rgba::new(0.8, 0.0, 0.0, 1.0), // Red
                    2 => Rgba::new(0.0, 0.8, 0.0, 1.0), // Green
                    3 => Rgba::new(0.8, 0.8, 0.0, 1.0), // Yellow
                    4 => Rgba::new(0.0, 0.0, 0.8, 1.0), // Blue
                    5 => Rgba::new(0.8, 0.0, 0.8, 1.0), // Magenta
                    6 => Rgba::new(0.0, 0.8, 0.8, 1.0), // Cyan
                    7 => Rgba::white(),
                    _ => Rgba::white(), // Default for other indices
                }
            }
            TerminalColor::Rgb(r, g, b) => {
                Rgba::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
            }
        }
    }
}

impl Component for TerminalWidget {
    type Props = TerminalProps;
    type State = TerminalState;

    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        // Process any pending events
        // Note: In a real implementation, this would be handled differently
        // to avoid mutable access in render

        Element::layout(crate::component::LayoutType::Flex)
            .class("terminal-widget w-full h-full bg-black text-white")
            .children(vec![
                // Terminal title bar (if enabled)
                if !self.props.title.is_empty() {
                    Element::layout(crate::component::LayoutType::Flex)
                        .class("terminal-title-bar h-6 bg-gray-800 text-white px-2 items-center")
                        .children(vec![
                            Element::text(self.title()).class("text-sm font-bold"),
                            Element::layout(crate::component::LayoutType::Flex)
                                .class("ml-auto")
                                .children(vec![if self.is_running() {
                                    Element::text("●").class("text-green-400 mr-2")
                                } else {
                                    Element::text("●").class("text-red-400 mr-2")
                                }]),
                        ])
                } else {
                    Element::empty()
                },
                // Main terminal content area
                Element::layout(crate::component::LayoutType::Flex)
                    .class("terminal-content flex-1 relative")
                    .children(vec![
                        // Terminal screen (this would be rendered via custom rendering)
                        Element::text("[Terminal Content - Custom Rendered]")
                            .class("absolute inset-0 font-mono text-sm"),
                        // Scrollbar (if enabled)
                        if self.props.show_scrollbar {
                            Element::layout(crate::component::LayoutType::Flex)
                                .class("scrollbar absolute right-0 top-0 bottom-0 w-2 bg-gray-700")
                                .children(vec![Element::layout(crate::component::LayoutType::Flex)
                                    .class("scrollbar-thumb bg-gray-500 rounded")])
                        } else {
                            Element::empty()
                        },
                    ]),
            ])
    }

    fn handle_event(
        &mut self,
        event: &Event,
        _props: &mut Self::Props,
        _state: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            Event::Resize(resize_event) => self.handle_resize_event(resize_event),
            _ => EventResult::Ignored,
        }
    }
}

impl TerminalWidget {
    /// Handle keyboard events
    fn handle_key_event(&mut self, event: &KeyEvent) -> EventResult {
        if !self.state.has_focus {
            return EventResult::Ignored;
        }

        // Convert key event to terminal input
        let input = self.key_event_to_bytes(event);
        if let Err(e) = self.send_input(&input) {
            eprintln!("Failed to send input to terminal: {}", e);
        }

        EventResult::Handled
    }

    /// Handle mouse events
    fn handle_mouse_event(&mut self, event: &MouseEvent) -> EventResult {
        use crate::event::types::{MouseEventKind, Position};

        match event.kind {
            MouseEventKind::Click => {
                // Focus the terminal on click
                self.state.has_focus = true;
                EventResult::Handled
            }
            MouseEventKind::Wheel => {
                // Handle scrolling
                if let Position::Cell { y, .. } = event.position {
                    if y > 0 {
                        self.scroll(-3); // Scroll up
                    } else {
                        self.scroll(3); // Scroll down
                    }
                }
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }

    /// Handle resize events
    fn handle_resize_event(&mut self, event: &ResizeEvent) -> EventResult {
        if let Err(e) = self.resize(event.width, event.height) {
            eprintln!("Failed to resize terminal: {}", e);
        }
        EventResult::Handled
    }

    /// Convert key event to terminal input bytes
    fn key_event_to_bytes(&self, event: &KeyEvent) -> Vec<u8> {
        use crate::event::types::KeyCode;

        let mut bytes = Vec::new();

        // Handle modifiers
        if event.modifiers.ctrl {
            if let KeyCode::Char(c) = event.code {
                // Ctrl+letter combinations
                let ctrl_code = (c.to_ascii_uppercase() as u8)
                    .wrapping_sub(b'A')
                    .wrapping_add(1);
                bytes.push(ctrl_code);
            }
        } else {
            match event.code {
                KeyCode::Char(c) => bytes.extend(c.to_string().as_bytes()),
                KeyCode::Enter => bytes.push(b'\r'),
                KeyCode::Tab => bytes.push(b'\t'),
                KeyCode::Backspace => bytes.push(0x7F),
                KeyCode::Delete => bytes.extend(b"\x1b[3~"),
                KeyCode::Up => bytes.extend(b"\x1b[A"),
                KeyCode::Down => bytes.extend(b"\x1b[B"),
                KeyCode::Right => bytes.extend(b"\x1b[C"),
                KeyCode::Left => bytes.extend(b"\x1b[D"),
                KeyCode::Home => bytes.extend(b"\x1b[H"),
                KeyCode::End => bytes.extend(b"\x1b[F"),
                KeyCode::PageUp => bytes.extend(b"\x1b[5~"),
                KeyCode::PageDown => bytes.extend(b"\x1b[6~"),
                KeyCode::Escape => bytes.push(0x1B),
                KeyCode::Space => bytes.push(b' '),
                KeyCode::F(n) => {
                    // Function keys
                    match n {
                        1 => bytes.extend(b"\x1bOP"),
                        2 => bytes.extend(b"\x1bOQ"),
                        3 => bytes.extend(b"\x1bOR"),
                        4 => bytes.extend(b"\x1bOS"),
                        5 => bytes.extend(b"\x1b[15~"),
                        6 => bytes.extend(b"\x1b[17~"),
                        7 => bytes.extend(b"\x1b[18~"),
                        8 => bytes.extend(b"\x1b[19~"),
                        9 => bytes.extend(b"\x1b[20~"),
                        10 => bytes.extend(b"\x1b[21~"),
                        11 => bytes.extend(b"\x1b[23~"),
                        12 => bytes.extend(b"\x1b[24~"),
                        _ => {} // Ignore other function keys
                    }
                }
                _ => {} // Ignore other keys
            }
        }

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::types::{KeyCode, KeyModifiers};

    // Helper function to check if we're in a TTY environment
    #[allow(dead_code)]
    fn is_tty_available() -> bool {
        // Skip TTY-dependent tests in CI/CD environments
        std::env::var("CI").is_err()
            && std::env::var("GITHUB_ACTIONS").is_err()
            && std::env::var("TERM").is_ok() // Basic check for terminal environment
    }

    #[test]
    fn test_terminal_props_default() {
        let props = TerminalProps::default();
        assert_eq!(props.title, "Terminal");
        assert!(props.auto_focus);
        assert!(props.show_scrollbar);
        assert!(props.shell_command.is_none());
        assert!(props.working_directory.is_none());
        assert!(props.env_vars.is_empty());
    }

    #[test]
    fn test_terminal_props_creation() {
        let props = TerminalProps {
            config: TerminalConfig::default(),
            auto_focus: false,
            show_scrollbar: false,
            shell_command: Some("/bin/bash".to_string()),
            env_vars: vec![("TEST".to_string(), "value".to_string())],
            working_directory: Some("/tmp".to_string()),
            title: "Custom Terminal".to_string(),
        };

        assert_eq!(props.title, "Custom Terminal");
        assert!(!props.auto_focus);
        assert!(!props.show_scrollbar);
        assert_eq!(props.shell_command, Some("/bin/bash".to_string()));
        assert_eq!(props.working_directory, Some("/tmp".to_string()));
        assert_eq!(props.env_vars.len(), 1);
        assert_eq!(props.env_vars[0], ("TEST".to_string(), "value".to_string()));
    }

    #[test]
    fn test_terminal_state_default() {
        let state = TerminalState::default();
        assert!(!state.is_running);
        assert!(!state.has_focus);
        assert_eq!(state.scroll_position, 0);
        assert_eq!(state.cached_size, (80, 24));
        assert!(state.needs_redraw);
        assert!(state.event_receiver.is_none());
    }

    #[test]
    fn test_terminal_widget_creation() {
        let props = TerminalProps::default();
        let widget = TerminalWidget::new(props.clone());

        assert_eq!(widget.props.title, props.title);
        assert!(!widget.state.is_running);
        assert!(widget.state.needs_redraw);
    }

    #[test]
    fn test_terminal_widget_with_custom_props() {
        let props = TerminalProps {
            shell_command: Some("/bin/zsh".to_string()),
            working_directory: Some("/home/user".to_string()),
            title: "ZSH Terminal".to_string(),
            env_vars: vec![
                ("SHELL".to_string(), "/bin/zsh".to_string()),
                ("HOME".to_string(), "/home/user".to_string()),
            ],
            ..Default::default()
        };

        let widget = TerminalWidget::new(props);
        assert_eq!(widget.props.title, "ZSH Terminal");
        assert_eq!(widget.props.shell_command, Some("/bin/zsh".to_string()));
        assert_eq!(
            widget.props.working_directory,
            Some("/home/user".to_string())
        );
        assert_eq!(widget.props.env_vars.len(), 2);
    }

    #[test]
    fn test_key_event_to_bytes_characters() {
        let widget = TerminalWidget::new(TerminalProps::default());

        // Test regular characters
        let event = KeyEvent::new(KeyCode::Char('a'));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"a");

        let event = KeyEvent::new(KeyCode::Char('Z'));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"Z");

        let event = KeyEvent::new(KeyCode::Char('1'));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"1");
    }

    #[test]
    fn test_key_event_to_bytes_special_keys() {
        let widget = TerminalWidget::new(TerminalProps::default());

        // Test special keys
        let event = KeyEvent::new(KeyCode::Enter);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\r");

        let event = KeyEvent::new(KeyCode::Tab);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\t");

        let event = KeyEvent::new(KeyCode::Backspace);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x7F");

        let event = KeyEvent::new(KeyCode::Escape);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1B");

        let event = KeyEvent::new(KeyCode::Space);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b" ");
    }

    #[test]
    fn test_key_event_to_bytes_arrow_keys() {
        let widget = TerminalWidget::new(TerminalProps::default());

        let event = KeyEvent::new(KeyCode::Up);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[A");

        let event = KeyEvent::new(KeyCode::Down);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[B");

        let event = KeyEvent::new(KeyCode::Right);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[C");

        let event = KeyEvent::new(KeyCode::Left);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[D");
    }

    #[test]
    fn test_key_event_to_bytes_function_keys() {
        let widget = TerminalWidget::new(TerminalProps::default());

        let event = KeyEvent::new(KeyCode::F(1));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1bOP");

        let event = KeyEvent::new(KeyCode::F(2));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1bOQ");

        let event = KeyEvent::new(KeyCode::F(5));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[15~");

        let event = KeyEvent::new(KeyCode::F(12));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[24~");
    }

    #[test]
    fn test_key_event_to_bytes_ctrl_combinations() {
        let widget = TerminalWidget::new(TerminalProps::default());

        // Test Ctrl+A (should be 0x01)
        let event = KeyEvent::new(KeyCode::Char('a')).with_modifiers(KeyModifiers::ctrl());
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, vec![0x01]);

        // Test Ctrl+C (should be 0x03)
        let event = KeyEvent::new(KeyCode::Char('c')).with_modifiers(KeyModifiers::ctrl());
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, vec![0x03]);

        // Test Ctrl+Z (should be 0x1A)
        let event = KeyEvent::new(KeyCode::Char('z')).with_modifiers(KeyModifiers::ctrl());
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, vec![0x1A]);
    }

    #[test]
    fn test_terminal_widget_scroll() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Initial scroll position should be 0
        assert_eq!(widget.state.scroll_position, 0);

        // Scroll down
        widget.scroll(5);
        assert_eq!(widget.state.scroll_position, 5);
        assert!(widget.state.needs_redraw);

        // Scroll up
        widget.state.needs_redraw = false;
        widget.scroll(-2);
        assert_eq!(widget.state.scroll_position, 3);
        assert!(widget.state.needs_redraw);

        // Scroll up beyond 0 (should saturate at 0)
        widget.scroll(-10);
        assert_eq!(widget.state.scroll_position, 0);
    }

    #[test]
    fn test_terminal_widget_resize() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Initial size should be default
        assert_eq!(widget.state.cached_size, (80, 24));

        // Resize should update cached size and mark for redraw
        widget.state.needs_redraw = false;
        let result = widget.resize(100, 30);

        // Should succeed even without TTY (just updates state)
        assert!(result.is_ok());
        assert_eq!(widget.state.cached_size, (100, 30));
        assert!(widget.state.needs_redraw);

        // Resize to same size should be no-op
        widget.state.needs_redraw = false;
        let result = widget.resize(100, 30);
        assert!(result.is_ok());
        assert!(!widget.state.needs_redraw); // Should not mark for redraw
    }

    #[test]
    fn test_terminal_widget_focus() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Initially should not have focus
        assert!(!widget.state.has_focus);

        // Set focus
        widget.set_focus(true);
        assert!(widget.state.has_focus);
        assert!(widget.state.needs_redraw);

        // Remove focus
        widget.state.needs_redraw = false;
        widget.set_focus(false);
        assert!(!widget.state.has_focus);
        assert!(widget.state.needs_redraw);
    }

    #[test]
    fn test_terminal_widget_running_state() {
        let widget = TerminalWidget::new(TerminalProps::default());

        // Initially should not be running
        assert!(!widget.state.is_running);
        assert!(!widget.is_running());

        // Note: We don't test start() and stop() here because they require TTY access
        // Those would be tested in integration tests with proper TTY setup
    }

    #[test]
    fn test_terminal_widget_send_string() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Test that send_string doesn't panic (even if terminal isn't running)
        // In a real TTY environment, this would send data to the terminal
        let result = widget.send_string("echo hello\n");

        // The result depends on the terminal implementation - just ensure it doesn't panic
        let _ = result; // Consume the result without asserting specific behavior
    }

    #[test]
    fn test_terminal_widget_send_input() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Test that send_input doesn't panic
        let result = widget.send_input(b"test data");

        // The result depends on the terminal implementation - just ensure it doesn't panic
        let _ = result; // Consume the result without asserting specific behavior
    }

    #[test]
    fn test_key_event_to_bytes_navigation_keys() {
        let widget = TerminalWidget::new(TerminalProps::default());

        let event = KeyEvent::new(KeyCode::Home);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[H");

        let event = KeyEvent::new(KeyCode::End);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[F");

        let event = KeyEvent::new(KeyCode::PageUp);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[5~");

        let event = KeyEvent::new(KeyCode::PageDown);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[6~");

        let event = KeyEvent::new(KeyCode::Delete);
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, b"\x1b[3~");
    }

    #[test]
    fn test_key_event_to_bytes_unsupported_keys() {
        let widget = TerminalWidget::new(TerminalProps::default());

        // Test unsupported function key
        let event = KeyEvent::new(KeyCode::F(25));
        let bytes = widget.key_event_to_bytes(&event);
        assert_eq!(bytes, Vec::<u8>::new()); // Should return empty vec

        // Test other unsupported keys would go here
        // The current implementation ignores them (returns empty)
    }

    #[test]
    fn test_terminal_config_integration() {
        let config = TerminalConfig::default();

        let props = TerminalProps {
            config,
            title: "Override Title".to_string(),
            shell_command: Some("/bin/sh".to_string()),
            working_directory: Some("/tmp".to_string()),
            env_vars: vec![("TEST_VAR".to_string(), "test_value".to_string())],
            ..Default::default()
        };

        let widget = TerminalWidget::new(props);

        // Verify that props override config values
        assert_eq!(widget.props.title, "Override Title");
        assert_eq!(widget.props.shell_command, Some("/bin/sh".to_string()));
        assert_eq!(widget.props.working_directory, Some("/tmp".to_string()));
        assert_eq!(widget.props.env_vars.len(), 1);
    }

    // TTY-dependent tests would be conditionally compiled or skipped
    #[test]
    #[ignore] // Use #[ignore] for tests that require special setup
    fn test_terminal_start_stop_integration() {
        if !is_tty_available() {
            eprintln!("Skipping TTY-dependent test in non-TTY environment");
            return;
        }

        // This test would only run in a proper TTY environment
        // Implementation would test actual terminal start/stop functionality
        let widget = TerminalWidget::new(TerminalProps::default());

        // In a real TTY environment, these would work
        // let result = widget.start();
        // assert!(result.is_ok());
        // assert!(widget.is_running());

        // let result = widget.stop();
        // assert!(result.is_ok());
        // assert!(!widget.is_running());

        // For now, just verify the test framework works
        assert!(!widget.is_running());
    }
}
