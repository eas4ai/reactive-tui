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
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(ResizeEvent),
    Focus(FocusEvent),
    Paste(PasteEvent),
    Custom(CustomEvent),
}

impl Event {
    pub fn is_key(&self) -> bool {
        matches!(self, Event::Key(_))
    }

    pub fn is_mouse(&self) -> bool {
        matches!(self, Event::Mouse(_))
    }

    pub fn as_key(&self) -> Option<&KeyEvent> {
        match self {
            Event::Key(e) => Some(e),
            _ => None,
        }
    }

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
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
    pub repeat: bool,
    pub timestamp: Instant,
}

impl KeyEvent {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            repeat: false,
            timestamp: Instant::now(),
        }
    }

    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }

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

#[derive(Clone, Debug, PartialEq)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    Char(char),

    // Function keys
    F(u8), // F1-F24

    // Navigation
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,

    // Editing
    Backspace,
    Delete,
    Insert,
    Enter,
    Tab,
    BackTab, // Shift+Tab

    // Control
    Escape,
    Space,

    // Modifiers (when pressed alone)
    CapsLock,
    NumLock,
    ScrollLock,

    // Media keys
    MediaPlay,
    MediaPause,
    MediaPlayPause,
    MediaStop,
    MediaNext,
    MediaPrevious,

    // Special
    Null,
    Unknown,
}

/// Keyboard modifiers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool, // Super/Windows/Command key
}

impl KeyModifiers {
    pub fn empty() -> Self {
        Self {
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn shift() -> Self {
        Self {
            shift: true,
            ..Self::empty()
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            ..Self::empty()
        }
    }

    pub fn alt() -> Self {
        Self {
            alt: true,
            ..Self::empty()
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }
}

/// Mouse event
#[derive(Clone, Debug, PartialEq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub button: MouseButton,
    pub position: Position,
    pub modifiers: KeyModifiers,
    pub timestamp: Instant,
}

impl MouseEvent {
    pub fn new(kind: MouseEventKind, position: Position) -> Self {
        Self {
            kind,
            button: MouseButton::None,
            position,
            modifiers: KeyModifiers::empty(),
            timestamp: Instant::now(),
        }
    }

    pub fn with_button(mut self, button: MouseButton) -> Self {
        self.button = button;
        self
    }

    pub fn with_modifiers(mut self, modifiers: KeyModifiers) -> Self {
        self.modifiers = modifiers;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MouseEventKind {
    Down,
    Up,
    Click,
    DoubleClick,
    TripleClick,
    Move,
    Drag,
    Enter,
    Leave,
    Wheel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseButton {
    None,
    Left,
    Right,
    Middle,
    Back,    // Button 4
    Forward, // Button 5
    Other(u8),
}

/// Position in terminal cells or pixels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Position {
    /// Position in terminal cells (column, row)
    Cell { x: u16, y: u16 },
    /// Position in pixels (for terminals that support pixel-level mouse)
    Pixel { x: u32, y: u32 },
}

impl Position {
    pub fn cell(x: u16, y: u16) -> Self {
        Position::Cell { x, y }
    }

    pub fn pixel(x: u32, y: u32) -> Self {
        Position::Pixel { x, y }
    }

    pub fn x(&self) -> u32 {
        match self {
            Position::Cell { x, .. } => *x as u32,
            Position::Pixel { x, .. } => *x,
        }
    }

    pub fn y(&self) -> u32 {
        match self {
            Position::Cell { y, .. } => *y as u32,
            Position::Pixel { y, .. } => *y,
        }
    }
}

/// Mouse wheel event information
#[derive(Clone, Debug, PartialEq)]
pub struct WheelEvent {
    pub delta: WheelDelta,
    pub phase: WheelPhase,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WheelDelta {
    Lines { x: f32, y: f32 },
    Pixels { x: f32, y: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum WheelPhase {
    Started,
    Changed,
    Ended,
}

/// Terminal resize event
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeEvent {
    pub width: u16,
    pub height: u16,
    pub pixel_width: Option<u32>,
    pub pixel_height: Option<u32>,
    pub timestamp: Instant,
}

impl ResizeEvent {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            pixel_width: None,
            pixel_height: None,
            timestamp: Instant::now(),
        }
    }

    pub fn with_pixels(mut self, pixel_width: u32, pixel_height: u32) -> Self {
        self.pixel_width = Some(pixel_width);
        self.pixel_height = Some(pixel_height);
        self
    }
}

/// Focus event
#[derive(Clone, Debug, PartialEq)]
pub struct FocusEvent {
    pub kind: FocusEventKind,
    pub timestamp: Instant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FocusEventKind {
    Gained,
    Lost,
    /// Focus moved to next element
    Next,
    /// Focus moved to previous element
    Previous,
}

/// Paste event for bracketed paste mode
#[derive(Clone, Debug, PartialEq)]
pub struct PasteEvent {
    pub content: String,
    pub timestamp: Instant,
}

impl PasteEvent {
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
    pub name: String,
    pub data: Vec<u8>,
    pub timestamp: Instant,
}

impl CustomEvent {
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
