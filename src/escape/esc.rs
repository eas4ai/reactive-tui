/// ESC sequences (non-CSI)
#[derive(Debug, Clone, PartialEq)]
pub enum ESCAction {
    /// Index - Move cursor down one line, scroll if at bottom
    Index,

    /// Next Line - Move cursor to beginning of next line, scroll if at bottom
    NextLine,

    /// Tab Set - Set a tab stop at current cursor position
    TabSet,

    /// Reverse Index - Move cursor up one line, scroll if at top
    ReverseIndex,

    /// Single Shift G2 - Use G2 character set for next character only
    SingleShiftG2,

    /// Single Shift G3 - Use G3 character set for next character only
    SingleShiftG3,

    /// Start of Protected Area
    StartProtectedArea,

    /// End of Protected Area
    EndProtectedArea,

    /// Reset to Initial State
    Reset,

    /// Save Cursor (DECSC)
    SaveCursor,

    /// Restore Cursor (DECRC)
    RestoreCursor,

    /// Application Keypad Mode
    ApplicationKeypad,

    /// Normal Keypad Mode
    NormalKeypad,

    /// Set charset G0
    SetCharsetG0(Charset),

    /// Set charset G1
    SetCharsetG1(Charset),

    /// Set charset G2
    SetCharsetG2(Charset),

    /// Set charset G3
    SetCharsetG3(Charset),

    /// Invoke charset G0
    InvokeCharsetG0,

    /// Invoke charset G1
    InvokeCharsetG1,

    /// Invoke charset G2
    InvokeCharsetG2,

    /// Invoke charset G3
    InvokeCharsetG3,

    /// DEC Screen Alignment Test - Fill screen with 'E'
    ScreenAlignmentTest,

    /// Set terminal title (old style)
    SetTitle(String),

    /// Bell alternative
    VisualBell,

    /// Unknown ESC sequence
    Unknown(Vec<u8>),
}

/// Character set selection for terminal escape sequences
#[derive(Debug, Clone, PartialEq)]
pub enum Charset {
    /// Standard ASCII character set
    ASCII,
    /// DEC special graphics character set
    DecSpecialGraphics,
    /// United Kingdom character set
    UnitedKingdom,
    /// US alternate character set
    USAlternate,
    /// DEC supplemental character set
    DecSupplemental,
    /// DEC technical character set
    DecTechnical,
    /// French character set
    French,
    /// French Canadian character set
    FrenchCanadian,
    /// German character set
    German,
    /// Italian character set
    Italian,
    /// Norwegian/Danish character set
    NorwegianDanish,
    /// Portuguese character set
    Portuguese,
    /// Spanish character set
    Spanish,
    /// Swedish character set
    Swedish,
    /// Swiss character set
    Swiss,
    /// DEC Greek character set
    DecGreek,
    /// DEC Hebrew character set
    DecHebrew,
    /// DEC Turkish character set
    DecTurkish,
    /// DEC Cyrillic character set
    DecCyrillic,
    /// SCS DEC supplemental character set
    SCSDecSupplemental,
    /// Dutch character set
    Dutch,
    /// Finnish character set
    Finnish,
}

impl ESCAction {
    /// Parse an ESC sequence from its components
    pub fn parse(intermediates: &[u8], final_byte: u8) -> Option<Self> {
        // Handle sequences with no intermediates
        if intermediates.is_empty() {
            return match final_byte {
                b'D' => Some(ESCAction::Index),
                b'E' => Some(ESCAction::NextLine),
                b'H' => Some(ESCAction::TabSet),
                b'M' => Some(ESCAction::ReverseIndex),
                b'N' => Some(ESCAction::SingleShiftG2),
                b'O' => Some(ESCAction::SingleShiftG3),
                b'V' => Some(ESCAction::StartProtectedArea),
                b'W' => Some(ESCAction::EndProtectedArea),
                b'c' => Some(ESCAction::Reset),
                b'n' => Some(ESCAction::InvokeCharsetG2),
                b'o' => Some(ESCAction::InvokeCharsetG3),
                b'|' => Some(ESCAction::InvokeCharsetG3),
                b'}' => Some(ESCAction::InvokeCharsetG2),
                b'~' => Some(ESCAction::InvokeCharsetG1),
                b'7' => Some(ESCAction::SaveCursor),
                b'8' => Some(ESCAction::RestoreCursor),
                b'=' => Some(ESCAction::ApplicationKeypad),
                b'>' => Some(ESCAction::NormalKeypad),
                _ => None,
            };
        }

        // Handle sequences with intermediates
        match (intermediates.first(), final_byte) {
            // Charset selection
            (Some(b'('), b) => parse_charset(b).map(ESCAction::SetCharsetG0),
            (Some(b')'), b) => parse_charset(b).map(ESCAction::SetCharsetG1),
            (Some(b'*'), b) => parse_charset(b).map(ESCAction::SetCharsetG2),
            (Some(b'+'), b) => parse_charset(b).map(ESCAction::SetCharsetG3),

            // DEC private sequences
            (Some(b'#'), b'3') => Some(ESCAction::ScreenAlignmentTest),
            (Some(b'#'), b'8') => Some(ESCAction::ScreenAlignmentTest),

            // Title setting (old xterm style)
            (Some(b']'), b'0') => Some(ESCAction::SetTitle(String::new())),

            _ => None,
        }
    }
}

fn parse_charset(byte: u8) -> Option<Charset> {
    match byte {
        b'A' => Some(Charset::UnitedKingdom),
        b'B' => Some(Charset::ASCII),
        b'0' => Some(Charset::DecSpecialGraphics),
        b'1' => Some(Charset::USAlternate),
        b'2' => Some(Charset::USAlternate),
        b'4' => Some(Charset::Dutch),
        b'5' => Some(Charset::Finnish),
        b'6' => Some(Charset::NorwegianDanish),
        b'7' => Some(Charset::Swedish),
        b'9' => Some(Charset::FrenchCanadian),
        b'<' => Some(Charset::DecSupplemental),
        b'=' => Some(Charset::Swiss),
        b'>' => Some(Charset::DecTechnical),
        b'?' => Some(Charset::DecGreek),
        b'F' => Some(Charset::DecGreek),
        b'H' => Some(Charset::DecHebrew),
        b'K' => Some(Charset::German),
        b'Q' => Some(Charset::FrenchCanadian),
        b'R' => Some(Charset::French),
        b'Y' => Some(Charset::Italian),
        b'Z' => Some(Charset::Spanish),
        b'`' => Some(Charset::NorwegianDanish),
        _ => None,
    }
}
