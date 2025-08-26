/// Control Sequence Introducer (CSI) sequences
#[derive(Debug, Clone, PartialEq)]
pub enum CSIAction {
    // Cursor Movement
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorNextLine(u16),
    CursorPreviousLine(u16),
    CursorPosition { row: u16, col: u16 },
    CursorHorizontalAbsolute(u16),
    
    // Cursor Visibility
    ShowCursor,
    HideCursor,
    
    // Screen/Line Operations
    EraseDisplay(EraseMode),
    EraseLine(EraseMode),
    InsertLines(u16),
    DeleteLines(u16),
    InsertChars(u16),
    DeleteChars(u16),
    ScrollUp(u16),
    ScrollDown(u16),
    
    // Text Attributes (SGR - Select Graphic Rendition)
    SetGraphicsMode(Vec<SGRAttribute>),
    
    // Mouse Events
    MouseEvent(MouseEvent),
    
    // Device Status
    DeviceStatusReport,
    ReportCursorPosition,
    
    // Modes
    SetMode(Vec<Mode>),
    ResetMode(Vec<Mode>),
    
    // Other
    SetScrollRegion { top: u16, bottom: u16 },
    SaveCursor,
    RestoreCursor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EraseMode {
    ToEnd,      // From cursor to end
    ToBeginning, // From beginning to cursor
    All,        // Entire display/line
    Scrollback, // Including scrollback buffer (for EraseDisplay)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SGRAttribute {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strikethrough,
    
    // Foreground colors
    ForegroundColor(Color),
    ForegroundDefault,
    
    // Background colors
    BackgroundColor(Color),
    BackgroundDefault,
    
    // Underline colors
    UnderlineColor(Color),
    UnderlineDefault,
    
    // Extended attributes
    DoubleUnderline,
    CurlyUnderline,
    DottedUnderline,
    DashedUnderline,
    NoUnderline,
    
    // Reset individual attributes
    NoBold,
    NoItalic,
    NoReverse,
    NoStrikethrough,
    NoBlink,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    // Basic 16 colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    
    // Extended colors
    Indexed(u8),        // 256-color palette
    RGB { r: u8, g: u8, b: u8 }, // True color
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub modifiers: MouseModifiers,
    pub x: u16,
    pub y: u16,
    pub mode: MouseMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    Button(u8), // Extended buttons
    Motion,     // Mouse motion without button
    Release,    // Button release
}

#[derive(Debug, Clone, PartialEq)]
pub struct MouseModifiers {
    pub shift: bool,
    pub alt: bool,
    pub ctrl: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseMode {
    Normal,     // X10 compatibility mode
    ButtonEvent, // Button press/release events
    AnyEvent,   // Any mouse event
    SGR,        // SGR extended mode (supports coordinates > 223)
    Pixel,      // Pixel-level precision
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mode {
    // ANSI modes
    KeyboardAction,
    Insert,
    SendReceive,
    AutomaticNewline,
    
    // DEC Private modes
    ApplicationCursorKeys,
    DECCOLM, // 132 column mode
    SmoothScroll,
    ReverseVideo,
    Origin,
    AutoWrap,
    AutoRepeat,
    Interlace,
    
    // Mouse modes
    MouseX10,
    MouseVT200,
    MouseVT200Highlight,
    MouseButtonEvent,
    MouseAnyEvent,
    MouseFocusEvent,
    MouseSGR,
    MousePixel,
    
    // Other modes
    AlternateScreen,
    BracketedPaste,
    ShowCursor,
    
    // Unknown mode (for forward compatibility)
    Unknown(u16),
}

impl CSIAction {
    /// Parse a CSI sequence from its components
    pub fn parse(params: &[u16], intermediates: &[u8], final_byte: u8) -> Option<Self> {
        // Handle private sequences (starting with '?')
        let is_private = intermediates.contains(&b'?');
        
        match final_byte {
            // Cursor movement
            b'A' => Some(CSIAction::CursorUp(params.get(0).copied().unwrap_or(1))),
            b'B' => Some(CSIAction::CursorDown(params.get(0).copied().unwrap_or(1))),
            b'C' => Some(CSIAction::CursorForward(params.get(0).copied().unwrap_or(1))),
            b'D' => Some(CSIAction::CursorBack(params.get(0).copied().unwrap_or(1))),
            b'E' => Some(CSIAction::CursorNextLine(params.get(0).copied().unwrap_or(1))),
            b'F' => Some(CSIAction::CursorPreviousLine(params.get(0).copied().unwrap_or(1))),
            b'G' => Some(CSIAction::CursorHorizontalAbsolute(params.get(0).copied().unwrap_or(1))),
            b'H' | b'f' => {
                let row = params.get(0).copied().unwrap_or(1);
                let col = params.get(1).copied().unwrap_or(1);
                Some(CSIAction::CursorPosition { row, col })
            }
            
            // Erase operations
            b'J' => {
                let mode = match params.get(0).copied().unwrap_or(0) {
                    0 => EraseMode::ToEnd,
                    1 => EraseMode::ToBeginning,
                    2 => EraseMode::All,
                    3 => EraseMode::Scrollback,
                    _ => return None,
                };
                Some(CSIAction::EraseDisplay(mode))
            }
            b'K' => {
                let mode = match params.get(0).copied().unwrap_or(0) {
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
            b'S' => Some(CSIAction::ScrollUp(params.get(0).copied().unwrap_or(1))),
            b'T' => Some(CSIAction::ScrollDown(params.get(0).copied().unwrap_or(1))),
            
            // Line operations
            b'L' => Some(CSIAction::InsertLines(params.get(0).copied().unwrap_or(1))),
            b'M' => Some(CSIAction::DeleteLines(params.get(0).copied().unwrap_or(1))),
            
            // Character operations
            b'@' => Some(CSIAction::InsertChars(params.get(0).copied().unwrap_or(1))),
            b'P' => Some(CSIAction::DeleteChars(params.get(0).copied().unwrap_or(1))),
            
            // Mode settings
            b'h' if is_private => {
                let modes = params.iter().filter_map(|&p| parse_private_mode(p)).collect();
                Some(CSIAction::SetMode(modes))
            }
            b'l' if is_private => {
                let modes = params.iter().filter_map(|&p| parse_private_mode(p)).collect();
                Some(CSIAction::ResetMode(modes))
            }
            
            // Scroll region
            b'r' => {
                let top = params.get(0).copied().unwrap_or(1);
                let bottom = params.get(1).copied().unwrap_or(0); // 0 means default
                Some(CSIAction::SetScrollRegion { top, bottom })
            }
            
            // Save/Restore cursor
            b's' => Some(CSIAction::SaveCursor),
            b'u' => Some(CSIAction::RestoreCursor),
            
            // Device status
            b'n' if params.get(0) == Some(&6) => Some(CSIAction::ReportCursorPosition),
            b'n' if params.get(0) == Some(&5) => Some(CSIAction::DeviceStatusReport),
            
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
                if let Some(color) = parse_extended_color(&params[i+1..]) {
                    attrs.push(SGRAttribute::ForegroundColor(color.0));
                    i += color.1;
                }
            }
            39 => attrs.push(SGRAttribute::ForegroundDefault),
            
            // Background colors
            40..=47 => attrs.push(SGRAttribute::BackgroundColor(basic_color(params[i] - 40))),
            48 => {
                if let Some(color) = parse_extended_color(&params[i+1..]) {
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
                if let Some(color) = parse_extended_color(&params[i+1..]) {
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
            Some((Color::RGB {
                r: params[1] as u8,
                g: params[2] as u8,
                b: params[3] as u8,
            }, 4))
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