//! Escape sequence parser for terminal input
//!
//! Bounded streaming parser for terminal escape sequences

use super::{
    ColorScheme, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind, TerminalEvent,
};

/// Key state for compatibility
#[derive(Debug, Clone, Default)]
pub struct KeyState;

/// Parser result containing an event and number of bytes consumed
#[derive(Debug)]
pub struct ParseResult {
    /// Parsed terminal event (None if incomplete)
    pub event: Option<TerminalEvent>,
    /// Number of bytes consumed from input
    pub n: usize,
}

/// Mouse button bits for parsing
mod mouse_bits {
    pub const MOTION: u8 = 0b00100000;
    pub const SHIFT: u8 = 0b00000100;
    pub const ALT: u8 = 0b00001000;
    pub const CTRL: u8 = 0b00010000;
}

/// Parser state for escape sequence processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserState {
    /// Normal character processing state
    Ground,
    /// Escape sequence started (ESC received)
    Escape,
    /// Control Sequence Introducer (CSI) state
    Csi,
    /// Operating System Command (OSC) state
    Osc,
    /// Device Control String (DCS) state
    Dcs,
    /// Start of String (SOS) state
    Sos,
    /// Privacy Message (PM) state
    Pm,
    /// Application Program Command (APC) state
    Apc,
    /// Single Shift 2 (SS2) state
    Ss2,
    /// Single Shift 3 (SS3) state
    Ss3,
}

/// Escape sequence parser with comprehensive terminal support
pub struct EscapeSequenceParser {
    pending: Vec<u8>,
}

impl EscapeSequenceParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
        }
    }

    /// Parse input bytes and return terminal events
    pub fn parse(&mut self, input: &[u8]) -> Vec<TerminalEvent> {
        const MAX_PENDING: usize = 4096;
        if self.pending.len().saturating_add(input.len()) > MAX_PENDING {
            self.pending.clear();
            return vec![TerminalEvent::Error(format!(
                "terminal input sequence exceeds the {MAX_PENDING}-byte parser limit"
            ))];
        }
        let mut pending = std::mem::take(&mut self.pending);
        pending.extend_from_slice(input);
        let mut events = Vec::new();
        let mut offset = 0;

        while offset < pending.len() {
            let remaining = &pending[offset..];
            if remaining == [0x1b] {
                break;
            }
            let result = self.parse_single(remaining);

            if result.n == 0 {
                // No complete sequence found, break
                break;
            }

            if let Some(event) = result.event {
                events.push(event);
            }

            offset += result.n;
        }
        self.pending.extend_from_slice(&pending[offset..]);
        events
    }

    /// Resolve an ambiguous lone Escape byte after the caller's input timeout.
    pub fn flush_pending(&mut self) -> Vec<TerminalEvent> {
        if self.pending == [0x1b] {
            self.pending.clear();
            vec![TerminalEvent::Key {
                code: KeyCode::Escape,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }]
        } else {
            Vec::new()
        }
    }

    /// Parse the first event from the input buffer
    ///
    /// If a complete event is not present, ParseResult.event will be None and ParseResult.n will be 0
    /// If an unknown event is found, ParseResult.event will be None and ParseResult.n will be > 0
    fn parse_single(&mut self, input: &[u8]) -> ParseResult {
        if input.is_empty() {
            return ParseResult { event: None, n: 0 };
        }

        // We gate this for len > 1 so we can detect singular escape key presses
        if input[0] == 0x1b && input.len() > 1 {
            match input[1] {
                0x4F => self.parse_ss3(input),
                0x50 => self.skip_until_st(input), // DCS
                0x58 => self.skip_until_st(input), // SOS
                0x5B => {
                    // Check for legacy mouse format first
                    if input.len() >= 3 && input[2] == b'M' {
                        self.parse_mouse_legacy(input)
                    } else {
                        self.parse_csi(input)
                    }
                }
                0x5D => self.parse_osc(input),     // OSC
                0x5E => self.skip_until_st(input), // PM
                0x5F => self.parse_apc(input),     // APC
                _ => match std::str::from_utf8(&input[1..]) {
                    Ok(text) => match text.chars().next() {
                        Some(ch) => ParseResult {
                            event: Some(TerminalEvent::Key {
                                code: KeyCode::Char(ch),
                                modifiers: KeyModifiers {
                                    alt: true,
                                    ..Default::default()
                                },
                                kind: KeyEventKind::Press,
                            }),
                            n: 1 + ch.len_utf8(),
                        },
                        None => ParseResult { event: None, n: 0 },
                    },
                    Err(error) if error.error_len().is_none() => ParseResult { event: None, n: 0 },
                    Err(_) => ParseResult {
                        event: Some(TerminalEvent::Error("invalid UTF-8 after Escape".into())),
                        n: 2,
                    },
                },
            }
        } else {
            self.parse_ground(input)
        }
    }

    /// Parse ground state (normal characters)
    fn parse_ground(&mut self, input: &[u8]) -> ParseResult {
        let b = input[0];
        let mut n = 1;

        // Ground state generates keypresses when parsing input. We
        // generally get ascii characters, but anything less than
        // 0x20 is a Ctrl+<c> keypress. We map these to lowercase
        // ascii characters when we can
        let event = match b {
            0x00 => Some(TerminalEvent::Key {
                code: KeyCode::Char('@'),
                modifiers: KeyModifiers {
                    ctrl: true,
                    ..Default::default()
                },
                kind: KeyEventKind::Press,
            }),
            0x08 => Some(TerminalEvent::Key {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            0x09 => Some(TerminalEvent::Key {
                code: KeyCode::Tab,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            0x0A | 0x0D => Some(TerminalEvent::Key {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            0x01..=0x07 | 0x0B..=0x0C | 0x0E..=0x1A => {
                // Ctrl+letter combinations
                Some(TerminalEvent::Key {
                    code: KeyCode::Char((b + 0x60) as char),
                    modifiers: KeyModifiers {
                        ctrl: true,
                        ..Default::default()
                    },
                    kind: KeyEventKind::Press,
                })
            }
            0x1B => {
                // ESC key (only when input.len() == 1)
                debug_assert!(input.len() == 1);
                Some(TerminalEvent::Key {
                    code: KeyCode::Escape,
                    modifiers: KeyModifiers::empty(),
                    kind: KeyEventKind::Press,
                })
            }
            0x7F => Some(TerminalEvent::Key {
                code: KeyCode::Backspace,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            _ => {
                // Handle UTF-8 characters
                match std::str::from_utf8(input) {
                    Ok(s) => s.chars().next().map(|ch| {
                        n = ch.len_utf8();
                        TerminalEvent::Key {
                            code: KeyCode::Char(ch),
                            modifiers: KeyModifiers::empty(),
                            kind: KeyEventKind::Press,
                        }
                    }),
                    Err(error) if error.error_len().is_none() => {
                        n = 0;
                        None
                    }
                    Err(error) => {
                        n = error.error_len().unwrap_or(1);
                        Some(TerminalEvent::Error("invalid UTF-8 terminal input".into()))
                    }
                }
            }
        };

        ParseResult { event, n }
    }

    /// Parse SS3 sequences (ESC O)
    fn parse_ss3(&mut self, input: &[u8]) -> ParseResult {
        if input.len() < 3 {
            return ParseResult { event: None, n: 0 };
        }

        let event = match input[2] {
            0x1B => return ParseResult { event: None, n: 2 },
            b'A' => Some(TerminalEvent::Key {
                code: KeyCode::Up,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'B' => Some(TerminalEvent::Key {
                code: KeyCode::Down,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'C' => Some(TerminalEvent::Key {
                code: KeyCode::Right,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'D' => Some(TerminalEvent::Key {
                code: KeyCode::Left,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'E' => Some(TerminalEvent::Key {
                code: KeyCode::Char('5'), // KP_BEGIN
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'F' => Some(TerminalEvent::Key {
                code: KeyCode::End,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'H' => Some(TerminalEvent::Key {
                code: KeyCode::Home,
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'P' => Some(TerminalEvent::Key {
                code: KeyCode::F(1),
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'Q' => Some(TerminalEvent::Key {
                code: KeyCode::F(2),
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'R' => Some(TerminalEvent::Key {
                code: KeyCode::F(3),
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            b'S' => Some(TerminalEvent::Key {
                code: KeyCode::F(4),
                modifiers: KeyModifiers::empty(),
                kind: KeyEventKind::Press,
            }),
            _ => {
                // Unknown SS3 sequence
                None
            }
        };

        ParseResult { event, n: 3 }
    }

    /// Parse APC sequences (ESC _)
    fn parse_apc(&mut self, input: &[u8]) -> ParseResult {
        if input.len() < 3 {
            return ParseResult { event: None, n: 0 };
        }

        let end = if let Some(pos) = input[2..].windows(2).position(|w| w == b"\x1b\\") {
            pos + 2 + 2 // position + offset + ESC \ length
        } else {
            return ParseResult { event: None, n: 0 };
        };

        let sequence = &input[0..end];

        let event = match input[2] {
            b'G' => Some(TerminalEvent::CapabilityResponse(
                "kitty_graphics".to_string(),
            )),
            _ => None,
        };

        ParseResult {
            event,
            n: sequence.len(),
        }
    }

    /// Skip sequences until we see an ST (String Terminator, ESC \)
    fn skip_until_st(&mut self, input: &[u8]) -> ParseResult {
        if input.len() < 3 {
            return ParseResult { event: None, n: 0 };
        }

        let end = if let Some(pos) = input[2..].windows(2).position(|w| w == b"\x1b\\") {
            pos + 2 + 2 // position + offset + ESC \ length
        } else {
            return ParseResult { event: None, n: 0 };
        };

        ParseResult {
            event: None,
            n: end,
        }
    }

    /// Parse OSC sequences (ESC ])
    fn parse_osc(&mut self, input: &[u8]) -> ParseResult {
        if input.len() < 3 {
            return ParseResult { event: None, n: 0 };
        }

        let mut bel_terminated = false;

        // Find terminator (either ESC \ or BEL)
        let end = if let Some(pos) = input[2..].windows(2).position(|w| w == b"\x1b\\") {
            pos + 2 + 2 // position + offset + ESC \ length
        } else if let Some(pos) = input[2..].iter().position(|&b| b == 0x07) {
            bel_terminated = true;
            pos + 2 + 1 // position + offset + BEL length
        } else {
            return ParseResult { event: None, n: 0 };
        };

        let sequence = &input[0..end];
        let null_result = ParseResult {
            event: None,
            n: sequence.len(),
        };

        // Find semicolon separator
        let semicolon_idx = if let Some(pos) = input[2..].iter().position(|&b| b == b';') {
            pos + 2
        } else {
            return null_result;
        };

        // Parse parameter
        let ps_str = std::str::from_utf8(&input[2..semicolon_idx]).ok();
        let ps: u16 = if let Some(s) = ps_str {
            s.parse().ok().unwrap_or(0)
        } else {
            return null_result;
        };

        let event = match ps {
            4 => {
                // Color palette query response
                self.parse_color_palette_response(
                    &input[semicolon_idx + 1..end - if bel_terminated { 1 } else { 2 }],
                )
            }
            10..=12 => {
                // Foreground/background/cursor color query response
                self.parse_color_query_response(
                    ps,
                    &input[semicolon_idx + 1..end - if bel_terminated { 1 } else { 2 }],
                )
            }
            52 => None,
            996 => {
                // Color scheme response
                self.parse_color_scheme_response(
                    &input[semicolon_idx + 1..end - if bel_terminated { 1 } else { 2 }],
                )
            }
            _ => None,
        };

        ParseResult {
            event,
            n: sequence.len(),
        }
    }

    /// Parse color palette response (OSC 4)
    fn parse_color_palette_response(&mut self, data: &[u8]) -> Option<TerminalEvent> {
        let content = std::str::from_utf8(data).ok()?;
        let mut parts = content.splitn(2, ';');
        let _index = parts.next()?.parse::<u8>().ok()?;
        let _color_spec = parts.next()?;

        // For now, just return a capability response
        Some(TerminalEvent::CapabilityResponse(format!(
            "color_palette:{}",
            content
        )))
    }

    /// Parse color query response (OSC 10/11/12)
    fn parse_color_query_response(&mut self, ps: u16, data: &[u8]) -> Option<TerminalEvent> {
        let content = std::str::from_utf8(data).ok()?;
        let color_type = match ps {
            10 => "fg",
            11 => "bg",
            12 => "cursor",
            _ => return None,
        };

        Some(TerminalEvent::CapabilityResponse(format!(
            "color_{}:{}",
            color_type, content
        )))
    }

    /// Parse color scheme response (OSC 996)
    fn parse_color_scheme_response(&mut self, data: &[u8]) -> Option<TerminalEvent> {
        let content = std::str::from_utf8(data).ok()?;
        let scheme = match content.trim() {
            "dark" => ColorScheme::Dark,
            "light" => ColorScheme::Light,
            _ => return None,
        };

        Some(TerminalEvent::ColorScheme(scheme))
    }

    /// Parse CSI sequences (ESC [)
    fn parse_csi(&mut self, input: &[u8]) -> ParseResult {
        if input.len() < 3 {
            return ParseResult { event: None, n: 0 };
        }

        // Find the final byte (0x40-0xFF)
        let sequence = if let Some((i, _)) = input[2..].iter().enumerate().find(|(_, &b)| b >= 0x40)
        {
            &input[0..i + 2 + 1]
        } else {
            return ParseResult { event: None, n: 0 };
        };

        let _null_result = ParseResult {
            event: None,
            n: sequence.len(),
        };
        let final_byte = sequence[sequence.len() - 1];

        let event = match final_byte {
            b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'H' | b'P' | b'Q' | b'R' | b'S' => {
                self.parse_legacy_keys(sequence, final_byte)
            }
            b'~' => self.parse_tilde_keys(sequence),
            b'I' => Some(TerminalEvent::FocusGained),
            b'O' => Some(TerminalEvent::FocusLost),
            b'M' | b'm' => self.parse_mouse_sgr(sequence),

            b'c' => self.parse_device_attributes(sequence),
            b'n' => self.parse_device_status_report(sequence),
            b't' => self.parse_window_manipulation(sequence),
            b'u' => self.parse_kitty_keyboard(sequence),
            b'y' => self.parse_decrpm(sequence),
            _ => None,
        };

        ParseResult {
            event,
            n: sequence.len(),
        }
    }

    /// Parse legacy mouse events (CSI M ...)
    fn parse_mouse_legacy(&mut self, input: &[u8]) -> ParseResult {
        // Legacy mouse format: ESC [ M <button> <x> <y>
        // Need at least 5 bytes: ESC [ M + 2 data bytes (minimum)
        if input.len() < 6 {
            return ParseResult { event: None, n: 0 };
        }

        let button = input[3];
        let x = input[4];
        let y = input[5];

        // Decode button (subtract 32 to get actual values)
        let button_data = button.wrapping_sub(32);
        let x_pos = (x.wrapping_sub(33)) as u16; // 1-based to 0-based
        let y_pos = (y.wrapping_sub(33)) as u16; // 1-based to 0-based

        let base = button_data & 0x03;
        let button = match base {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            _ => None,
        };
        let kind = if button_data & 0x40 != 0 {
            if base == 0 {
                MouseEventKind::ScrollUp
            } else {
                MouseEventKind::ScrollDown
            }
        } else if button_data & mouse_bits::MOTION != 0 {
            if button.is_some() {
                MouseEventKind::Drag
            } else {
                MouseEventKind::Move
            }
        } else if base == 3 {
            MouseEventKind::Up
        } else {
            MouseEventKind::Down
        };

        let modifiers = KeyModifiers {
            shift: (button_data & 0x04) != 0,
            alt: (button_data & 0x08) != 0,
            ctrl: (button_data & 0x10) != 0,
            meta: false,
        };

        let event = Some(TerminalEvent::Mouse {
            kind,
            button,
            column: x_pos,
            row: y_pos,
            pixel_x: None,
            pixel_y: None,
            modifiers,
        });

        ParseResult {
            event,
            n: input.len().min(6),
        }
    }

    /// Parse legacy keys (CSI {ABCDEFHPQRS})
    fn parse_legacy_keys(&mut self, sequence: &[u8], final_byte: u8) -> Option<TerminalEvent> {
        // Split into fields delimited by ';'
        let params_str = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
        let mut fields = params_str.split(';');

        // Skip first field
        let _ = fields.next();

        let mut is_release = false;
        let key_code = match final_byte {
            b'A' => KeyCode::Up,
            b'B' => KeyCode::Down,
            b'C' => KeyCode::Right,
            b'D' => KeyCode::Left,
            b'E' => KeyCode::Char('5'), // KP_BEGIN
            b'F' => KeyCode::End,
            b'H' => KeyCode::Home,
            b'P' => KeyCode::F(1),
            b'Q' => KeyCode::F(2),
            b'R' => KeyCode::F(3),
            b'S' => KeyCode::F(4),
            _ => return None,
        };

        let mut modifiers = KeyModifiers::empty();

        // Field 2: modifier_mask:event_type
        if let Some(field) = fields.next() {
            let mut params = field.split(':');
            if let Some(modifier_str) = params.next() {
                if let Ok(modifier_mask) = modifier_str.parse::<u8>() {
                    modifiers = self.decode_modifiers(modifier_mask.saturating_sub(1));
                }
            }
            if let Some(event_type) = params.next() {
                is_release = event_type == "3";
            }
        }

        // Field 3: text_as_codepoint - handle Unicode codepoints for text input
        if let Some(field) = fields.next() {
            if let Ok(codepoint) = field.parse::<u32>() {
                // Convert codepoint to char if valid
                if let Some(ch) = char::from_u32(codepoint) {
                    // For printable characters, we could enhance the key event
                    if ch.is_ascii_graphic() || ch.is_whitespace() {
                        // This codepoint provides additional context for the key event
                        // Could be used for international keyboard support
                    }
                }
            }
        }

        let kind = if is_release {
            KeyEventKind::Release
        } else {
            KeyEventKind::Press
        };

        Some(TerminalEvent::Key {
            code: key_code,
            modifiers,
            kind,
        })
    }

    /// Parse tilde keys (CSI number ~)
    fn parse_tilde_keys(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        let params_str = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
        let mut fields = params_str.split(';');

        let number_str = fields.next()?;
        let number: u16 = number_str.parse().ok()?;

        let key_code = match number {
            2 => KeyCode::Insert,
            3 => KeyCode::Delete,
            5 => KeyCode::PageUp,
            6 => KeyCode::PageDown,
            7 => KeyCode::Home,
            8 => KeyCode::End,
            11 => KeyCode::F(1),
            12 => KeyCode::F(2),
            13 => KeyCode::F(3),
            14 => KeyCode::F(4),
            15 => KeyCode::F(5),
            17 => KeyCode::F(6),
            18 => KeyCode::F(7),
            19 => KeyCode::F(8),
            20 => KeyCode::F(9),
            21 => KeyCode::F(10),
            23 => KeyCode::F(11),
            24 => KeyCode::F(12),
            200 => return Some(TerminalEvent::PasteStart),
            201 => return Some(TerminalEvent::PasteEnd),
            57427 => KeyCode::Char('5'), // KP_BEGIN
            _ => return None,
        };

        let mut modifiers = KeyModifiers::empty();
        let mut is_release = false;

        // Field 2: modifier_mask:event_type
        if let Some(field) = fields.next() {
            let mut params = field.split(':');
            if let Some(modifier_str) = params.next() {
                if let Ok(modifier_mask) = modifier_str.parse::<u8>() {
                    modifiers = self.decode_modifiers(modifier_mask.saturating_sub(1));
                }
            }
            if let Some(event_type) = params.next() {
                is_release = event_type == "3";
            }
        }

        // Field 3: text_as_codepoint - handle Unicode codepoints for text input
        if let Some(field) = fields.next() {
            if let Ok(codepoint) = field.parse::<u32>() {
                if let Some(ch) = char::from_u32(codepoint) {
                    if ch.is_ascii_graphic() || ch.is_whitespace() {
                        // Additional context for international keyboard support
                    }
                }
            }
        }

        let kind = if is_release {
            KeyEventKind::Release
        } else {
            KeyEventKind::Press
        };

        Some(TerminalEvent::Key {
            code: key_code,
            modifiers,
            kind,
        })
    }

    /// Parse SGR mouse events (CSI < ... M/m)
    fn parse_mouse_sgr(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        if sequence.len() < 4 || sequence[2] != b'<' {
            return None;
        }

        let params_str = std::str::from_utf8(&sequence[3..sequence.len() - 1]).ok()?;
        let params: Vec<&str> = params_str.split(';').collect();

        if params.len() < 3 {
            return None;
        }

        let button_mask: u16 = params[0].parse().ok()?;
        let px: u16 = params[1].parse().ok()?;
        let py: u16 = params[2].parse().ok()?;

        let button = button_mask & 0x03;
        let motion = (button_mask & mouse_bits::MOTION as u16) > 0;
        let shift = (button_mask & mouse_bits::SHIFT as u16) > 0;
        let alt = (button_mask & mouse_bits::ALT as u16) > 0;
        let ctrl = (button_mask & mouse_bits::CTRL as u16) > 0;

        let modifiers = KeyModifiers {
            shift,
            alt,
            ctrl,
            meta: false,
        };

        let physical_button = match button {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            _ => None,
        };
        let kind = if button_mask & 0x40 != 0 {
            if button == 0 {
                MouseEventKind::ScrollUp
            } else {
                MouseEventKind::ScrollDown
            }
        } else if motion && physical_button.is_some() {
            MouseEventKind::Drag
        } else if motion {
            MouseEventKind::Move
        } else if sequence[sequence.len() - 1] == b'm' {
            MouseEventKind::Up
        } else {
            MouseEventKind::Down
        };

        Some(TerminalEvent::Mouse {
            kind,
            button: physical_button,
            column: px.saturating_sub(1),
            row: py.saturating_sub(1),
            pixel_x: None,
            pixel_y: None,
            modifiers,
        })
    }

    /// Parse device attributes (CSI ? ... c)
    fn parse_device_attributes(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        if sequence.len() >= 4 && sequence[2] == b'?' {
            Some(TerminalEvent::CapabilityResponse("da1".to_string()))
        } else {
            None
        }
    }

    /// Parse device status report (CSI ? ... n)
    fn parse_device_status_report(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        if sequence.len() < 4 || sequence[2] != b'?' {
            return None;
        }

        let params_str = std::str::from_utf8(&sequence[3..sequence.len() - 1]).ok()?;
        let mut parts = params_str.split(';');
        let ps: u16 = parts.next()?.parse().ok()?;

        match ps {
            997 => {
                // Color scheme update
                if let Some(scheme_str) = parts.next() {
                    match scheme_str {
                        "1" => Some(TerminalEvent::ColorScheme(super::ColorScheme::Dark)),
                        "2" => Some(TerminalEvent::ColorScheme(super::ColorScheme::Light)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Parse window manipulation (CSI ... t)
    fn parse_window_manipulation(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        let params_str = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
        let params: Vec<&str> = params_str.split(';').collect();

        if params.len() >= 5 && params[0] == "48" {
            // In-band window resize: CSI 48 ; height ; width ; height_pix ; width_pix t
            let height: u16 = params[1].parse().ok()?;
            let width: u16 = params[2].parse().ok()?;

            Some(TerminalEvent::Resize { width, height })
        } else {
            None
        }
    }

    /// Parse Kitty keyboard protocol (CSI ... u)
    fn parse_kitty_keyboard(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        if sequence.len() > 2 && sequence[2] == b'?' {
            return Some(TerminalEvent::CapabilityResponse(
                "kitty_keyboard".to_string(),
            ));
        }

        let params_str = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
        let mut fields = params_str.split(';');

        // Field 1: unicode-key-code:shifted_codepoint:base_layout_codepoint
        let field1 = fields.next()?;
        let mut key_params = field1.split(':');
        let codepoint: u32 = key_params.next()?.parse().ok()?;
        let _shifted_codepoint: Option<u32> = key_params.next().and_then(|s| s.parse().ok());
        let _base_layout_codepoint: Option<u32> = key_params.next().and_then(|s| s.parse().ok());

        let mut modifiers = KeyModifiers::empty();
        let mut is_release = false;

        // Field 2: modifier_mask:event_type
        if let Some(field2) = fields.next() {
            let mut mod_params = field2.split(':');
            if let Some(modifier_str) = mod_params.next() {
                if let Ok(modifier_mask) = modifier_str.parse::<u8>() {
                    modifiers = self.decode_modifiers(modifier_mask.saturating_sub(1));
                }
            }
            if let Some(event_type) = mod_params.next() {
                is_release = event_type == "3";
            }
        }

        // Field 3: text_as_codepoint - handle Unicode codepoints for text input
        if let Some(field3) = fields.next() {
            if let Ok(codepoint) = field3.parse::<u32>() {
                if let Some(ch) = char::from_u32(codepoint) {
                    if ch.is_ascii_graphic() || ch.is_whitespace() {
                        // Additional context for international keyboard support
                    }
                }
            }
        }

        let code = self.map_kitty_codepoint_to_keycode(codepoint);
        let kind = if is_release {
            KeyEventKind::Release
        } else {
            KeyEventKind::Press
        };

        Some(TerminalEvent::Key {
            code,
            modifiers,
            kind,
        })
    }

    /// Parse DECRPM (CSI ? ... $ y)
    fn parse_decrpm(&mut self, sequence: &[u8]) -> Option<TerminalEvent> {
        if sequence.len() < 6 || sequence[2] != b'?' {
            return None;
        }

        let params_str = std::str::from_utf8(&sequence[3..sequence.len() - 2]).ok()?;
        let mut parts = params_str.split(';');
        let ps: u16 = parts.next()?.parse().ok()?;
        let pm: u8 = parts.next()?.parse().ok()?;

        match ps {
            1016 => {
                // Mouse pixel reporting
                if pm != 0 && pm != 4 {
                    Some(TerminalEvent::CapabilityResponse("sgr_pixels".to_string()))
                } else {
                    None
                }
            }
            2027 => {
                // Unicode Core. The terminal's answer is the glyph capability
                // report the charts follow (CHT-028): set (1, 3) keeps braille
                // and block glyphs, reset (2, 4) switches them to ASCII, and
                // "not recognized" (0) reports nothing.
                match pm {
                    1 | 3 => {
                        crate::widgets::display::charts::report_glyph_support(true);
                        Some(TerminalEvent::CapabilityResponse("unicode".to_string()))
                    }
                    2 | 4 => {
                        crate::widgets::display::charts::report_glyph_support(false);
                        None
                    }
                    _ => None,
                }
            }
            2031 => {
                // Color scheme reporting
                if pm != 0 && pm != 4 {
                    Some(TerminalEvent::CapabilityResponse(
                        "color_scheme_updates".to_string(),
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Decode modifier mask to KeyModifiers
    fn decode_modifiers(&self, mask: u8) -> KeyModifiers {
        KeyModifiers {
            shift: (mask & 1) != 0,
            alt: (mask & 2) != 0,
            ctrl: (mask & 4) != 0,
            meta: (mask & 8) != 0,
        }
    }

    /// Map Kitty codepoint to KeyCode with full Unicode support
    fn map_kitty_codepoint_to_keycode(&self, codepoint: u32) -> KeyCode {
        match codepoint {
            // Control characters
            8 => KeyCode::Backspace,
            9 => KeyCode::Tab,
            13 => KeyCode::Enter,
            27 => KeyCode::Escape,
            127 => KeyCode::Delete,

            // Function keys (Kitty extended)
            57344 => KeyCode::F(1),  // F1
            57345 => KeyCode::F(2),  // F2
            57346 => KeyCode::F(3),  // F3
            57347 => KeyCode::F(4),  // F4
            57348 => KeyCode::F(5),  // F5
            57349 => KeyCode::F(6),  // F6
            57350 => KeyCode::F(7),  // F7
            57351 => KeyCode::F(8),  // F8
            57352 => KeyCode::F(9),  // F9
            57353 => KeyCode::F(10), // F10
            57354 => KeyCode::F(11), // F11
            57355 => KeyCode::F(12), // F12

            // Navigation keys (Kitty extended)
            57376 => KeyCode::Up,
            57377 => KeyCode::Down,
            57378 => KeyCode::Right,
            57379 => KeyCode::Left,
            57380 => KeyCode::Home,
            57381 => KeyCode::End,
            57382 => KeyCode::PageUp,
            57383 => KeyCode::PageDown,
            57384 => KeyCode::Insert,
            57385 => KeyCode::Delete,

            // Regular Unicode characters
            _ => {
                if let Some(ch) = char::from_u32(codepoint) {
                    KeyCode::Char(ch)
                } else {
                    KeyCode::Char('\u{FFFD}') // Replacement character
                }
            }
        }
    }
}

impl Default for EscapeSequenceParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = EscapeSequenceParser::new();
        assert!(parser.pending.is_empty());

        let default_parser = EscapeSequenceParser::default();
        assert!(default_parser.pending.is_empty());
    }

    /// CHT-028: the mode 2027 reply reaches the charts' glyph report; a
    /// reset reply turns glyph support off, a set reply turns it on.
    #[test]
    fn unicode_mode_reply_drives_the_chart_glyph_report() {
        use crate::widgets::display::charts::{
            glyph_support, report_glyph_support, GLYPH_REPORT_TEST_LOCK,
        };
        let _serial = GLYPH_REPORT_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut parser = EscapeSequenceParser::new();
        let reset = parser.parse(b"\x1b[?2027;2$y");
        let after_reset = glyph_support();
        let set = parser.parse(b"\x1b[?2027;1$y");
        let after_set = glyph_support();
        report_glyph_support(true);
        assert!(
            reset.is_empty(),
            "a reset reply is not a capability: {reset:?}"
        );
        assert!(!after_reset, "a reset reply must report glyphs unavailable");
        assert!(
            matches!(set.as_slice(), [TerminalEvent::CapabilityResponse(name)] if name == "unicode"),
            "a set reply is the unicode capability: {set:?}"
        );
        assert!(after_set, "a set reply must report glyphs available");
    }

    #[test]
    fn test_parse_result() {
        let result = ParseResult { event: None, n: 0 };
        assert!(result.event.is_none());
        assert_eq!(result.n, 0);
    }

    #[test]
    fn test_parser_state_enum() {
        let states = [
            ParserState::Ground,
            ParserState::Escape,
            ParserState::Csi,
            ParserState::Osc,
            ParserState::Dcs,
            ParserState::Sos,
            ParserState::Pm,
            ParserState::Apc,
            ParserState::Ss2,
            ParserState::Ss3,
        ];

        // Should be able to compare states
        assert_eq!(states[0], ParserState::Ground);
        assert_ne!(states[0], ParserState::Escape);
    }

    #[test]
    fn test_parse_empty_input() {
        let mut parser = EscapeSequenceParser::new();
        let events = parser.parse(&[]);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_parse_single_character() {
        let mut parser = EscapeSequenceParser::new();
        let events = parser.parse(b"a");
        assert_eq!(events.len(), 1);

        if let TerminalEvent::Key {
            code,
            modifiers,
            kind,
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('a'));
            assert_eq!(*modifiers, KeyModifiers::empty());
            assert_eq!(*kind, KeyEventKind::Press);
        } else {
            panic!("Expected key event");
        }
    }

    #[test]
    fn test_parse_multiple_characters() {
        let mut parser = EscapeSequenceParser::new();
        let events = parser.parse(b"abc");
        assert_eq!(events.len(), 3);

        let expected_chars = ['a', 'b', 'c'];
        for (i, event) in events.iter().enumerate() {
            if let TerminalEvent::Key { code, .. } = event {
                assert_eq!(*code, KeyCode::Char(expected_chars[i]));
            } else {
                panic!("Expected key event");
            }
        }
    }

    #[test]
    fn test_parse_special_keys() {
        let mut parser = EscapeSequenceParser::new();

        // Test backspace
        let events = parser.parse(&[0x08]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Backspace);
        }

        // Test tab
        let events = parser.parse(&[0x09]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Tab);
        }

        // Test enter (LF)
        let events = parser.parse(&[0x0A]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Enter);
        }

        // Test enter (CR)
        let events = parser.parse(&[0x0D]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Enter);
        }

        // Test escape
        let events = parser.parse(&[0x1B]);
        assert!(events.is_empty());
        let events = parser.flush_pending();
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Escape);
        }

        // Test DEL
        let events = parser.parse(&[0x7F]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Backspace);
        }
    }

    #[test]
    fn test_parse_ctrl_combinations() {
        let mut parser = EscapeSequenceParser::new();

        // Test Ctrl+A (0x01)
        let events = parser.parse(&[0x01]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key {
            code, modifiers, ..
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('a'));
            assert!(modifiers.ctrl);
        }

        // Test Ctrl+Z (0x1A)
        let events = parser.parse(&[0x1A]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key {
            code, modifiers, ..
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('z'));
            assert!(modifiers.ctrl);
        }

        // Test Ctrl+@ (0x00)
        let events = parser.parse(&[0x00]);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key {
            code, modifiers, ..
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('@'));
            assert!(modifiers.ctrl);
        }
    }

    #[test]
    fn test_parse_alt_combinations() {
        let mut parser = EscapeSequenceParser::new();

        // Test Alt+A (ESC + A)
        let events = parser.parse(&[0x1B, b'a']);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key {
            code, modifiers, ..
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('a'));
            assert!(modifiers.alt);
            assert!(!modifiers.ctrl);
        }

        // Test Alt+1 (ESC + 1)
        let events = parser.parse(&[0x1B, b'1']);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key {
            code, modifiers, ..
        } = &events[0]
        {
            assert_eq!(*code, KeyCode::Char('1'));
            assert!(modifiers.alt);
        }
    }

    #[test]
    fn test_parse_utf8_characters() {
        let mut parser = EscapeSequenceParser::new();

        // Test UTF-8 character (é)
        let utf8_bytes = "é".as_bytes();
        let events = parser.parse(utf8_bytes);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Char('é'));
        }

        // Test emoji (🚀)
        let emoji_bytes = "🚀".as_bytes();
        let events = parser.parse(emoji_bytes);
        assert_eq!(events.len(), 1);
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Char('🚀'));
        }
    }

    #[test]
    fn api019_split_utf8_escape_and_mouse_sequences_are_retained() {
        let mut parser = EscapeSequenceParser::new();
        assert!(parser.parse(&"🚀".as_bytes()[..2]).is_empty());
        let events = parser.parse(&"🚀".as_bytes()[2..]);
        assert!(matches!(
            events.as_slice(),
            [TerminalEvent::Key {
                code: KeyCode::Char('🚀'),
                ..
            }]
        ));

        assert!(parser.parse(b"\x1b[").is_empty());
        assert!(matches!(
            parser.parse(b"A").as_slice(),
            [TerminalEvent::Key {
                code: KeyCode::Up,
                ..
            }]
        ));

        assert!(parser.parse(b"\x1b[M").is_empty());
        assert!(parser.parse(&[32 | 2, 34]).is_empty());
        let events = parser.parse(&[35]);
        assert!(matches!(
            events.as_slice(),
            [TerminalEvent::Mouse {
                kind: MouseEventKind::Down,
                button: Some(MouseButton::Right),
                column: 1,
                row: 2,
                ..
            }]
        ));
    }

    #[test]
    fn api019_sgr_mouse_preserves_buttons_modifiers_drag_and_wheel() {
        let mut parser = EscapeSequenceParser::new();
        let right = parser.parse(b"\x1b[<18;4;5M");
        assert!(matches!(
            right.as_slice(),
            [TerminalEvent::Mouse {
                kind: MouseEventKind::Down,
                button: Some(MouseButton::Right),
                modifiers: KeyModifiers { ctrl: true, .. },
                ..
            }]
        ));
        let drag = parser.parse(b"\x1b[<33;4;5M");
        assert!(matches!(
            drag.as_slice(),
            [TerminalEvent::Mouse {
                kind: MouseEventKind::Drag,
                button: Some(MouseButton::Middle),
                ..
            }]
        ));
        let wheel = parser.parse(b"\x1b[<64;4;5M");
        assert!(matches!(
            wheel.as_slice(),
            [TerminalEvent::Mouse {
                kind: MouseEventKind::ScrollUp,
                ..
            }]
        ));
    }

    #[test]
    fn api019_incomplete_sequence_buffer_is_bounded_and_reports_overflow() {
        let mut parser = EscapeSequenceParser::new();
        assert!(parser.parse(b"\x1b]").is_empty());
        let events = parser.parse(&vec![b'x'; 4095]);
        assert!(
            matches!(events.as_slice(), [TerminalEvent::Error(message)] if message.contains("4096-byte"))
        );
        assert!(parser.pending.is_empty());
    }

    #[test]
    fn test_parse_mixed_input() {
        let mut parser = EscapeSequenceParser::new();

        // Mix of regular chars, special keys, and escape sequences
        let input = b"a\x08b\x09c";
        let events = parser.parse(input);
        assert_eq!(events.len(), 5);

        // Should be: 'a', Backspace, 'b', Tab, 'c'
        if let TerminalEvent::Key { code, .. } = &events[0] {
            assert_eq!(*code, KeyCode::Char('a'));
        }
        if let TerminalEvent::Key { code, .. } = &events[1] {
            assert_eq!(*code, KeyCode::Backspace);
        }
        if let TerminalEvent::Key { code, .. } = &events[2] {
            assert_eq!(*code, KeyCode::Char('b'));
        }
        if let TerminalEvent::Key { code, .. } = &events[3] {
            assert_eq!(*code, KeyCode::Tab);
        }
        if let TerminalEvent::Key { code, .. } = &events[4] {
            assert_eq!(*code, KeyCode::Char('c'));
        }
    }
}
