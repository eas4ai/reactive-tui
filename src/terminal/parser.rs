//! ANSI escape sequence parser

/// Internal parser state for ANSI escape sequence parsing
#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserState {
    /// Normal character processing
    Ground,
    /// Escape sequence started
    Escape,
    /// CSI sequence entry
    CsiEntry,
    /// CSI parameter processing
    CsiParam,
    /// CSI intermediate bytes
    CsiIntermediate,
    /// Discard an unsupported CSI through its final byte
    CsiIgnore,
    /// OSC string processing
    OscString,
    /// Escape seen inside an OSC, independent of payload capacity
    OscEscape,
    /// DCS sequence entry
    DcsEntry,
    /// DCS parameter processing
    DcsParam,
    /// DCS intermediate bytes
    DcsIntermediate,
    /// DCS passthrough mode
    DcsPassthrough,
    /// Escape seen after a DCS payload
    DcsEscape,
    /// SOS string processing
    SosString,
    /// PM string processing
    PmString,
    /// APC string processing
    ApcString,
}

/// ANSI escape sequence events
#[derive(Debug, Clone, PartialEq)]
pub enum AnsiEvent {
    /// Print a single character
    Print(char),
    /// Print a string of characters
    PrintString(String),
    /// Execute a control character
    Execute(u8),
    /// Control Sequence Introducer command
    Csi {
        /// Final byte of the CSI sequence
        final_byte: char,
        /// Numeric parameters
        params: Vec<u16>,
        /// Intermediate bytes
        intermediates: Vec<u8>,
        /// Whether this is a private sequence
        private: bool,
    },
    /// Operating System Command
    Osc {
        /// OSC command string
        command: String,
        /// String parameters
        params: Vec<String>,
    },
    /// Device Control String
    Dcs {
        /// Final byte of the DCS sequence
        final_byte: char,
        /// Numeric parameters
        params: Vec<u16>,
        /// Intermediate bytes
        intermediates: Vec<u8>,
        /// Data payload
        data: Vec<u8>,
    },
    /// Bell character (0x07)
    Bell,
    /// Backspace character (0x08)
    Backspace,
    /// Tab character (0x09)
    Tab,
    /// Line feed character (0x0A)
    LineFeed,
    /// Vertical tab character (0x0B)
    VerticalTab,
    /// Form feed character (0x0C)
    FormFeed,
    /// Carriage return character (0x0D)
    CarriageReturn,
}

/// ANSI escape sequence parser
#[derive(Debug)]
pub struct AnsiParser {
    state: ParserState,
    params: Vec<u16>,
    current_param: Option<u16>,
    intermediates: Vec<u8>,
    string_buffer: Vec<u8>,
    private: bool,
    dcs_final: u8,
    utf8_decoder: Utf8Decoder,
}

impl AnsiParser {
    /// Maximum buffer sizes to prevent DoS attacks
    const MAX_PARAMS: usize = 32; // CSI sequences rarely need more than 16
    const MAX_INTERMEDIATES: usize = 8; // Intermediates are single bytes, rarely more than 2
    pub(crate) const MAX_STRING_BUFFER: usize = 8192; // OSC/DCS strings should be reasonable

    /// Create a new ANSI parser
    pub fn new() -> Self {
        Self {
            state: ParserState::Ground,
            params: Vec::new(),
            current_param: None,
            intermediates: Vec::new(),
            string_buffer: Vec::new(),
            private: false,
            dcs_final: 0,
            utf8_decoder: Utf8Decoder::new(),
        }
    }

    /// Parse a single byte and return any resulting events
    pub fn parse(&mut self, byte: u8) -> Vec<AnsiEvent> {
        let mut events = Vec::new();
        if matches!(byte, 0x18 | 0x1A) {
            self.reset_state();
            self.utf8_decoder.len = 0;
            self.execute_control(byte, &mut events);
            return events;
        }
        if byte == 0x1B
            && matches!(
                self.state,
                ParserState::Escape
                    | ParserState::CsiEntry
                    | ParserState::CsiParam
                    | ParserState::CsiIntermediate
                    | ParserState::CsiIgnore
                    | ParserState::DcsEntry
                    | ParserState::DcsParam
                    | ParserState::DcsIntermediate
            )
        {
            self.reset_buffers();
            self.state = ParserState::Escape;
            return events;
        }

        match self.state {
            ParserState::Ground => {
                self.parse_ground(byte, &mut events);
            }
            ParserState::Escape => {
                self.parse_escape(byte, &mut events);
            }
            ParserState::CsiEntry => {
                self.parse_csi_entry(byte, &mut events);
            }
            ParserState::CsiParam => {
                self.parse_csi_param(byte, &mut events);
            }
            ParserState::CsiIntermediate => {
                self.parse_csi_intermediate(byte, &mut events);
            }
            ParserState::CsiIgnore => {
                if (0x40..=0x7E).contains(&byte) {
                    self.reset_state();
                } else if byte < 0x20 {
                    self.execute_control(byte, &mut events);
                }
            }
            ParserState::OscString => {
                self.parse_osc_string(byte, &mut events);
            }
            ParserState::OscEscape => {
                if byte == b'\\' {
                    self.finalize_osc(&mut events);
                    self.reset_state();
                } else {
                    self.reset_buffers();
                    self.state = ParserState::Escape;
                    self.parse_escape(byte, &mut events);
                }
            }
            ParserState::DcsEscape => {
                if byte == b'\\' {
                    events.push(AnsiEvent::Dcs {
                        final_byte: self.dcs_final as char,
                        params: std::mem::take(&mut self.params),
                        intermediates: std::mem::take(&mut self.intermediates),
                        data: std::mem::take(&mut self.string_buffer),
                    });
                    self.reset_state();
                } else {
                    self.reset_buffers();
                    self.state = ParserState::Escape;
                    self.parse_escape(byte, &mut events);
                }
            }
            ParserState::DcsEntry => {
                self.parse_dcs_entry(byte, &mut events);
            }
            ParserState::DcsParam => {
                self.parse_dcs_param(byte, &mut events);
            }
            ParserState::DcsIntermediate => {
                self.parse_dcs_intermediate(byte, &mut events);
            }
            ParserState::DcsPassthrough => {
                self.parse_dcs_passthrough(byte, &mut events);
            }
            _ => {
                // Handle other string states (SOS, PM, APC)
                self.parse_string_state(byte, &mut events);
            }
        }

        events
    }

    /// Parse multiple bytes and return all resulting events
    pub fn parse_bytes(&mut self, bytes: &[u8]) -> Vec<AnsiEvent> {
        let mut events = Vec::new();
        for &byte in bytes {
            events.extend(self.parse(byte));
        }
        events
    }

    fn parse_ground(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        if byte < 0x80 {
            self.utf8_decoder.len = 0;
        }
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x18 | 0x1A => {
                self.reset_state();
                self.execute_control(byte, events);
            }
            0x1B => {
                self.state = ParserState::Escape;
                self.reset_buffers();
            }
            0x20..=0x7E => {
                events.push(AnsiEvent::Print(byte as char));
            }
            0x7F => {}
            0x80..=0xFF => {
                if let Some(ch) = self.utf8_decoder.decode(byte) {
                    events.push(AnsiEvent::Print(ch));
                }
            }
        }
    }

    fn parse_escape(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
            }
            0x30..=0x4F | 0x51..=0x57 | 0x59 | 0x5A | 0x5C | 0x60..=0x7E => {
                self.execute_escape(byte, events);
                self.reset_state();
            }
            0x50 => {
                self.state = ParserState::DcsEntry;
            }
            0x5B => {
                self.state = ParserState::CsiEntry;
            }
            0x5D => {
                self.state = ParserState::OscString;
            }
            0x58 | 0x5E | 0x5F => {
                self.state = match byte {
                    0x58 => ParserState::SosString,
                    0x5E => ParserState::PmString,
                    0x5F => ParserState::ApcString,
                    _ => unreachable!(),
                };
            }
            _ => {
                self.reset_state();
            }
        }
    }

    fn parse_csi_entry(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
                self.state = ParserState::CsiIntermediate;
            }
            0x30..=0x39 | 0x3B => {
                self.state = ParserState::CsiParam;
                self.parse_csi_param(byte, events);
            }
            0x3A => {
                self.state = ParserState::CsiIgnore;
            }
            0x3C..=0x3F => {
                self.private = true;
                self.state = ParserState::CsiParam;
            }
            0x40..=0x7E => {
                self.finalize_param();
                events.push(AnsiEvent::Csi {
                    final_byte: byte as char,
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    private: self.private,
                });
                self.reset_state();
            }
            _ => {
                self.reset_state();
            }
        }
    }

    fn parse_csi_param(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                self.finalize_param();
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
                self.state = ParserState::CsiIntermediate;
            }
            0x30..=0x39 => {
                let digit = (byte - 0x30) as u16;
                self.current_param = Some(
                    self.current_param
                        .unwrap_or(0)
                        .saturating_mul(10)
                        .saturating_add(digit),
                );
            }
            0x3A => {
                self.state = ParserState::CsiIgnore;
            }
            0x3B => {
                self.current_param.get_or_insert(0);
                self.finalize_param();
                self.current_param = Some(0);
            }
            0x3C..=0x3F => {
                self.reset_state();
            }
            0x40..=0x7E => {
                self.finalize_param();
                events.push(AnsiEvent::Csi {
                    final_byte: byte as char,
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    private: self.private,
                });
                self.reset_state();
            }
            _ => {
                self.reset_state();
            }
        }
    }

    fn parse_csi_intermediate(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
            }
            0x40..=0x7E => {
                events.push(AnsiEvent::Csi {
                    final_byte: byte as char,
                    params: self.params.clone(),
                    intermediates: self.intermediates.clone(),
                    private: self.private,
                });
                self.reset_state();
            }
            _ => {
                self.reset_state();
            }
        }
    }

    fn parse_osc_string(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x18 | 0x1A => self.reset_state(),
            0x07 => {
                self.finalize_osc(events);
                self.reset_state();
            }
            0x1B => self.state = ParserState::OscEscape,
            // Embedded controls do not turn the remainder of a title into text.
            0x00..=0x1F => {}
            _ => {
                if self.string_buffer.len() < Self::MAX_STRING_BUFFER {
                    self.string_buffer.push(byte);
                }
            }
        }
    }

    fn execute_control(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x07 => events.push(AnsiEvent::Bell),
            0x08 => events.push(AnsiEvent::Backspace),
            0x09 => events.push(AnsiEvent::Tab),
            0x0A => events.push(AnsiEvent::LineFeed),
            0x0B => events.push(AnsiEvent::VerticalTab),
            0x0C => events.push(AnsiEvent::FormFeed),
            0x0D => events.push(AnsiEvent::CarriageReturn),
            _ => events.push(AnsiEvent::Execute(byte)),
        }
    }

    fn execute_escape(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        // Intermediate bytes identify a different escape command (for example
        // charset designation). Do not execute its final byte as bare ESC D/M/c.
        if !self.intermediates.is_empty() {
            return;
        }
        match byte {
            0x37 => {
                events.push(AnsiEvent::Csi {
                    final_byte: 's',
                    params: vec![],
                    intermediates: vec![],
                    private: false,
                });
            }
            0x38 => {
                events.push(AnsiEvent::Csi {
                    final_byte: 'u',
                    params: vec![],
                    intermediates: vec![],
                    private: false,
                });
            }
            _ => {
                events.push(AnsiEvent::Execute(byte));
            }
        }
    }

    fn finalize_param(&mut self) {
        // Limit params to prevent DoS
        if self.params.len() >= Self::MAX_PARAMS {
            return;
        }

        if let Some(param) = self.current_param.take() {
            self.params.push(param);
        } else if !self.params.is_empty() || self.current_param.is_some() {
            self.params.push(0);
        }
    }

    fn finalize_osc(&mut self, events: &mut Vec<AnsiEvent>) {
        let string = String::from_utf8_lossy(&self.string_buffer);
        let parts: Vec<&str> = string.splitn(2, ';').collect();

        if let Some(command) = parts.first() {
            let params = if parts.len() > 1 {
                match *command {
                    "0" | "1" | "2" => {
                        vec![super::sanitize_host_text(parts[1]).into_owned()]
                    }
                    "7" => vec![parts[1].to_string()],
                    "8" => parts[1].splitn(2, ';').map(str::to_string).collect(),
                    _ => parts[1].split(';').map(str::to_string).collect(),
                }
            } else {
                vec![]
            };

            events.push(AnsiEvent::Osc {
                command: command.to_string(),
                params,
            });
        }
    }

    fn reset_state(&mut self) {
        self.state = ParserState::Ground;
        self.reset_buffers();
    }

    fn reset_buffers(&mut self) {
        self.params.clear();
        self.current_param = None;
        self.intermediates.clear();
        self.string_buffer.clear();
        self.private = false;
        self.dcs_final = 0;
    }

    /// Parse DCS entry state - Device Control String entry
    fn parse_dcs_entry(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
                self.state = ParserState::DcsIntermediate;
            }
            0x30..=0x39 | 0x3B => {
                self.current_param = Some(0);
                self.state = ParserState::DcsParam;
                self.parse_dcs_param(byte, events);
            }
            0x3A => {
                self.state = ParserState::DcsParam;
            }
            0x3C..=0x3F => {
                self.private = true;
                self.state = ParserState::DcsParam;
            }
            0x40..=0x7E => {
                self.dcs_final = byte;
                self.state = ParserState::DcsPassthrough;
            }
            _ => {
                self.reset_state();
            }
        }
    }

    /// Parse DCS parameter state
    fn parse_dcs_param(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                self.finalize_param();
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
                self.state = ParserState::DcsIntermediate;
            }
            0x30..=0x39 => {
                if let Some(ref mut param) = self.current_param {
                    *param = param
                        .saturating_mul(10)
                        .saturating_add((byte - b'0') as u16);
                }
            }
            0x3A => {
                // Sub-parameter separator - not commonly used
            }
            0x3B => {
                if self.params.len() < Self::MAX_PARAMS {
                    self.params.push(self.current_param.unwrap_or(0));
                }
                self.current_param = Some(0);
            }
            0x3C..=0x3F => {
                self.private = true;
            }
            0x40..=0x7E => {
                if let Some(param) = self.current_param {
                    if self.params.len() < Self::MAX_PARAMS {
                        self.params.push(param);
                    }
                }
                self.dcs_final = byte;
                self.state = ParserState::DcsPassthrough;
            }
            _ => {
                self.reset_state();
            }
        }
    }

    /// Parse DCS intermediate state
    fn parse_dcs_intermediate(&mut self, byte: u8, events: &mut Vec<AnsiEvent>) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.execute_control(byte, events);
            }
            0x20..=0x2F => {
                if self.intermediates.len() < Self::MAX_INTERMEDIATES {
                    self.intermediates.push(byte);
                }
                // Silently drop if buffer is full
            }
            0x40..=0x7E => {
                self.dcs_final = byte;
                self.state = ParserState::DcsPassthrough;
            }
            _ => {
                self.reset_state();
            }
        }
    }

    /// Parse DCS passthrough state - collect DCS data
    fn parse_dcs_passthrough(&mut self, byte: u8, _events: &mut [AnsiEvent]) {
        match byte {
            0x00..=0x17 | 0x19 | 0x1C..=0x1F => {
                // Control characters in passthrough - ignore for now
            }
            0x1B => {
                // Escape - might be end of DCS
                self.state = ParserState::DcsEscape;
            }
            _ => {
                // Collect DCS data
                if self.string_buffer.len() < Self::MAX_STRING_BUFFER {
                    self.string_buffer.push(byte);
                }
                // Silently drop if buffer is full
            }
        }
    }

    /// Parse other string states (SOS, PM, APC)
    fn parse_string_state(&mut self, byte: u8, _events: &mut [AnsiEvent]) {
        match byte {
            0x1B => {
                // Escape - end of string
                self.state = ParserState::Escape;
            }
            0x07 => {
                // Bell - end of string (alternative terminator)
                self.reset_state();
            }
            _ => {
                // Collect string data
                if self.string_buffer.len() < Self::MAX_STRING_BUFFER {
                    self.string_buffer.push(byte);
                }
                // Silently drop if buffer is full
            }
        }
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Utf8Decoder {
    buffer: [u8; 4],
    len: usize,
}

impl Utf8Decoder {
    fn new() -> Self {
        Self {
            buffer: [0; 4],
            len: 0,
        }
    }

    fn decode(&mut self, byte: u8) -> Option<char> {
        // A fresh leading byte ends any incomplete prior character.
        if !(0x80..=0xBF).contains(&byte) {
            self.len = 0;
        }
        if self.len == 0 && !matches!(byte, 0x00..=0x7F | 0xC2..=0xF4) {
            return None;
        }
        self.buffer[self.len] = byte;
        self.len += 1;
        match std::str::from_utf8(&self.buffer[..self.len]) {
            Ok(text) => {
                let character = text.chars().next();
                self.len = 0;
                character
            }
            Err(error) => {
                if error.error_len().is_some() || self.len == self.buffer.len() {
                    self.len = 0;
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = AnsiParser::new();
        assert_eq!(parser.state, ParserState::Ground);
    }

    #[test]
    fn test_parse_simple_text() {
        let mut parser = AnsiParser::new();
        let events = parser.parse_bytes(b"Hello");

        assert_eq!(events.len(), 5);
        for (i, &ch) in b"Hello".iter().enumerate() {
            assert_eq!(events[i], AnsiEvent::Print(ch as char));
        }
    }

    #[test]
    fn test_parse_control_characters() {
        let mut parser = AnsiParser::new();

        let events = parser.parse(0x07);
        assert_eq!(events, vec![AnsiEvent::Bell]);

        let events = parser.parse(0x08);
        assert_eq!(events, vec![AnsiEvent::Backspace]);

        let events = parser.parse(0x0A);
        assert_eq!(events, vec![AnsiEvent::LineFeed]);
    }

    #[test]
    fn test_parse_csi_sequence() {
        let mut parser = AnsiParser::new();
        let events = parser.parse_bytes(b"\x1b[2J");

        assert_eq!(events.len(), 1);
        match &events[0] {
            AnsiEvent::Csi {
                final_byte, params, ..
            } => {
                assert_eq!(*final_byte, 'J');
                assert_eq!(*params, vec![2]);
            }
            _ => panic!("Expected CSI event"),
        }
    }

    #[test]
    fn test_parse_osc_sequence() {
        let mut parser = AnsiParser::new();
        let events = parser.parse_bytes(b"\x1b]0;Terminal Title\x07");

        assert_eq!(events.len(), 1);
        match &events[0] {
            AnsiEvent::Osc { command, params } => {
                assert_eq!(command, "0");
                assert_eq!(params, &vec!["Terminal Title".to_string()]);
            }
            _ => panic!("Expected OSC event"),
        }
    }

    #[test]
    fn test_utf8_decoder_buffer_overflow_protection() {
        let mut decoder = Utf8Decoder::new();

        // Test that we don't overflow when buffer is full
        // Try to decode 5 invalid bytes (more than buffer size of 4)
        // This should not panic or cause buffer overflow
        for _ in 0..5 {
            decoder.decode(0xFF); // Invalid UTF-8 byte
        }

        // Decoder should still be functional after overflow attempt
        // The fact that this works proves the buffer was reset
        // Test with valid UTF-8 sequence
        assert_eq!(decoder.decode(0x41), Some('A')); // ASCII 'A'

        // Test with multi-byte UTF-8 sequence
        assert_eq!(decoder.decode(0xC3), None); // First byte of 2-byte sequence
        assert_eq!(decoder.decode(0xA9), Some('é')); // Second byte completes 'é'

        // Test continuous invalid bytes don't cause overflow
        // Send many continuation bytes without start byte
        for i in 0..10 {
            let result = decoder.decode(0x80); // Continuation byte without start byte
            assert_eq!(
                result, None,
                "Iteration {}: should return None for invalid continuation byte",
                i
            );
            // We can't check len directly as it's private, but no panic means no overflow
        }

        // Verify decoder still works after all the invalid input
        assert_eq!(decoder.decode(0x42), Some('B')); // ASCII 'B'
    }

    #[test]
    fn test_utf8_decoder_valid_sequence_recovery() {
        let mut decoder = Utf8Decoder::new();

        // Test that valid UTF-8 is extracted even from mixed invalid data
        decoder.decode(0xC3); // Start of 2-byte sequence
        decoder.decode(0xA9); // Valid completion: 'é'

        // Add invalid bytes that would previously cause issues
        decoder.decode(0xFF);
        decoder.decode(0xFF);
        decoder.decode(0xFF);
        decoder.decode(0xFF);

        // Should still be able to decode valid sequences
        assert_eq!(decoder.decode(0x43), Some('C')); // ASCII 'C'
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn dcs_emits_payload_and_resumes_text_after_terminator() {
        let mut parser = AnsiParser::new();
        assert_eq!(
            parser.parse_bytes(b"\x1bP1;2 qabc\x1b\\X"),
            vec![
                AnsiEvent::Dcs {
                    final_byte: 'q',
                    params: vec![1, 2],
                    intermediates: vec![b' '],
                    data: b"abc".to_vec()
                },
                AnsiEvent::Print('X'),
            ]
        );
        parser.parse_bytes(b"\x1bPq");
        parser.parse_bytes(&vec![b'a'; AnsiParser::MAX_STRING_BUFFER + 10]);
        let events = parser.parse_bytes(b"\x1b\\X");
        assert!(
            matches!(&events[0], AnsiEvent::Dcs { data, .. } if data.len() == AnsiParser::MAX_STRING_BUFFER)
        );
        assert_eq!(events.last(), Some(&AnsiEvent::Print('X')));
    }

    #[test]
    fn csi_recovers_after_cancellation_and_preserves_empty_parameters() {
        let mut parser = AnsiParser::new();
        assert_eq!(
            parser.parse_bytes(b"\x1b[9\x1b[;2H"),
            vec![AnsiEvent::Csi {
                final_byte: 'H',
                params: vec![0, 2],
                intermediates: vec![],
                private: false,
            }]
        );
        assert_eq!(
            parser.parse_bytes(b"\x1b[38:2:1:2:3mX"),
            vec![AnsiEvent::Print('X')]
        );
        assert_eq!(
            parser.parse_bytes(b"\x1b[12 q"),
            vec![AnsiEvent::Csi {
                final_byte: 'q',
                params: vec![12],
                intermediates: vec![b' '],
                private: false,
            }]
        );
        let events = parser.parse_bytes(b"\x1b[123\x18X");
        assert_eq!(events.last(), Some(&AnsiEvent::Print('X')));
        assert!(!events
            .iter()
            .any(|event| matches!(event, AnsiEvent::Csi { .. })));
    }

    #[test]
    fn interrupted_utf8_does_not_poison_following_characters() {
        for prefix in [&b"\xc3X"[..], &b"\xe2\x1b[0m"[..], &b"\xf0\xff"[..]] {
            let mut parser = AnsiParser::new();
            parser.parse_bytes(prefix);
            assert_eq!(
                parser.parse_bytes("é界".as_bytes()),
                vec![AnsiEvent::Print('é'), AnsiEvent::Print('界')]
            );
        }
        assert!(AnsiParser::new().parse(0x7f).is_empty());
    }

    #[test]
    fn oversized_osc_terminates_without_consuming_following_text() {
        for length in [
            AnsiParser::MAX_STRING_BUFFER,
            AnsiParser::MAX_STRING_BUFFER + 10,
        ] {
            let mut parser = AnsiParser::new();
            parser.parse_bytes(b"\x1b]0;");
            parser.parse_bytes(&vec![b'a'; length]);
            let events = parser.parse_bytes(b"\x1b\\OK");
            assert!(events.ends_with(&[AnsiEvent::Print('O'), AnsiEvent::Print('K')]));
            assert_eq!(parser.state, ParserState::Ground);
            assert!(parser.string_buffer.is_empty());
        }
    }

    #[test]
    fn osc_ignores_controls_and_allows_escape_to_start_new_sequence() {
        let mut parser = AnsiParser::new();
        assert_eq!(
            parser.parse_bytes(b"\x1b]0;ti\x01tle\x07"),
            vec![AnsiEvent::Osc {
                command: "0".into(),
                params: vec!["title".into()],
            }]
        );
        assert_eq!(
            parser.parse_bytes(b"\x1b]0;discard\x1b[2JX"),
            vec![
                AnsiEvent::Csi {
                    final_byte: 'J',
                    params: vec![2],
                    intermediates: vec![],
                    private: false
                },
                AnsiEvent::Print('X'),
            ]
        );
    }
}
