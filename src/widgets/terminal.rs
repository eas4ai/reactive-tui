//! Terminal Widget - Embedded terminal emulator component
//!
//! Provides a complete terminal emulator widget that can be embedded in TUI applications.
//! This is the "killer feature" equivalent to libvaxis's embedded terminal.

use crate::component::{Component, Element, Props};
use crate::core::surface::{Attr, Cell, Rgba};
use crate::event::router::EventResult;
use crate::event::types::{Event, KeyEvent, MouseEvent, ResizeEvent};
use crate::terminal::{
    Terminal, TerminalCell, TerminalColor, TerminalConfig, TerminalEvent, TerminalResult,
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
    /// Event receiver for terminal events (wrapped in Arc<Mutex> for Sync)
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
            let mut terminal = self.state.terminal.lock().unwrap();
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
            let mut terminal = self.state.terminal.lock().unwrap();
            terminal.stop()?;
        }

        self.state.is_running = false;
        Ok(())
    }

    /// Send input to the terminal
    pub fn send_input(&mut self, data: &[u8]) -> TerminalResult<()> {
        let mut terminal = self.state.terminal.lock().unwrap();
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
            let mut terminal = self.state.terminal.lock().unwrap();
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
        let terminal = self.state.terminal.lock().unwrap();
        let title = terminal.title();
        if title.is_empty() {
            self.props.title.clone()
        } else {
            title.to_string()
        }
    }

    /// Check if terminal is running
    pub fn is_running(&self) -> bool {
        self.state.is_running
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
