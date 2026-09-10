use std::time::Instant;

/// Base event trait that all events must implement
pub trait EventTrait: Clone + Send + Sync {
    /// Get the event type name
    fn event_type(&self) -> &'static str;

    /// Whether this event bubbles up the component tree
    fn bubbles(&self) -> bool {
        true
    }

    /// Whether this event can be cancelled
    fn cancelable(&self) -> bool {
        true
    }

    /// Get the timestamp when the event was created
    fn timestamp(&self) -> Instant;
}

/// Main event enum containing all possible events
#[derive(Clone, Debug)]
pub enum Event {
    /// Keyboard input event
    Key(KeyEvent),
    /// Mouse input event
    Mouse(MouseEvent),
    /// Terminal resize event
    Resize(ResizeEvent),
    /// Focus change event
    Focus(FocusEvent),
    /// Text paste event
    Paste(PasteEvent),
    /// Custom application-defined event
    Custom(CustomEvent),
}

impl Event {
    /// Native activation shared by builder and VDOM controls.
    pub(crate) fn activates_control(&self) -> bool {
        match self {
            Event::Key(key) => {
                key.kind == KeyEventKind::Press
                    && !key.repeat
                    && key.modifiers.is_empty()
                    && matches!(
                        key.code,
                        KeyCode::Enter | KeyCode::Space | KeyCode::Char(' ')
                    )
            }
            Event::Mouse(mouse) => {
                mouse.button == MouseButton::Left
                    && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click)
            }
            _ => false,
        }
    }

    /// Check if this is a keyboard event
    pub fn is_key(&self) -> bool {
        matches!(self, Event::Key(_))
    }

    /// Check if this is a mouse event
    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }

    /// Get the event as a keyboard event if it is one
    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(e) => Some(e),
            _ => None,
        }
    }

    /// Get the event as a mouse event if it is one
    pub fn as_mouse(&self) -> Option<&MouseEvent> {
        match self {
            Event::Mouse(e) => Some(e),
            _ => None,
        }
    }
}

/// Keyboard event
#[derive(Clone, Debug, PartialEq)]
pub struct KeyEvent {
    /// The key that was pressed
    pub code: KeyCode,
    /// Modifier keys held during the event
    pub modifiers: KeyModifiers,
    /// Type of key event (press, release, repeat)
    pub kind: KeyEventKind,
    /// Whether this is a repeated key press
    pub repeat: bool,
    /// When the event occurred
    pub timestamp: Instant,
}

impl KeyEvent {
    /// Create a new key event with the given key code
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            repeat: false,
            timestamp: Instant::now(),
        }
    }

    /// Set the modifier keys for this event
    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

    /// Set the event kind (press, release, repeat)
    pub fn with_kind(mut self, kind: KeyEventKind) -> Self {
        self.kind = kind;
        self
    }

    /// Convenience: does this key match (code, modifiers)?
    pub fn matches(&self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        self.code == code && self.modifiers == modifiers
    }

    /// Convenience: does this key match any of the provided patterns?
    pub fn matches_any(&self, patterns: &[(KeyCode, KeyModifiers)]) -> bool {
        patterns
            .iter()
            .any(|(c, m)| self.code == *c && self.modifiers == *m)
    }
}

/// Type of keyboard event
///
/// Distinguishes between key press, release, and repeat events.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyEventKind {
    /// Key was pressed down
    Press,
    /// Key was released
    Release,
    /// Key is being held down and repeating
    Repeat,
}

/// Keyboard key codes
///
/// Represents all possible keyboard keys that can be detected.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    /// Character key (letters, numbers, symbols)
    Char(char),

    // Function keys
    /// Function key (F1-F24)
    F(u8),

    // Navigation
    /// Up arrow key
    Up,
    /// Down arrow key
    Down,
    /// Left arrow key
    Left,
    /// Right arrow key
    Right,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,

    // Editing
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Insert key
    Insert,
    /// Enter/Return key
    Enter,
    /// Tab key
    Tab,
    /// Back tab key (Shift+Tab)
    BackTab,

    // Control
    /// Escape key
    Escape,
    /// Space bar
    Space,

    // Modifiers (when pressed alone)
    /// Caps Lock key
    CapsLock,
    /// Num Lock key
    NumLock,
    /// Scroll Lock key
    ScrollLock,

    // Media keys
    /// Media play key
    MediaPlay,
    /// Media pause key
    MediaPause,
    /// Media play/pause toggle key
    MediaPlayPause,
    /// Media stop key
    MediaStop,
    /// Media next track key
    MediaNext,
    /// Media previous track key
    MediaPrevious,

    // Special
    /// Null character
    Null,
    /// Unknown or unrecognized key
    Unknown,
}

/// Keyboard modifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyModifiers {
    /// Whether Shift key is pressed
    pub shift: bool,
    /// Whether Ctrl key is pressed
    pub ctrl: bool,
    /// Whether Alt key is pressed
    pub alt: bool,
    /// Whether Meta key is pressed (Super/Windows/Command key)
    pub meta: bool,
}

impl KeyModifiers {
    /// Create empty modifiers (no keys pressed)
    ///
    /// # Returns
    /// `KeyModifiers` with all modifier keys disabled
    pub fn empty() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    /// Create modifiers with only shift pressed
    ///
    /// # Returns
    /// `KeyModifiers` with shift enabled and other modifiers disabled
    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Self::empty()
        }
    }

    /// Create modifiers with only ctrl pressed
    ///
    /// # Returns
    /// `KeyModifiers` with ctrl enabled and other modifiers disabled
    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::empty()
        }
    }

    /// Create modifiers with only alt pressed
    ///
    /// # Returns
    /// `KeyModifiers` with alt enabled and other modifiers disabled
    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Self::empty()
        }
    }

    /// Check if no modifier keys are pressed
    ///
    /// # Returns
    /// `true` if no modifier keys are active, `false` otherwise
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

/// Mouse event
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    /// Type of mouse event (click, move, scroll, etc.)
    pub kind: MouseEventKind,
    /// Which mouse button was involved
    pub button: MouseButton,
    /// Position where the event occurred
    pub position: Position,
    /// Keyboard modifiers held during the event
    pub modifiers: KeyModifiers,
    /// When the event occurred
    pub timestamp: Instant,
    /// Wheel event data (only present for wheel events)
    pub wheel: Option<WheelEvent>,
}

impl MouseEvent {
    /// Create a new mouse event
    ///
    /// # Arguments
    /// * `kind` - The type of mouse event
    /// * `position` - Where the event occurred
    ///
    /// # Returns
    /// A new `MouseEvent` with default button and modifiers
    pub fn new(kind: MouseEventKind, position: Position) -> Self {
        Self {
            kind,
            button: MouseButton::None,
            position,
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
            wheel: None,
        }
    }

    /// Create a new wheel mouse event
    ///
    /// # Arguments
    /// * `position` - Where the event occurred
    /// * `wheel` - Wheel event data
    ///
    /// # Returns
    /// A new `MouseEvent` for wheel scrolling
    pub fn wheel(position: Position, wheel: WheelEvent) -> Self {
        Self {
            kind: MouseEventKind::Wheel,
            button: MouseButton::None,
            position,
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
            wheel: Some(wheel),
        }
    }

    /// Set the mouse button for this event
    ///
    /// # Arguments
    /// * `button` - The mouse button that was involved
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_button(mut self, button: MouseButton) -> Self {
        self.button = button;
        self
    }

    /// Set the keyboard modifiers for this event
    ///
    /// # Arguments
    /// * `modifiers` - The keyboard modifiers held during the event
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
}

/// Type of mouse event
///
/// Represents different kinds of mouse interactions that can occur
/// in the terminal. These events capture button presses, movement,
/// and wheel scrolling.
#[derive(Clone, Debug, PartialEq)]
pub enum MouseEventKind {
    /// Mouse button was pressed down
    Down,
    /// Mouse button was released
    Up,
    /// Single click (press and release)
    Click,
    /// Double click detected
    DoubleClick,
    /// Triple click detected
    TripleClick,
    /// Mouse cursor moved without buttons pressed
    Move,
    /// Mouse moved while button held (dragging)
    Drag,
    /// Mouse cursor entered the terminal window or a routed node's painted bounds.
    Enter,
    /// Mouse cursor left the terminal window or a routed node's painted bounds.
    /// Component-local leave positions may be outside the control; do not use
    /// them to activate or select content.
    Leave,
    /// Mouse wheel was scrolled
    Wheel,
}

/// Mouse button identifier
///
/// Represents which mouse button is involved in an event.
/// Supports standard buttons plus extended buttons for
/// mice with additional controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    /// No button or button released
    None,
    /// Primary (left) mouse button
    Left,
    /// Secondary (right) mouse button  
    Right,
    /// Middle mouse button (wheel click)
    Middle,
    /// Back/previous button (button 4)
    Back,
    /// Forward/next button (button 5)
    Forward,
    /// Other numbered button
    Other(u8),
}

/// Position in terminal cells or pixels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    /// Position in terminal cells (column, row)
    Cell {
        /// Column position (0-based)
        x: u16,
        /// Row position (0-based)
        y: u16,
    },
    /// Position in pixels (for terminals that support pixel-level mouse)
    Pixel {
        /// X coordinate in pixels
        x: u32,
        /// Y coordinate in pixels
        y: u32,
    },
}

impl Position {
    /// Create a cell-based position
    ///
    /// # Arguments
    /// * `x` - Column position
    /// * `y` - Row position
    ///
    /// # Returns
    /// A `Position::Cell` variant
    pub fn cell(x: u16, y: u16) -> Self {
        Position::Cell { x, y }
    }

    /// Create a pixel-based position
    ///
    /// # Arguments
    /// * `x` - Horizontal pixel position
    /// * `y` - Vertical pixel position
    ///
    /// # Returns
    /// A `Position::Pixel` variant
    pub fn pixel(x: u32, y: u32) -> Self {
        Position::Pixel { x, y }
    }

    /// Get the x coordinate as a u32
    ///
    /// # Returns
    /// The x coordinate, converted to u32 if needed
    pub fn x(&self) -> u32 {
        match self {
            Position::Cell { x, .. } => *x as u32,
            Position::Pixel { x, .. } => *x,
        }
    }

    /// Get the y coordinate as a u32
    ///
    /// # Returns
    /// The y coordinate, converted to u32 if needed
    pub fn y(&self) -> u32 {
        match self {
            Position::Cell { y, .. } => *y as u32,
            Position::Pixel { y, .. } => *y,
        }
    }
}

/// Mouse wheel event information
///
/// Contains details about a mouse wheel scrolling event,
/// including the scroll amount and the phase of the gesture.
#[derive(Clone, Debug, PartialEq)]
pub struct WheelEvent {
    /// Amount and direction of scrolling
    pub delta: WheelDelta,
    /// Current phase of the scroll gesture
    pub phase: WheelPhase,
}

/// Mouse wheel scrolling delta
///
/// Represents the amount and type of scrolling that occurred.
/// Different terminals and platforms may report wheel events
/// in lines or pixels.
#[derive(Clone, Debug, PartialEq)]
pub enum WheelDelta {
    /// Scrolling measured in text lines
    Lines {
        /// Horizontal scroll amount (negative = left, positive = right)
        x: f32,
        /// Vertical scroll amount (negative = up, positive = down)
        y: f32,
    },
    /// Scrolling measured in pixels
    Pixels {
        /// Horizontal scroll amount in pixels
        x: f32,
        /// Vertical scroll amount in pixels
        y: f32,
    },
}

/// Mouse wheel scrolling phase
///
/// Tracks the lifecycle of a scrolling gesture, particularly
/// useful for trackpad scrolling which has distinct phases.
#[derive(Clone, Debug, PartialEq)]
pub enum WheelPhase {
    /// Scrolling gesture has started
    Started,
    /// Scrolling is ongoing
    Changed,
    /// Scrolling gesture has ended
    Ended,
}

/// Terminal resize event
///
/// Fired when the terminal window is resized. Contains both
/// character cell dimensions and optional pixel dimensions.
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeEvent {
    /// New width in character cells
    pub width: u16,
    /// New height in character cells
    pub height: u16,
    /// New width in pixels (if supported)
    pub pixel_width: Option<u32>,
    /// New height in pixels (if supported)
    pub pixel_height: Option<u32>,
    /// When the resize occurred
    pub timestamp: Instant,
}

impl ResizeEvent {
    /// Create a new resize event with character cell dimensions
    ///
    /// # Arguments
    /// * `width` - New width in character cells
    /// * `height` - New height in character cells
    ///
    /// # Returns
    /// A new `ResizeEvent` with the current timestamp
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixel_width: None,
            pixel_height: None,
            timestamp: Instant::now(),
        }
    }

    /// Add pixel dimensions to the resize event
    ///
    /// # Arguments
    /// * `pixel_width` - Width in pixels
    /// * `pixel_height` - Height in pixels
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_pixels(mut self, pixel_width: u32, pixel_height: u32) -> Self {
        self.pixel_width = Some(pixel_width);
        self.pixel_height = Some(pixel_height);
        self
    }
}

/// Focus event
///
/// Represents changes in focus state for the terminal window
/// or UI elements within the application.
#[derive(Clone, Debug, PartialEq)]
pub struct FocusEvent {
    /// Type of focus change
    pub kind: FocusEventKind,
    /// When the focus change occurred
    pub timestamp: Instant,
}

/// Type of focus change
///
/// Distinguishes between different kinds of focus events,
/// including gaining/losing focus and navigation between elements.
#[derive(Clone, Debug, PartialEq)]
pub enum FocusEventKind {
    /// Terminal window or element gained focus
    Gained,
    /// Terminal window or element lost focus
    Lost,
    /// Focus moved to next element (Tab navigation)
    Next,
    /// Focus moved to previous element (Shift+Tab navigation)
    Previous,
    /// Focus moved up (Arrow key navigation)
    Up,
    /// Focus moved down (Arrow key navigation)
    Down,
    /// Focus moved left (Arrow key navigation)
    Left,
    /// Focus moved right (Arrow key navigation)
    Right,
}

/// Paste event for bracketed paste mode
///
/// Fired when text is pasted into the terminal while
/// bracketed paste mode is enabled. This allows proper
/// handling of multi-line pastes.
#[derive(Clone, Debug, PartialEq)]
pub struct PasteEvent {
    /// The pasted text content
    pub content: String,
    /// When the paste occurred
    pub timestamp: Instant,
}

impl PasteEvent {
    /// Create a new paste event
    ///
    /// # Arguments
    /// * `content` - The pasted text
    ///
    /// # Returns
    /// A new `PasteEvent` with the current timestamp
    pub fn new(content: String) -> Self {
        Self {
            content,
            timestamp: Instant::now(),
        }
    }
}

/// Custom user-defined event
#[derive(Clone, Debug)]
pub struct CustomEvent {
    /// Name identifier for the custom event
    pub name: String,
    /// Binary data payload for the event
    pub data: Vec<u8>,
    /// Timestamp when the event was created
    pub timestamp: Instant,
}

impl CustomEvent {
    /// Create a new custom event
    ///
    /// # Arguments
    /// * `name` - Name identifier for the event
    /// * `data` - Binary data payload
    ///
    /// # Returns
    /// A new `CustomEvent` with the current timestamp
    pub fn new(name: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            data,
            timestamp: Instant::now(),
        }
    }
}

// Implement EventTrait for all event types
impl EventTrait for KeyEvent {
    fn event_type(&self) -> &'static str {
        "key"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

impl EventTrait for MouseEvent {
    fn event_type(&self) -> &'static str {
        "mouse"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

impl EventTrait for ResizeEvent {
    fn event_type(&self) -> &'static str {
        "resize"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
    fn bubbles(&self) -> bool {
        false
    }
}

impl EventTrait for FocusEvent {
    fn event_type(&self) -> &'static str {
        "focus"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
    fn bubbles(&self) -> bool {
        false
    }
}

impl EventTrait for PasteEvent {
    fn event_type(&self) -> &'static str {
        "paste"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

impl EventTrait for CustomEvent {
    fn event_type(&self) -> &'static str {
        "custom"
    }
    fn timestamp(&self) -> Instant {
        self.timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent::new(KeyCode::Char('a'));
        assert_eq!(event.code, KeyCode::Char('a'));
        assert_eq!(event.modifiers, KeyModifiers::empty());
        assert_eq!(event.kind, KeyEventKind::Press);
        assert!(!event.repeat);
    }

    #[test]
    fn test_key_event_builder() {
        let event = KeyEvent::new(KeyCode::Enter)
            .with_modifiers(KeyModifiers::ctrl())
            .with_kind(KeyEventKind::Release);

        assert_eq!(event.code, KeyCode::Enter);
        assert!(event.modifiers.ctrl);
        assert_eq!(event.kind, KeyEventKind::Release);
    }

    #[test]
    fn test_key_event_matches() {
        let event = KeyEvent::new(KeyCode::Char('s')).with_modifiers(KeyModifiers::ctrl());

        assert!(event.matches(KeyCode::Char('s'), KeyModifiers::ctrl()));
        assert!(!event.matches(KeyCode::Char('s'), KeyModifiers::empty()));
        assert!(!event.matches(KeyCode::Char('a'), KeyModifiers::ctrl()));
    }

    #[test]
    fn test_key_event_matches_any() {
        let event = KeyEvent::new(KeyCode::Char('c')).with_modifiers(KeyModifiers::ctrl());

        let patterns = [
            (KeyCode::Char('a'), KeyModifiers::ctrl()),
            (KeyCode::Char('c'), KeyModifiers::ctrl()),
            (KeyCode::Char('v'), KeyModifiers::ctrl()),
        ];

        assert!(event.matches_any(&patterns));

        let no_match_patterns = [
            (KeyCode::Char('a'), KeyModifiers::ctrl()),
            (KeyCode::Char('v'), KeyModifiers::ctrl()),
        ];

        assert!(!event.matches_any(&no_match_patterns));
    }

    #[test]
    fn test_key_modifiers() {
        let empty = KeyModifiers::empty();
        assert!(empty.is_empty());
        assert!(!empty.shift && !empty.ctrl && !empty.alt && !empty.meta);

        let shift = KeyModifiers::shift();
        assert!(!shift.is_empty());
        assert!(shift.shift && !shift.ctrl && !shift.alt && !shift.meta);

        let ctrl = KeyModifiers::ctrl();
        assert!(!ctrl.is_empty());
        assert!(!ctrl.shift && ctrl.ctrl && !ctrl.alt && !ctrl.meta);

        let alt = KeyModifiers::alt();
        assert!(!alt.is_empty());
        assert!(!alt.shift && !alt.ctrl && alt.alt && !alt.meta);
    }

    #[test]
    fn test_key_codes() {
        // Test character keys
        assert_eq!(KeyCode::Char('a'), KeyCode::Char('a'));
        assert_ne!(KeyCode::Char('a'), KeyCode::Char('b'));

        // Test function keys
        assert_eq!(KeyCode::F(1), KeyCode::F(1));
        assert_ne!(KeyCode::F(1), KeyCode::F(2));

        // Test navigation keys
        assert_ne!(KeyCode::Up, KeyCode::Down);
        assert_ne!(KeyCode::Left, KeyCode::Right);

        // Test editing keys
        assert_ne!(KeyCode::Backspace, KeyCode::Delete);
        assert_ne!(KeyCode::Enter, KeyCode::Tab);
    }

    #[test]
    fn test_mouse_event_creation() {
        let position = Position::cell(10, 5);
        let event = MouseEvent::new(MouseEventKind::Click, position);

        assert_eq!(event.kind, MouseEventKind::Click);
        assert_eq!(event.position, position);
        assert_eq!(event.button, MouseButton::None);
        assert_eq!(event.modifiers, KeyModifiers::empty());
    }

    #[test]
    fn test_mouse_event_builder() {
        let position = Position::pixel(100, 200);
        let event = MouseEvent::new(MouseEventKind::Down, position)
            .with_button(MouseButton::Left)
            .with_modifiers(KeyModifiers::shift());

        assert_eq!(event.kind, MouseEventKind::Down);
        assert_eq!(event.button, MouseButton::Left);
        assert!(event.modifiers.shift);
    }

    #[test]
    fn test_position() {
        let cell_pos = Position::cell(5, 10);
        assert_eq!(cell_pos.x(), 5);
        assert_eq!(cell_pos.y(), 10);

        let pixel_pos = Position::pixel(150, 300);
        assert_eq!(pixel_pos.x(), 150);
        assert_eq!(pixel_pos.y(), 300);

        // Test convenience constructors
        assert_eq!(Position::cell(1, 2), Position::Cell { x: 1, y: 2 });
        assert_eq!(Position::pixel(3, 4), Position::Pixel { x: 3, y: 4 });
    }

    #[test]
    fn test_mouse_buttons() {
        assert_ne!(MouseButton::Left, MouseButton::Right);
        assert_ne!(MouseButton::Middle, MouseButton::Back);
        assert_eq!(MouseButton::Other(7), MouseButton::Other(7));
        assert_ne!(MouseButton::Other(7), MouseButton::Other(8));
    }

    #[test]
    fn test_resize_event() {
        let event = ResizeEvent::new(80, 24);
        assert_eq!(event.width, 80);
        assert_eq!(event.height, 24);
        assert!(event.pixel_width.is_none());
        assert!(event.pixel_height.is_none());

        let event_with_pixels = event.with_pixels(1920, 1080);
        assert_eq!(event_with_pixels.pixel_width, Some(1920));
        assert_eq!(event_with_pixels.pixel_height, Some(1080));
    }

    #[test]
    fn test_focus_event() {
        let event = FocusEvent {
            kind: FocusEventKind::Gained,
            timestamp: Instant::now(),
        };
        assert_eq!(event.kind, FocusEventKind::Gained);

        // Test all focus event kinds
        assert_ne!(FocusEventKind::Gained, FocusEventKind::Lost);
        assert_ne!(FocusEventKind::Next, FocusEventKind::Previous);
    }

    #[test]
    fn test_paste_event() {
        let content = "Hello, world!".to_string();
        let event = PasteEvent::new(content.clone());
        assert_eq!(event.content, content);
    }

    #[test]
    fn test_custom_event() {
        let name = "user-action";
        let data = vec![1, 2, 3, 4];
        let event = CustomEvent::new(name, data.clone());
        assert_eq!(event.name, name);
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_wheel_event() {
        let wheel = WheelEvent {
            delta: WheelDelta::Lines { x: 0.0, y: 1.0 },
            phase: WheelPhase::Started,
        };
        assert_eq!(wheel.phase, WheelPhase::Started);

        if let WheelDelta::Lines { x, y } = wheel.delta {
            assert_eq!(x, 0.0);
            assert_eq!(y, 1.0);
        } else {
            panic!("Expected Lines delta");
        }

        // Test pixel delta
        let pixel_wheel = WheelEvent {
            delta: WheelDelta::Pixels { x: 10.5, y: -5.2 },
            phase: WheelPhase::Changed,
        };

        if let WheelDelta::Pixels { x, y } = pixel_wheel.delta {
            assert_eq!(x, 10.5);
            assert_eq!(y, -5.2);
        } else {
            panic!("Expected Pixels delta");
        }
    }

    #[test]
    fn test_event_trait_implementations() {
        let key_event = KeyEvent::new(KeyCode::Char('a'));
        assert_eq!(key_event.event_type(), "key");
        assert!(key_event.bubbles()); // Default is true

        let mouse_event = MouseEvent::new(MouseEventKind::Click, Position::cell(0, 0));
        assert_eq!(mouse_event.event_type(), "mouse");
        assert!(mouse_event.bubbles()); // Default is true

        let resize_event = ResizeEvent::new(80, 24);
        assert_eq!(resize_event.event_type(), "resize");
        assert!(!resize_event.bubbles()); // Resize events don't bubble

        let focus_event = FocusEvent {
            kind: FocusEventKind::Gained,
            timestamp: Instant::now(),
        };
        assert_eq!(focus_event.event_type(), "focus");
        assert!(!focus_event.bubbles()); // Focus events don't bubble

        let paste_event = PasteEvent::new("text".to_string());
        assert_eq!(paste_event.event_type(), "paste");
        assert!(paste_event.bubbles()); // Default is true

        let custom_event = CustomEvent::new("test", vec![]);
        assert_eq!(custom_event.event_type(), "custom");
        assert!(custom_event.bubbles()); // Default is true
    }

    #[test]
    fn test_key_event_kinds() {
        assert_ne!(KeyEventKind::Press, KeyEventKind::Release);
        assert_ne!(KeyEventKind::Press, KeyEventKind::Repeat);
        assert_ne!(KeyEventKind::Release, KeyEventKind::Repeat);
    }

    #[test]
    fn test_mouse_event_kinds() {
        let kinds = [
            MouseEventKind::Down,
            MouseEventKind::Up,
            MouseEventKind::Click,
            MouseEventKind::DoubleClick,
            MouseEventKind::TripleClick,
            MouseEventKind::Move,
            MouseEventKind::Drag,
            MouseEventKind::Enter,
            MouseEventKind::Leave,
            MouseEventKind::Wheel,
        ];

        // All should be different
        for (i, kind1) in kinds.iter().enumerate() {
            for (j, kind2) in kinds.iter().enumerate() {
                if i != j {
                    assert_ne!(kind1, kind2);
                }
            }
        }
    }

    #[test]
    fn test_wheel_phases() {
        assert_ne!(WheelPhase::Started, WheelPhase::Changed);
        assert_ne!(WheelPhase::Changed, WheelPhase::Ended);
        assert_ne!(WheelPhase::Started, WheelPhase::Ended);
    }

    #[test]
    fn test_event_timestamps() {
        let before = Instant::now();

        let key_event = KeyEvent::new(KeyCode::Space);
        let mouse_event = MouseEvent::new(MouseEventKind::Move, Position::cell(1, 1));
        let resize_event = ResizeEvent::new(100, 50);
        let paste_event = PasteEvent::new("content".to_string());
        let custom_event = CustomEvent::new("test", vec![]);

        let after = Instant::now();

        // All timestamps should be between before and after
        assert!(key_event.timestamp >= before && key_event.timestamp <= after);
        assert!(mouse_event.timestamp >= before && mouse_event.timestamp <= after);
        assert!(resize_event.timestamp >= before && resize_event.timestamp <= after);
        assert!(paste_event.timestamp >= before && paste_event.timestamp <= after);
        assert!(custom_event.timestamp >= before && custom_event.timestamp <= after);
    }

    #[test]
    fn test_complex_key_combinations() {
        // Test complex modifier combinations
        let complex_modifiers = KeyModifiers {
            shift: true,
            ctrl: true,
            alt: false,
            meta: true,
        };

        assert!(!complex_modifiers.is_empty());

        let event = KeyEvent::new(KeyCode::F(12)).with_modifiers(complex_modifiers);

        assert!(event.matches(KeyCode::F(12), complex_modifiers));
        assert!(!event.matches(KeyCode::F(12), KeyModifiers::ctrl()));
    }

    #[test]
    fn test_position_conversions() {
        // Test that cell positions convert correctly
        let cell = Position::cell(u16::MAX, u16::MAX);
        assert_eq!(cell.x(), u16::MAX as u32);
        assert_eq!(cell.y(), u16::MAX as u32);

        // Test that pixel positions work with large values
        let pixel = Position::pixel(u32::MAX, u32::MAX);
        assert_eq!(pixel.x(), u32::MAX);
        assert_eq!(pixel.y(), u32::MAX);
    }
}
