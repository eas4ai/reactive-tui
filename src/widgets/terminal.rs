//! Terminal Widget - Embedded terminal emulator component
//!
//! Provides a complete terminal emulator widget that can be embedded in TUI applications.

use crate::component::{Component, Element, Props};
use crate::component::{LayoutInfo, LifecycleEvent};
use crate::event::router::EventResult;
use crate::event::types::{
    Event, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind, ResizeEvent, WheelDelta,
};
use crate::terminal::{Terminal, TerminalConfig, TerminalError, TerminalResult};
use std::any::Any;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use crate::terminal::keyboard;
mod blink;
mod monitor;
mod paint;

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

/// State for the retained terminal widget.
#[derive(Debug)]
pub struct TerminalState {
    terminal: Arc<Mutex<Terminal>>,
    has_focus: bool,
    scroll_position: usize,
    cached_size: (u16, u16),
    needs_redraw: bool,
}
impl Default for TerminalState {
    fn default() -> Self {
        Self {
            terminal: Arc::new(Mutex::new(Terminal::new(TerminalConfig::default()))),
            has_focus: false,
            scroll_position: 0,
            cached_size: (80, 24),
            needs_redraw: true,
        }
    }
}

/// A terminal whose child process and output monitor live until stop or removal.
pub struct TerminalWidget {
    props: TerminalProps,
    state: TerminalState,
    monitor: Option<monitor::Monitor>,
    blink: Arc<blink::Blink>,
    focused: Arc<AtomicBool>,
    scroll_dragging: Arc<AtomicBool>,
    layout: Option<LayoutInfo>,
    mounted: bool,
    launched: bool,
    error: Option<String>,
    invalid_size: Option<(u16, u16)>,
}

fn configuration(props: &TerminalProps) -> TerminalConfig {
    let mut config = props.config.clone();
    if let Some(shell) = &props.shell_command {
        config.shell = Some(shell.clone());
    }
    if let Some(dir) = &props.working_directory {
        config.working_directory = Some(dir.clone());
    }
    config.env.extend(props.env_vars.iter().cloned());
    config.title = props.title.clone();
    config
}
fn configured_terminal(
    mut config: TerminalConfig,
) -> (Terminal, Option<(u16, u16)>, Option<String>) {
    match Terminal::try_new(config.clone()) {
        Ok(terminal) => (terminal, None, None),
        Err(error) => {
            let size = config.size;
            // Retain a small screen for the error view. start() refuses it until resize succeeds.
            config.size = (1, 1);
            (Terminal::new(config), Some(size), Some(error.to_string()))
        }
    }
}

fn same_launch(a: &TerminalProps, b: &TerminalProps) -> bool {
    let mut a = configuration(a);
    let mut b = configuration(b);
    a.size = (1, 1);
    b.size = (1, 1);
    a.title.clear();
    b.title.clear();
    a == b
}

impl TerminalWidget {
    /// Construct an unstarted terminal. App starts it after measuring its content.
    pub fn new(props: TerminalProps) -> Self {
        let config = configuration(&props);
        let size = config.size;
        let (terminal, invalid_size, error) = configured_terminal(config);
        Self {
            state: TerminalState {
                cached_size: size,
                terminal: Arc::new(Mutex::new(terminal)),
                ..Default::default()
            },
            props,
            monitor: None,
            blink: Arc::new(blink::Blink::new()),
            focused: Arc::new(AtomicBool::new(false)),
            scroll_dragging: Arc::new(AtomicBool::new(false)),
            layout: None,
            mounted: false,
            launched: false,
            error,
            invalid_size,
        }
    }
    /// Start the child and its owned output monitor.
    pub fn start(&mut self) -> TerminalResult<()> {
        if let Some((width, height)) = self.invalid_size {
            return Err(TerminalError::InvalidSize { width, height });
        }
        if self.is_running() {
            return Ok(());
        }
        self.monitor = None;
        self.state
            .terminal
            .lock()
            .map_err(|_| lock_error())?
            .start()?;
        match monitor::Monitor::new(self.state.terminal.clone()) {
            Ok(monitor) => self.monitor = Some(monitor),
            Err(error) => {
                self.state
                    .terminal
                    .lock()
                    .map_err(|_| lock_error())?
                    .stop()?;
                return Err(TerminalError::Process(format!(
                    "Cannot monitor terminal: {error}"
                )));
            }
        }
        self.launched = true;
        self.error = None;
        self.state.needs_redraw = true;
        Ok(())
    }
    /// Stop monitoring, terminate and reap the owned child.
    pub fn stop(&mut self) -> TerminalResult<()> {
        self.blink.cancel();
        self.scroll_dragging.store(false, Ordering::Release);
        self.monitor = None;
        self.state.terminal.lock().map_err(|_| lock_error())?.stop()
    }
    /// Queue input or report a stopped child or full input queue.
    pub fn send_input(&mut self, data: &[u8]) -> TerminalResult<()> {
        self.state
            .terminal
            .lock()
            .map_err(|_| lock_error())?
            .write_input(data)
    }
    /// Queue UTF-8 input.
    pub fn send_string(&mut self, text: &str) -> TerminalResult<()> {
        self.send_input(text.as_bytes())
    }
    /// Resize the screen and child PTY in cells.
    pub fn resize(&mut self, width: u16, height: u16) -> TerminalResult<()> {
        crate::terminal::VirtualScreen::validate_size(width, height)?;
        if self.state.cached_size == (width, height) {
            return Ok(());
        }
        self.state
            .terminal
            .lock()
            .map_err(|_| lock_error())?
            .resize(width, height)?;
        self.state.cached_size = (width, height);
        if self.invalid_size.take().is_some() {
            self.error = None;
        }
        self.state.needs_redraw = true;
        Ok(())
    }
    /// Move into history with positive lines, toward live output with negative lines.
    pub fn scroll(&mut self, lines: i32) {
        self.state.scroll_position = if lines >= 0 {
            self.state.scroll_position.saturating_add(lines as usize)
        } else {
            self.state
                .scroll_position
                .saturating_sub(lines.unsigned_abs() as usize)
        };
        if let Ok(terminal) = self.state.terminal.lock() {
            self.state.scroll_position = self
                .state
                .scroll_position
                .min(terminal.screen().scrollback_len());
        }
        self.state.needs_redraw = true;
    }
    /// Current child title, with the configured title as fallback.
    pub fn title(&self) -> String {
        self.state
            .terminal
            .lock()
            .ok()
            .and_then(|terminal| {
                (!terminal.title().is_empty()).then(|| terminal.title().to_owned())
            })
            .unwrap_or_else(|| self.props.title.clone())
    }
    /// Whether the directly owned child is running.
    pub fn is_running(&self) -> bool {
        self.state
            .terminal
            .lock()
            .is_ok_and(|terminal| terminal.is_running())
    }
    /// Override focus for callers delivering events directly.
    pub fn set_focus(&mut self, focused: bool) {
        self.focused.store(focused, Ordering::Release);
        if !focused {
            self.blink.cancel();
            self.scroll_dragging.store(false, Ordering::Release);
        }
        self.state.has_focus = focused;
        self.state.needs_redraw = true;
    }
    /// Most recent widget or PTY error, also shown in its content area.
    pub fn last_error(&self) -> Option<String> {
        self.error.clone().or_else(|| {
            self.state
                .terminal
                .lock()
                .ok()
                .and_then(|terminal| terminal.last_error().map(str::to_owned))
        })
    }
    fn input(&mut self, input: &[u8]) -> EventResult {
        if input.is_empty() {
            return EventResult::Ignored;
        }
        match self.send_input(input) {
            Ok(()) => self.state.scroll_position = 0,
            Err(error) => self.error = Some(error.to_string()),
        }
        EventResult::Consumed
    }
    fn handle_key_event(&mut self, event: &KeyEvent) -> EventResult {
        if !self.focused.load(Ordering::Acquire) || event.kind == KeyEventKind::Release {
            return EventResult::Ignored;
        }
        self.input(&self.key_event_to_bytes(event))
    }
    fn key_event_to_bytes(&self, event: &KeyEvent) -> Vec<u8> {
        let application_cursor = self
            .state
            .terminal
            .lock()
            .is_ok_and(|terminal| terminal.screen().input_modes().application_cursor_keys);
        keyboard::encode(event, application_cursor)
    }
    fn scrollbar_offset(&self, event: &MouseEvent, dragging: bool) -> Option<usize> {
        let (width, height) = paint::content_size(self);
        if !self.props.show_scrollbar || height == 0 || self.last_error().is_some() {
            return None;
        }
        let insets = self.layout?.insets;
        let x = event.position.x() as f32 - insets[0];
        let y = event.position.y() as f32
            - insets[1]
            - f32::from(u8::from(!self.props.title.is_empty()));
        if !dragging
            && (!(f32::from(width)..f32::from(width) + 1.0).contains(&x)
                || !(0.0..f32::from(height)).contains(&y))
        {
            return None;
        }
        let history = self.state.terminal.lock().ok()?.screen().scrollback_len();
        let range = height - 1;
        if range == 0 {
            return Some(0);
        }
        let position = y.clamp(0.0, f32::from(range)) as u16;
        Some((history as u128 * u128::from(range - position) / u128::from(range)) as usize)
    }

    fn handle_mouse_event(&mut self, event: &MouseEvent) -> EventResult {
        match event.kind {
            MouseEventKind::Down | MouseEventKind::Click => {
                self.scroll_dragging.store(false, Ordering::Release);
                self.set_focus(true);
                if event.button == crate::event::types::MouseButton::Left {
                    if let Some(offset) = self.scrollbar_offset(event, false) {
                        self.state.scroll_position = offset;
                        self.state.needs_redraw = true;
                        self.scroll_dragging
                            .store(event.kind == MouseEventKind::Down, Ordering::Release);
                        return EventResult::Consumed;
                    }
                }
                EventResult::Handled
            }
            MouseEventKind::Drag
                if self.scroll_dragging.load(Ordering::Acquire)
                    && event.button == crate::event::types::MouseButton::Left =>
            {
                if let Some(offset) = self.scrollbar_offset(event, true) {
                    self.state.scroll_position = offset;
                    self.state.needs_redraw = true;
                }
                EventResult::Consumed
            }
            MouseEventKind::Up | MouseEventKind::Leave => {
                if self.scroll_dragging.swap(false, Ordering::AcqRel) {
                    EventResult::Consumed
                } else {
                    EventResult::Ignored
                }
            }
            MouseEventKind::Wheel => {
                let Some(wheel) = &event.wheel else {
                    return EventResult::Ignored;
                };
                let y = match wheel.delta {
                    WheelDelta::Lines { y, .. } | WheelDelta::Pixels { y, .. } => y,
                };
                if !y.is_finite() || y == 0.0 {
                    return EventResult::Ignored;
                }
                self.scroll((-y).round() as i32);
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }
    fn apply_measured_size(&mut self) -> bool {
        let (w, h) = paint::content_size(self);
        if w == 0 || h == 0 {
            return false;
        }
        let changed = self.state.cached_size != (w, h);
        if let Err(error) = self.resize(w, h) {
            self.error = Some(error.to_string());
            return changed;
        }
        if self.mounted && !self.launched {
            self.launched = true;
            if let Err(error) = self.start() {
                self.error = Some(error.to_string());
            }
            return true;
        }
        changed
    }
    fn handle_resize_event(&mut self, event: &ResizeEvent) -> EventResult {
        // App supplies local dimensions through layout, not host resize broadcasts.
        if self.mounted {
            return EventResult::Ignored;
        }
        if let Err(error) = self.resize(event.width, event.height) {
            self.error = Some(error.to_string());
        }
        EventResult::Handled
    }
}
fn lock_error() -> TerminalError {
    TerminalError::Process("Terminal lock poisoned".into())
}
impl Drop for TerminalWidget {
    fn drop(&mut self) {
        if let Err(error) = self.stop() {
            log::error!("Cannot stop terminal widget: {error}");
        }
    }
}
impl Component for TerminalWidget {
    type Props = TerminalProps;
    type State = TerminalState;
    fn new(props: Self::Props) -> Self {
        Self::new(props)
    }
    fn update(&mut self, props: &Self::Props, _: &mut Self::State) -> bool {
        if !same_launch(&self.props, props) {
            if let Err(error) = self.stop() {
                self.error = Some(error.to_string());
                return true;
            }
            let (terminal, invalid_size, error) = configured_terminal(configuration(props));
            self.state.terminal = Arc::new(Mutex::new(terminal));
            self.state.cached_size = props.config.size;
            self.state.scroll_position = 0;
            self.launched = false;
            self.invalid_size = invalid_size;
            self.error = error;
        }
        self.props = props.clone();
        if !props.show_scrollbar {
            self.scroll_dragging.store(false, Ordering::Release);
        }
        if self.layout.is_some() {
            self.apply_measured_size();
        }
        true
    }
    fn render(&self, _: &Self::Props, _: &Self::State) -> Element {
        if let Some(monitor) = &self.monitor {
            monitor.observe();
        }
        paint::render(self)
    }
    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut Self::State) -> bool {
        let changed = self.layout.is_none_or(|old| old != layout);
        self.layout = Some(layout);
        self.apply_measured_size() || changed
    }
    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut Self::State) {
        match event {
            LifecycleEvent::Mount => self.mounted = true,
            LifecycleEvent::Unmount => {
                self.mounted = false;
                if let Err(error) = self.stop() {
                    self.error = Some(error.to_string());
                }
            }
            _ => {}
        }
    }
    fn handle_event(
        &mut self,
        event: &Event,
        _: &mut Self::Props,
        _: &mut Self::State,
    ) -> EventResult {
        match event {
            Event::Key(key) => self.handle_key_event(key),
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Resize(resize) => self.handle_resize_event(resize),
            Event::Paste(paste) if self.focused.load(Ordering::Acquire) => {
                let bracketed = self
                    .state
                    .terminal
                    .lock()
                    .is_ok_and(|terminal| terminal.screen().input_modes().bracketed_paste);
                if bracketed {
                    if paste.content.len() > crate::terminal::pty::MAX_INPUT - 12 {
                        self.error = Some("Bracketed paste exceeds the 64 KiB input limit".into());
                        return EventResult::Consumed;
                    }
                    self.input(format!("\x1b[200~{}\x1b[201~", paste.content).as_bytes())
                } else {
                    self.input(paste.content.as_bytes())
                }
            }
            _ => EventResult::Ignored,
        }
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
    fn scrollbar_drag_ends_on_leave_blur_stop_and_hidden_track() {
        use crate::event::{
            hit::Bounds,
            types::{MouseButton, Position},
        };
        let mut widget = TerminalWidget::new(TerminalProps::default());
        widget.layout = Some(LayoutInfo::from_bounds(Bounds::new(0.0, 0.0, 8.0, 4.0)));
        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(&b"row\r\n".repeat(40));
        let down = MouseEvent::new(MouseEventKind::Down, Position::cell(7, 1))
            .with_button(MouseButton::Left);
        let drag = MouseEvent::new(MouseEventKind::Drag, Position::cell(7, 3))
            .with_button(MouseButton::Left);
        assert_eq!(widget.handle_mouse_event(&down), EventResult::Consumed);
        assert!(widget.state.scroll_position > 0);
        widget.handle_mouse_event(&MouseEvent::new(
            MouseEventKind::Leave,
            Position::cell(u16::MAX, u16::MAX),
        ));
        assert_eq!(widget.handle_mouse_event(&drag), EventResult::Ignored);
        widget.handle_mouse_event(&down);
        paint::render(&widget).focus.unwrap().on_blur.unwrap()();
        assert!(!widget.scroll_dragging.load(Ordering::Acquire));
        widget.handle_mouse_event(&down);
        widget.stop().unwrap();
        assert!(!widget.scroll_dragging.load(Ordering::Acquire));
        widget.handle_mouse_event(&down);
        let mut props = widget.props.clone();
        props.show_scrollbar = false;
        widget.update(&props, &mut TerminalState::default());
        assert!(!widget.scroll_dragging.load(Ordering::Acquire));
        assert_eq!(widget.handle_mouse_event(&drag), EventResult::Ignored);
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
        assert!(!state.terminal.lock().unwrap().is_running());
        assert!(!state.has_focus);
        assert_eq!(state.scroll_position, 0);
        assert_eq!(state.cached_size, (80, 24));
        assert!(state.needs_redraw);
    }

    #[test]
    fn test_terminal_widget_creation() {
        let props = TerminalProps::default();
        let widget = TerminalWidget::new(props.clone());

        assert_eq!(widget.props.title, props.title);
        assert!(!widget.is_running());
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

        widget
            .state
            .terminal
            .lock()
            .unwrap()
            .process_output(&b"line\r\n".repeat(40));
        // Move into retained history.
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
        assert!(!widget.is_running());
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
        assert!(result.is_err());
    }

    #[test]
    fn test_terminal_widget_send_input() {
        let mut widget = TerminalWidget::new(TerminalProps::default());

        // Test that send_input doesn't panic
        let result = widget.send_input(b"test data");

        // The result depends on the terminal implementation - just ensure it doesn't panic
        assert!(result.is_err());
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
}
