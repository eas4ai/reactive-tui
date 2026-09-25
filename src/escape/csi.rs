/// Control Sequence Introducer (CSI) sequences
///
/// Represents all possible CSI escape sequences that can be parsed.
/// CSI sequences are used for cursor control, text formatting,
/// screen manipulation, and other terminal operations.
#[derive(Debug, Clone, PartialEq)]
pub enum CSIAction {
    // Cursor Movement
    /// Move cursor up by n lines
    CursorUp(u16),
    /// Move cursor down by n lines
    CursorDown(u16),
    /// Move cursor forward by n columns
    CursorForward(u16),
    /// Move cursor backward by n columns
    CursorBack(u16),
    /// Move cursor to beginning of line n lines down
    CursorNextLine(u16),
    /// Move cursor to beginning of line n lines up
    CursorPreviousLine(u16),
    /// Move cursor to specific position (1-based)
    CursorPosition {
        /// Row position (1-based)
        row: u16,
        /// Column position (1-based)
        col: u16,
    },
    /// Move cursor to column n
    CursorHorizontalAbsolute(u16),

    // Cursor Visibility
    /// Make cursor visible
    ShowCursor,
    /// Make cursor invisible
    HideCursor,

    // Screen/Line Operations
    /// Erase display with specified mode
    EraseDisplay(EraseMode),
    /// Erase line with specified mode
    EraseLine(EraseMode),
    /// Insert n blank lines at cursor
    InsertLines(u16),
    /// Delete n lines at cursor
    DeleteLines(u16),
    /// Insert n blank characters at cursor
    InsertChars(u16),
    /// Delete n characters at cursor
    DeleteChars(u16),
    /// Scroll display up by n lines
    ScrollUp(u16),
    /// Scroll display down by n lines
    ScrollDown(u16),

    // Text Attributes (SGR - Select Graphic Rendition)
    /// Set text attributes and colors
    SetGraphicsMode(Vec<SGRAttribute>),

    // Mouse Events
    /// Mouse event from terminal
    MouseEvent(MouseEvent),

    // Device Status
    /// Request device status report
    DeviceStatusReport,
    /// Request cursor position report
    ReportCursorPosition,

    // Modes
    /// Set terminal modes
    SetMode(Vec<Mode>),
    /// Reset terminal modes
    ResetMode(Vec<Mode>),

    // Other
    /// Set scrolling region
    SetScrollRegion {
        /// Top line of scroll region (1-based)
        top: u16,
        /// Bottom line of scroll region (1-based)
        bottom: u16,
    },
    /// Save current cursor position
    SaveCursor,
    /// Restore saved cursor position
    RestoreCursor,
}

/// Erase mode for display and line operations
///
/// Specifies what portion of the display or line should be erased.
#[derive(Debug, Clone, PartialEq)]
pub enum EraseMode {
    /// Erase from cursor to end of line/display
    ToEnd,
    /// Erase from beginning to cursor
    ToBeginning,
    /// Erase entire line/display
    All,
    /// Erase entire display including scrollback buffer
    Scrollback,
}

/// Select Graphic Rendition (SGR) text attributes
///
/// Text formatting and color attributes that can be applied
/// to terminal text output.
#[derive(Debug, Clone, PartialEq)]
pub enum SGRAttribute {
    /// Reset all attributes to default
    Reset,
    /// Bold or increased intensity
    Bold,
    /// Dim or decreased intensity
    Dim,
    /// Italic text
    Italic,
    /// Underlined text
    Underline,
    /// Blinking text
    Blink,
    /// Reverse video (swap foreground/background)
    Reverse,
    /// Hidden/invisible text
    Hidden,
    /// Strikethrough text
    Strikethrough,

    // Foreground colors
    /// Set foreground text color
    ForegroundColor(Color),
    /// Reset foreground to default color
    ForegroundDefault,

    // Background colors
    /// Set background color
    BackgroundColor(Color),
    /// Reset background to default color
    BackgroundDefault,

    // Underline colors
    /// Set underline color
    UnderlineColor(Color),
    /// Reset underline to default color
    UnderlineDefault,

    // Extended attributes
    /// Double underline
    DoubleUnderline,
    /// Curly/wavy underline
    CurlyUnderline,
    /// Dotted underline
    DottedUnderline,
    /// Dashed underline
    DashedUnderline,
    /// Remove underline
    NoUnderline,

    // Reset individual attributes
    /// Remove bold/dim
    NoBold,
    /// Remove italic
    NoItalic,
    /// Remove reverse video
    NoReverse,
    /// Remove strikethrough
    NoStrikethrough,
    /// Remove blink
    NoBlink,
}

/// Terminal color representation
///
/// Supports basic 16 colors, 256-color palette, and true RGB colors.
#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    // Basic 16 colors
    /// Black (color 0)
    Black,
    /// Red (color 1)
    Red,
    /// Green (color 2)
    Green,
    /// Yellow (color 3)
    Yellow,
    /// Blue (color 4)
    Blue,
    /// Magenta (color 5)
    Magenta,
    /// Cyan (color 6)
    Cyan,
    /// White (color 7)
    White,
    /// Bright black/gray (color 8)
    BrightBlack,
    /// Bright red (color 9)
    BrightRed,
    /// Bright green (color 10)
    BrightGreen,
    /// Bright yellow (color 11)
    BrightYellow,
    /// Bright blue (color 12)
    BrightBlue,
    /// Bright magenta (color 13)
    BrightMagenta,
    /// Bright cyan (color 14)
    BrightCyan,
    /// Bright white (color 15)
    BrightWhite,

    // Extended colors
    /// 256-color palette index
    Indexed(u8),
    /// True 24-bit RGB color
    RGB {
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
}

/// Mouse event information from CSI sequences
///
/// Contains details about mouse interactions reported
/// through CSI mouse tracking sequences.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    /// Which button was pressed/released
    pub button: MouseButton,
    /// Keyboard modifiers held
    pub modifiers: MouseModifiers,
    /// X coordinate (1-based)
    pub x: u16,
    /// Y coordinate (1-based)
    pub y: u16,
    /// Mouse tracking mode
    pub mode: MouseMode,
}

/// Mouse button for CSI mouse events
///
/// Represents which mouse button is involved in a CSI mouse event.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button
    Middle,
    /// Right mouse button
    Right,
    /// Mouse wheel scrolled up
    WheelUp,
    /// Mouse wheel scrolled down
    WheelDown,
    /// Extended mouse button by number
    Button(u8),
    /// Mouse motion without button press
    Motion,
    /// Button was released
    Release,
}

/// Keyboard modifiers for mouse events
///
/// Tracks which modifier keys were held during a mouse event.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseModifiers {
    /// Shift key was held
    pub shift: bool,
    /// Alt/Option key was held
    pub alt: bool,
    /// Control key was held
    pub ctrl: bool,
}

/// Mouse tracking mode
///
/// Different protocols for reporting mouse events.
#[derive(Debug, Clone, PartialEq)]
pub enum MouseMode {
    /// X10 compatibility mode
    Normal,
    /// Button press/release events
    ButtonEvent,
    /// Any mouse event including movement
    AnyEvent,
    /// SGR extended mode (supports coordinates > 223)
    SGR,
    /// Pixel-level precision mouse tracking
    Pixel,
}

/// Terminal mode settings
#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    // ANSI modes
    /// Keyboard action mode
    KeyboardAction,
    /// Insert mode (vs replace mode)
    Insert,
    /// Send/receive mode
    SendReceive,
    /// Automatic newline mode
    AutomaticNewline,

    // DEC Private modes
    /// Application cursor keys mode
    ApplicationCursorKeys,
    /// 132 column mode
    DECCOLM,
    /// Smooth scroll mode
    SmoothScroll,
    /// Reverse video mode
    ReverseVideo,
    /// Origin mode (relative to scroll region)
    Origin,
    /// Auto wrap mode
    AutoWrap,
    /// Auto repeat mode
    AutoRepeat,
    /// Interlace mode
    Interlace,

    // Mouse modes
    /// X10 compatibility mouse mode
    MouseX10,
    /// VT200 mouse mode
    MouseVT200,
    /// VT200 mouse mode with highlighting
    MouseVT200Highlight,
    /// Button event mouse mode
    MouseButtonEvent,
    /// Any event mouse mode (including movement)
    MouseAnyEvent,
    /// Focus event mouse mode
    MouseFocusEvent,
    /// SGR extended mouse mode
    MouseSGR,
    /// Pixel-level mouse mode
    MousePixel,

    // Other modes
    /// Alternate screen buffer mode
    AlternateScreen,
    /// Bracketed paste mode
    BracketedPaste,
    /// Show cursor mode
    ShowCursor,

    /// Unknown mode (for forward compatibility)
    Unknown(u16),
}

impl CSIAction {
    /// Parse a CSI sequence from its components
    pub fn parse(params: &[u16], intermediates: &[u8], final_byte: u8) -> Option<Self> {
        // Handle private sequences (starting with '?')
        let is_private = intermediates.contains(&b'?');

        match final_byte {
            // Cursor movement
            b'A' => Some(CSIAction::CursorUp(params.first().copied().unwrap_or(1))),
            b'B' => Some(CSIAction::CursorDown(params.first().copied().unwrap_or(1))),
            b'C' => Some(CSIAction::CursorForward(
                params.first().copied().unwrap_or(1),
            )),
            b'D' => Some(CSIAction::CursorBack(params.first().copied().unwrap_or(1))),
            b'E' => Some(CSIAction::CursorNextLine(
                params.first().copied().unwrap_or(1),
            )),
            b'F' => Some(CSIAction::CursorPreviousLine(
                params.first().copied().unwrap_or(1),
            )),
            b'G' => Some(CSIAction::CursorHorizontalAbsolute(
                params.first().copied().unwrap_or(1),
            )),
            b'H' | b'f' => {
                let row = params.first().copied().unwrap_or(1);
                let col = params.get(1).copied().unwrap_or(1);
                Some(CSIAction::CursorPosition { row, col })
            }

            // Erase operations
            b'J' => {
                let mode = match params.first().copied().unwrap_or(0) {
                    0 => EraseMode::ToEnd,
                    1 => EraseMode::ToBeginning,
                    2 => EraseMode::All,
                    3 => EraseMode::Scrollback,
                    _ => return None,
                };
                Some(CSIAction::EraseDisplay(mode))
            }
            b'K' => {
                let mode = match params.first().copied().unwrap_or(0) {
                    0 => EraseMode::ToEnd,
                    1 => EraseMode::ToBeginning,
                    2 => EraseMode::All,
                    _ => return None,
                };
                Some(CSIAction::EraseLine(mode))
            }

            // SGR - Select Graphic Rendition
            b'm' => {
                let attrs = parse_sgr_sequence(params);
                Some(CSIAction::SetGraphicsMode(attrs))
            }

            // Scrolling
            b'S' => Some(CSIAction::ScrollUp(params.first().copied().unwrap_or(1))),
            b'T' => Some(CSIAction::ScrollDown(params.first().copied().unwrap_or(1))),

            // Line operations
            b'L' => Some(CSIAction::InsertLines(params.first().copied().unwrap_or(1))),
            b'M' => Some(CSIAction::DeleteLines(params.first().copied().unwrap_or(1))),

            // Character operations
            b'@' => Some(CSIAction::InsertChars(params.first().copied().unwrap_or(1))),
            b'P' => Some(CSIAction::DeleteChars(params.first().copied().unwrap_or(1))),

            // Mode settings
            b'h' if is_private => {
                let modes = params
                    .iter()
                    .filter_map(|&p| parse_private_mode(p))
                    .collect();
                Some(CSIAction::SetMode(modes))
            }
            b'l' if is_private => {
                let modes = params
                    .iter()
                    .filter_map(|&p| parse_private_mode(p))
                    .collect();
                Some(CSIAction::ResetMode(modes))
            }

            // Scroll region
            b'r' => {
                let top = params.first().copied().unwrap_or(1);
                let bottom = params.get(1).copied().unwrap_or(0); // 0 means default
                Some(CSIAction::SetScrollRegion { top, bottom })
            }

            // Save/Restore cursor
            b's' => Some(CSIAction::SaveCursor),
            b'u' => Some(CSIAction::RestoreCursor),

            // Device status
            b'n' if params.first() == Some(&6) => Some(CSIAction::ReportCursorPosition),
            b'n' if params.first() == Some(&5) => Some(CSIAction::DeviceStatusReport),

            _ => None,
        }
    }
}

fn parse_sgr_sequence(params: &[u16]) -> Vec<SGRAttribute> {
    let mut attrs = Vec::new();
    let mut i = 0;

    while i < params.len() {
        match params[i] {
            0 => attrs.push(SGRAttribute::Reset),
            1 => attrs.push(SGRAttribute::Bold),
            2 => attrs.push(SGRAttribute::Dim),
            3 => attrs.push(SGRAttribute::Italic),
            4 => attrs.push(SGRAttribute::Underline),
            5 => attrs.push(SGRAttribute::Blink),
            7 => attrs.push(SGRAttribute::Reverse),
            8 => attrs.push(SGRAttribute::Hidden),
            9 => attrs.push(SGRAttribute::Strikethrough),

            // Foreground colors
            30..=37 => attrs.push(SGRAttribute::ForegroundColor(basic_color(params[i] - 30))),
            38 => {
                if let Some(color) = parse_extended_color(&params[i + 1..]) {
                    attrs.push(SGRAttribute::ForegroundColor(color.0));
                    i += color.1;
                }
            }
            39 => attrs.push(SGRAttribute::ForegroundDefault),

            // Background colors
            40..=47 => attrs.push(SGRAttribute::BackgroundColor(basic_color(params[i] - 40))),
            48 => {
                if let Some(color) = parse_extended_color(&params[i + 1..]) {
                    attrs.push(SGRAttribute::BackgroundColor(color.0));
                    i += color.1;
                }
            }
            49 => attrs.push(SGRAttribute::BackgroundDefault),

            // Bright foreground colors
            90..=97 => attrs.push(SGRAttribute::ForegroundColor(bright_color(params[i] - 90))),

            // Bright background colors
            100..=107 => attrs.push(SGRAttribute::BackgroundColor(bright_color(params[i] - 100))),

            // Reset attributes
            21 => attrs.push(SGRAttribute::NoBold),
            22 => attrs.push(SGRAttribute::NoBold), // Also resets dim
            23 => attrs.push(SGRAttribute::NoItalic),
            24 => attrs.push(SGRAttribute::NoUnderline),
            25 => attrs.push(SGRAttribute::NoBlink),
            27 => attrs.push(SGRAttribute::NoReverse),
            29 => attrs.push(SGRAttribute::NoStrikethrough),

            // Underline variants
            53 => attrs.push(SGRAttribute::DoubleUnderline),

            // Underline color
            58 => {
                if let Some(color) = parse_extended_color(&params[i + 1..]) {
                    attrs.push(SGRAttribute::UnderlineColor(color.0));
                    i += color.1;
                }
            }
            59 => attrs.push(SGRAttribute::UnderlineDefault),

            _ => {} // Ignore unknown
        }
        i += 1;
    }

    attrs
}

fn basic_color(n: u16) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        _ => Color::Black,
    }
}

fn bright_color(n: u16) -> Color {
    match n {
        0 => Color::BrightBlack,
        1 => Color::BrightRed,
        2 => Color::BrightGreen,
        3 => Color::BrightYellow,
        4 => Color::BrightBlue,
        5 => Color::BrightMagenta,
        6 => Color::BrightCyan,
        7 => Color::BrightWhite,
        _ => Color::BrightBlack,
    }
}

fn parse_extended_color(params: &[u16]) -> Option<(Color, usize)> {
    if params.is_empty() {
        return None;
    }

    match params[0] {
        5 if params.len() > 1 => {
            // 256 color
            Some((Color::Indexed(params[1] as u8), 2))
        }
        2 if params.len() > 3 => {
            // RGB color
            Some((
                Color::RGB {
                    r: params[1] as u8,
                    g: params[2] as u8,
                    b: params[3] as u8,
                },
                4,
            ))
        }
        _ => None,
    }
}

fn parse_private_mode(param: u16) -> Option<Mode> {
    match param {
        1 => Some(Mode::ApplicationCursorKeys),
        3 => Some(Mode::DECCOLM),
        4 => Some(Mode::SmoothScroll),
        5 => Some(Mode::ReverseVideo),
        6 => Some(Mode::Origin),
        7 => Some(Mode::AutoWrap),
        8 => Some(Mode::AutoRepeat),
        9 => Some(Mode::MouseX10),
        25 => Some(Mode::ShowCursor),
        1000 => Some(Mode::MouseVT200),
        1001 => Some(Mode::MouseVT200Highlight),
        1002 => Some(Mode::MouseButtonEvent),
        1003 => Some(Mode::MouseAnyEvent),
        1004 => Some(Mode::MouseFocusEvent),
        1006 => Some(Mode::MouseSGR),
        1016 => Some(Mode::MousePixel),
        1047 | 1049 => Some(Mode::AlternateScreen),
        2004 => Some(Mode::BracketedPaste),
        _ => Some(Mode::Unknown(param)),
    }
}
