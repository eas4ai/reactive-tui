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
    /// OSC string processing
    OscString,
    /// DCS sequence entry
    DcsEntry,
    /// DCS parameter processing
    DcsParam,
    /// DCS intermediate bytes
    DcsIntermediate,
    /// DCS passthrough mode
    DcsPassthrough,
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
    utf8_decoder: Utf8Decoder,
}

impl AnsiParser {
    /// Maximum buffer sizes to prevent DoS attacks
    const MAX_PARAMS: usize = 32;        // CSI sequences rarely need more than 16
    const MAX_INTERMEDIATES: usize = 8;  // Intermediates are single bytes, rarely more than 2
    const MAX_STRING_BUFFER: usize = 8192; // OSC/DCS strings should be reasonable
    
    /// Create a new ANSI parser
    pub fn new() -> Self {
        Self {
            state: ParserState::Ground,
            params: Vec::new(),
            current_param: None,
            intermediates: Vec::new(),
            string_buffer: Vec::new(),
            private: false,
            utf8_decoder: Utf8Decoder::new(),
        }
    }

    /// Parse a single byte and return any resulting events
    pub fn parse(&mut self, byte: u8) -> Vec<AnsiEvent> {
        let mut events = Vec::new();

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
            ParserState::OscString => {
                self.parse_osc_string(byte, &mut events);
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
            0x20..=0x7F => {
                events.push(AnsiEvent::Print(byte as char));
            }
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
                self.parse_csi_param(byte, events);
            }
            0x3A => {
                self.reset_state();
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
                self.reset_state();
            }
            0x3B => {
                self.finalize_param();
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
            0x00..=0x06 | 0x08..=0x17 | 0x19 | 0x1C..=0x1F => {
                self.reset_state();
            }
            0x07 => {
                self.finalize_osc(events);
                self.reset_state();
            }
            0x1B => {
                if self.string_buffer.len() < Self::MAX_STRING_BUFFER {
                    self.string_buffer.push(byte);
                }
                // Silently drop if buffer is full
            }
            _ => {
                if self.string_buffer.ends_with(&[0x1B]) && byte == 0x5C {
                    self.string_buffer.pop();
                    self.finalize_osc(events);
                    self.reset_state();
                } else if self.string_buffer.len() < Self::MAX_STRING_BUFFER
                        && self.string_buffer.len() < Self::MAX_STRING_BUFFER {
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
                parts[1].split(';').map(|s| s.to_string()).collect()
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
                if self.params.len() < Self::MAX_PARAMS {
                    self.params.push(0);
                }
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
                self.state = ParserState::Escape;
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
        // Simple approach: if buffer is full, reset and start fresh
        if self.len >= 4 {
            self.len = 0;
        }

        self.buffer[self.len] = byte;
        self.len += 1;

        // Try to decode what we have so far
        match std::str::from_utf8(&self.buffer[..self.len]) {
            Ok(s) => {
                // Successfully decoded - return the character and reset
                if let Some(ch) = s.chars().next() {
                    self.len = 0;
                    Some(ch)
                } else {
                    // Empty string somehow - reset and continue
                    self.len = 0;
                    None
                }
            }
            Err(e) => {
                // Check if we have a valid prefix that we can decode
                if e.valid_up_to() > 0 {
                    if let Ok(valid_str) = std::str::from_utf8(&self.buffer[..e.valid_up_to()]) {
                        if let Some(ch) = valid_str.chars().next() {
                            // We found a valid character, shift remaining bytes
                            let remaining = self.len - e.valid_up_to();
                            if remaining > 0 {
                                self.buffer.copy_within(e.valid_up_to()..self.len, 0);
                                self.len = remaining;
                            } else {
                                self.len = 0;
                            }
                            return Some(ch);
                        }
                    }
                }

                // Check if this could be the start of a valid sequence
                if self.len == 1 {
                    // Single byte - check if it could be start of multi-byte sequence
                    match byte {
                        0x00..=0x7F => {
                            // ASCII - should have been decoded above, so it's invalid
                            self.len = 0;
                            None
                        }
                        0x80..=0xBF => {
                            // Continuation byte without start - invalid
                            self.len = 0;
                            None
                        }
                        0xC0..=0xDF => {
                            // Start of 2-byte sequence - wait for more
                            None
                        }
                        0xE0..=0xEF => {
                            // Start of 3-byte sequence - wait for more
                            None
                        }
                        0xF0..=0xF7 => {
                            // Start of 4-byte sequence - wait for more
                            None
                        }
                        0xF8..=0xFF => {
                            // Invalid UTF-8 start byte
                            self.len = 0;
                            None
                        }
                    }
                } else {
                    // Multi-byte sequence in progress
                    // If we're at max length and still invalid, reset
                    if self.len >= 4 {
                        self.len = 0;
                    }
                    None
                }
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
            assert_eq!(result, None, "Iteration {}: should return None for invalid continuation byte", i);
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
