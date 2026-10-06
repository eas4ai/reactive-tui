use super::{csi::CSIAction, esc::ESCAction, osc::OSCAction, Action};
use std::mem;

/// Maximum sizes for buffers to prevent DoS attacks
const MAX_OSC_STRING_SIZE: usize = 8192; // 8KB limit for OSC strings
const MAX_DCS_STRING_SIZE: usize = 16384; // 16KB limit for DCS strings
const MAX_INTERMEDIATE_SIZE: usize = 8; // VT standard allows max 2 intermediates
const MAX_PARAMS_SIZE: usize = 32; // VT standard allows max 16 params

/// VT parser state machine for ANSI escape sequences
/// Based on Paul Williams' state machine design: <https://vt100.net/emu/dec_ansi_parser>
///
/// PLT-005: the input is streaming UTF-8 in every state. A byte at or above
/// 0x80 is part of a character or an invalid byte (printed as U+FFFD, or
/// appended to the string being collected); the parser has no 8-bit C1
/// controls, which cannot be told from UTF-8 continuation bytes.
pub struct Parser {
    state: State,
    intermediate_bytes: Vec<u8>,
    params: Vec<u16>,
    osc_string: Vec<u8>,
    dcs_string: Vec<u8>,
    current_param: Option<u16>,
    actions: Vec<Action>,
    utf8_buffer: [u8; 4],
    utf8_len: usize,
    string_parent: State,
    string_kind: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
    Ground,
    Escape,
    EscapeIntermediate,
    CSIEntry,
    CSIParam,
    CSIIntermediate,
    CSIIgnore,
    DCSEntry,
    DCSParam,
    DCSIntermediate,
    DCSPassthrough,
    DCSIgnore,
    OSCString,
    SOSPMAPCString,
    StringEscape,
}

impl Parser {
    /// Create a new ANSI escape sequence parser
    pub fn new() -> Self {
        Self {
            state: State::Ground,
            intermediate_bytes: Vec::with_capacity(2),
            params: Vec::with_capacity(16),
            osc_string: Vec::with_capacity(256),
            dcs_string: Vec::with_capacity(1024),
            current_param: None,
            actions: Vec::with_capacity(16),
            utf8_buffer: [0; 4],
            utf8_len: 0,
            string_parent: State::Ground,
            string_kind: 0,
        }
    }

    /// Feed bytes to the parser and get back actions
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Action> {
        self.actions.clear();

        for &byte in bytes {
            self.process_byte(byte);
        }

        mem::take(&mut self.actions)
    }

    /// Process a single byte through the state machine
    fn process_byte(&mut self, byte: u8) {
        // PLT-005: bytes at or above 0x80 are UTF-8 in every state, so no
        // continuation byte can start a control sequence and no state loses
        // a character.
        // A byte that broke the character being collected is handled again
        // on its own below.
        if self.utf8_len > 0 && self.collect_utf8(byte) {
            return;
        }
        if (0xC2..=0xF4).contains(&byte) {
            self.utf8_buffer[0] = byte;
            self.utf8_len = 1;
            return;
        }
        if byte >= 0x80 {
            self.deliver_char('\u{FFFD}');
            return;
        }
        if self.consume_string_escape(byte) {
            return;
        }
        // C0 control characters are handled specially in most states
        if byte < 0x20 {
            match byte {
                0x00 => return, // NUL - ignore
                0x07 => {
                    // BEL - Bell
                    if self.state == State::OSCString {
                        self.osc_end();
                    } else {
                        self.actions.push(Action::Execute(byte));
                    }
                    return;
                }
                0x08..=0x0F => {
                    // Backspace, Tab, LF, VT, FF, CR, SO, SI
                    self.actions.push(Action::Execute(byte));
                    return;
                }
                0x18 | 0x1A => {
                    // CAN, SUB - Cancel sequence
                    self.transition_to(State::Ground);
                    return;
                }
                0x1B => {
                    self.start_escape();
                    return;
                }
                _ => {} // Other C0 controls - ignore in most states
            }
        }

        // State-specific processing
        match self.state {
            State::Ground => self.ground(byte),
            State::Escape => self.escape(byte),
            State::EscapeIntermediate => self.escape_intermediate(byte),
            State::CSIEntry => self.csi_entry(byte),
            State::CSIParam => self.csi_param(byte),
            State::CSIIntermediate => self.csi_intermediate(byte),
            State::CSIIgnore => self.csi_ignore(byte),
            State::DCSEntry => self.dcs_entry(byte),
            State::DCSParam => self.dcs_param(byte),
            State::DCSIntermediate => self.dcs_intermediate(byte),
            State::DCSPassthrough => self.dcs_passthrough(byte),
            State::DCSIgnore => self.dcs_ignore(byte),
            State::OSCString => self.osc_string(byte),
            State::SOSPMAPCString => self.sos_pm_apc_string(byte),
            State::StringEscape => unreachable!("handled before global controls"),
        }
    }

    /// Adds `byte` to the character being collected: true when the byte
    /// belongs to it (complete or not), false when it broke it, after
    /// delivering U+FFFD for the broken prefix; the byte is then handled
    /// again on its own (PLT-005).
    fn collect_utf8(&mut self, byte: u8) -> bool {
        self.utf8_buffer[self.utf8_len] = byte;
        self.utf8_len += 1;
        match std::str::from_utf8(&self.utf8_buffer[..self.utf8_len]) {
            Ok(text) => {
                let character = text.chars().next().unwrap();
                self.utf8_len = 0;
                self.deliver_char(character);
                true
            }
            Err(error) if error.error_len().is_none() => true,
            Err(_) => {
                self.utf8_len = 0;
                self.deliver_char('\u{FFFD}');
                false
            }
        }
    }

    /// A decoded character (or U+FFFD) in the current state: printed in the
    /// ground state, appended to the string being collected, and otherwise
    /// ending the sequence it interrupts before it prints (PLT-005).
    fn deliver_char(&mut self, character: char) {
        let mut encoded = [0u8; 4];
        let bytes = character.encode_utf8(&mut encoded).as_bytes();
        match self.state {
            State::Ground => self.actions.push(Action::Print(character)),
            State::OSCString | State::SOSPMAPCString => {
                if self.osc_string.len() + bytes.len() <= MAX_OSC_STRING_SIZE {
                    self.osc_string.extend_from_slice(bytes);
                }
            }
            State::DCSPassthrough => {
                if self.dcs_string.len() + bytes.len() <= MAX_DCS_STRING_SIZE {
                    self.dcs_string.extend_from_slice(bytes);
                }
            }
            State::DCSIgnore => {}
            State::StringEscape => {
                // An ESC inside a string followed by text aborts the string.
                self.osc_string.clear();
                self.dcs_string.clear();
                self.transition_to(State::Ground);
                self.actions.push(Action::Print(character));
            }
            _ => {
                self.transition_to(State::Ground);
                self.actions.push(Action::Print(character));
            }
        }
    }

    fn consume_string_escape(&mut self, byte: u8) -> bool {
        // PLT-006: string ESC is pending independently of payload capacity.
        if self.state != State::StringEscape {
            return false;
        }
        if byte == b'\\' {
            match self.string_parent {
                State::OSCString => self.osc_end(),
                State::DCSPassthrough => self.dcs_dispatch(),
                State::SOSPMAPCString => self.sos_pm_apc_string(0x9C),
                _ => {}
            }
            self.transition_to(State::Ground);
            return true;
        }
        // VT500: ESC followed by anything but backslash aborts OSC.
        // Other strings also resume the global escape transition.
        self.osc_string.clear();
        self.dcs_string.clear();
        self.transition_to(State::Escape);
        false
    }

    fn start_escape(&mut self) {
        match self.state {
            State::OSCString | State::DCSPassthrough | State::DCSIgnore | State::SOSPMAPCString => {
                self.string_parent = self.state;
                self.state = State::StringEscape;
            }
            _ => self.transition_to(State::Escape),
        }
    }

    fn transition_to(&mut self, new_state: State) {
        // Clear intermediate state when entering new sequence
        match new_state {
            State::Escape => {
                self.intermediate_bytes.clear();
            }
            State::CSIEntry => {
                self.params.clear();
                self.intermediate_bytes.clear();
                self.current_param = None;
            }
            State::DCSEntry => {
                self.params.clear();
                self.intermediate_bytes.clear();
                self.dcs_string.clear();
                self.current_param = None;
            }
            State::OSCString | State::SOSPMAPCString => {
                self.osc_string.clear();
            }
            _ => {}
        }
        self.state = new_state;
    }

    fn ground(&mut self, byte: u8) {
        if (0x20..0x7F).contains(&byte) {
            self.actions.push(Action::Print(byte as char));
        }
    }

    fn escape(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // Intermediate bytes
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                        if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                            self.intermediate_bytes.push(byte);
                        } else {
                            #[cfg(debug_assertions)]
                            log::warn!("intermediate_bytes buffer full; dropping byte");
                        }
                    }
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::EscapeIntermediate;
            }
            0x30..=0x4F | 0x51..=0x57 | 0x59 | 0x5A | 0x5C | 0x60..=0x7E => {
                // Final byte
                self.esc_dispatch(byte);
                self.transition_to(State::Ground);
            }
            0x50 => {
                // DCS
                self.transition_to(State::DCSEntry);
            }
            0x58 | 0x5E | 0x5F => {
                // SOS, PM, APC
                self.string_kind = match byte {
                    0x58 => 0x98,
                    0x5E => 0x9E,
                    _ => 0x9F,
                };
                self.transition_to(State::SOSPMAPCString);
            }
            0x5B => {
                // CSI
                self.transition_to(State::CSIEntry);
            }
            0x5D => {
                // OSC
                self.transition_to(State::OSCString);
            }
            _ => {
                // Invalid
                self.transition_to(State::Ground);
            }
        }
    }

    fn escape_intermediate(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // More intermediate bytes
                if self.intermediate_bytes.len() < 2
                    && self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE
                {
                    if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                        self.intermediate_bytes.push(byte);
                    } else {
                        #[cfg(debug_assertions)]
                        log::warn!("intermediate_bytes buffer full; dropping byte");
                    }
                }
            }
            0x30..=0x7E => {
                // Final byte
                self.esc_dispatch(byte);
                self.transition_to(State::Ground);
            }
            _ => {
                // Invalid
                self.transition_to(State::Ground);
            }
        }
    }

    fn csi_entry(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // Intermediate bytes
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::CSIIntermediate;
            }
            0x30..=0x39 => {
                // Parameter byte - digit
                self.param_digit(byte - b'0');
                self.state = State::CSIParam;
            }
            0x3A => {
                // Sub-parameter delimiter (ignored for now)
                self.state = State::CSIIgnore;
            }
            0x3B => {
                // Parameter delimiter
                self.param_separator();
                self.state = State::CSIParam;
            }
            0x3C..=0x3F => {
                // Private marker
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::CSIParam;
            }
            0x40..=0x7E => {
                // Final byte
                self.csi_dispatch(byte);
                self.transition_to(State::Ground);
            }
            _ => {
                // Invalid
                self.state = State::CSIIgnore;
            }
        }
    }

    fn csi_param(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // Intermediate bytes
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::CSIIntermediate;
            }
            0x30..=0x39 => {
                // Parameter digit
                self.param_digit(byte - b'0');
            }
            0x3A => {
                // Sub-parameter delimiter (ignored)
                self.state = State::CSIIgnore;
            }
            0x3B => {
                // Parameter separator
                self.param_separator();
            }
            0x3C..=0x3F => {
                // Should not happen in param state
                self.state = State::CSIIgnore;
            }
            0x40..=0x7E => {
                // Final byte
                self.csi_dispatch(byte);
                self.transition_to(State::Ground);
            }
            _ => {
                // Invalid
                self.state = State::CSIIgnore;
            }
        }
    }

    fn csi_intermediate(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // More intermediate bytes
                if self.intermediate_bytes.len() < 2
                    && self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE
                {
                    if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                        self.intermediate_bytes.push(byte);
                    } else {
                        #[cfg(debug_assertions)]
                        log::warn!("intermediate_bytes buffer full; dropping byte");
                    }
                }
            }
            0x30..=0x3F => {
                // Invalid in intermediate
                self.state = State::CSIIgnore;
            }
            0x40..=0x7E => {
                // Final byte
                self.csi_dispatch(byte);
                self.transition_to(State::Ground);
            }
            _ => {
                // Invalid
                self.state = State::CSIIgnore;
            }
        }
    }

    fn csi_ignore(&mut self, byte: u8) {
        // Ignore everything until final byte
        if (0x40..=0x7E).contains(&byte) {
            self.transition_to(State::Ground);
        }
    }

    fn dcs_entry(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // Intermediate bytes
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::DCSIntermediate;
            }
            0x30..=0x39 => {
                // Parameter digit
                self.param_digit(byte - b'0');
                self.state = State::DCSParam;
            }
            0x3A => {
                // Sub-parameter delimiter
                self.state = State::DCSIgnore;
            }
            0x3B => {
                // Parameter separator
                self.param_separator();
                self.state = State::DCSParam;
            }
            0x3C..=0x3F => {
                // Private marker
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::DCSParam;
            }
            0x40..=0x7E => {
                // Final byte - enter passthrough
                self.state = State::DCSPassthrough;
            }
            _ => {
                // Invalid
                self.state = State::DCSIgnore;
            }
        }
    }

    fn dcs_param(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // Intermediate bytes
                if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                    self.intermediate_bytes.push(byte);
                } else {
                    #[cfg(debug_assertions)]
                    log::warn!("intermediate_bytes buffer full; dropping byte");
                }
                self.state = State::DCSIntermediate;
            }
            0x30..=0x39 => {
                // Parameter digit
                self.param_digit(byte - b'0');
            }
            0x3A => {
                // Sub-parameter delimiter
                self.state = State::DCSIgnore;
            }
            0x3B => {
                // Parameter separator
                self.param_separator();
            }
            0x40..=0x7E => {
                // Final byte - enter passthrough
                self.state = State::DCSPassthrough;
            }
            _ => {
                // Invalid
                self.state = State::DCSIgnore;
            }
        }
    }

    fn dcs_intermediate(&mut self, byte: u8) {
        match byte {
            0x20..=0x2F => {
                // More intermediate bytes
                if self.intermediate_bytes.len() < 2
                    && self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE
                {
                    if self.intermediate_bytes.len() < MAX_INTERMEDIATE_SIZE {
                        self.intermediate_bytes.push(byte);
                    } else {
                        #[cfg(debug_assertions)]
                        log::warn!("intermediate_bytes buffer full; dropping byte");
                    }
                }
            }
            0x30..=0x3F => {
                // Invalid in intermediate
                self.state = State::DCSIgnore;
            }
            0x40..=0x7E => {
                // Final byte - enter passthrough
                self.state = State::DCSPassthrough;
            }
            _ => {
                // Invalid
                self.state = State::DCSIgnore;
            }
        }
    }

    fn dcs_passthrough(&mut self, byte: u8) {
        // PLT-006: the string ends on ESC backslash, handled before this.
        if self.dcs_string.len() < MAX_DCS_STRING_SIZE {
            self.dcs_string.push(byte);
        } else {
            // Buffer full - silently drop excess data and mark as in error state
            #[cfg(debug_assertions)]
            log::warn!("DCS string buffer full; dropping data");
            self.transition_to(State::DCSIgnore);
        }
    }

    fn dcs_ignore(&mut self, _byte: u8) {
        // PLT-006: the string ends on ESC backslash, handled before this.
    }

    fn osc_string(&mut self, byte: u8) {
        if byte == 0x07 {
            self.osc_end();
        } else {
            // Prevent unbounded buffer growth
            if self.osc_string.len() < MAX_OSC_STRING_SIZE {
                self.osc_string.push(byte);
            } else {
                // Buffer full - silently drop excess data and terminate the sequence
                #[cfg(debug_assertions)]
                log::warn!("OSC string buffer full; terminating sequence");
                self.osc_string.clear();
                self.transition_to(State::Ground);
            }
        }
    }

    fn sos_pm_apc_string(&mut self, byte: u8) {
        // PLT-006: the string ends on ESC backslash, which hands this 0x9C.
        if byte == 0x9C {
            let payload = mem::take(&mut self.osc_string);
            let action = match self.string_kind {
                0x98 => Action::SOS(payload),
                0x9E => Action::PM(payload),
                _ => Action::APC(payload),
            };
            self.actions.push(action);
            self.transition_to(State::Ground);
        } else if self.osc_string.len() < MAX_OSC_STRING_SIZE {
            self.osc_string.push(byte);
        }
    }

    // Helper methods

    fn param_digit(&mut self, digit: u8) {
        let digit = digit as u16;
        match self.current_param {
            Some(p) => {
                // Avoid overflow
                if p < 10000 {
                    self.current_param = Some(p * 10 + digit);
                }
            }
            None => {
                self.current_param = Some(digit);
            }
        }
    }

    fn param_separator(&mut self) {
        // Push current parameter and prepare for next
        let param = self.current_param.unwrap_or(0);
        // Prevent unbounded growth of params
        if self.params.len() < MAX_PARAMS_SIZE {
            self.params.push(param);
        } else {
            #[cfg(debug_assertions)]
            log::warn!("params buffer full; dropping parameter");
        }
        self.current_param = None;
    }

    fn finalize_params(&mut self) {
        // Push any pending parameter
        if let Some(p) = self.current_param.take() {
            if self.params.len() < MAX_PARAMS_SIZE {
                self.params.push(p);
            }
        } else if self.params.is_empty() {
            // Some sequences expect at least one parameter
            self.params.push(0);
        }
    }

    fn csi_dispatch(&mut self, final_byte: u8) {
        self.finalize_params();

        if let Some(action) = CSIAction::parse(&self.params, &self.intermediate_bytes, final_byte) {
            self.actions.push(Action::CSI(action));
        }
    }

    fn esc_dispatch(&mut self, final_byte: u8) {
        if let Some(action) = ESCAction::parse(&self.intermediate_bytes, final_byte) {
            self.actions.push(Action::ESC(action));
        }
    }

    fn osc_end(&mut self) {
        if let Some(action) = OSCAction::parse(&self.osc_string) {
            self.actions.push(Action::OSC(action));
        }
        // PLT-006: BEL and ST both finish and clear the title payload.
        self.osc_string.clear();
        self.transition_to(State::Ground);
    }

    fn dcs_dispatch(&mut self) {
        // For now, just store raw DCS data
        if !self.dcs_string.is_empty() {
            self.actions
                .push(Action::DCS(mem::take(&mut self.dcs_string)));
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // PLT-005: malformed UTF-8 replaces the prefix and reprocesses its bad byte.
    #[test]
    fn plt_005_invalid_utf8_recovers() {
        for bytes in [&b"\xc3X"[..], &b"\xc0X"[..], &b"\xffX"[..]] {
            assert_eq!(
                Parser::new().feed(bytes),
                vec![Action::Print('\u{FFFD}'), Action::Print('X')]
            );
        }
        // PLT-005: the byte that broke a character is handled again on its
        // own, and a lone continuation byte is itself invalid, so each broken
        // prefix and each stray byte prints its own U+FFFD.
        for (bytes, replacements) in [
            (&b"\xe0\x80X"[..], 2),
            (&b"\xed\xa0X"[..], 2),
            (&b"\xf4\xbfX"[..], 2),
        ] {
            let mut parser = Parser::new();
            let actions = bytes
                .iter()
                .flat_map(|byte| parser.feed(&[*byte]))
                .collect::<Vec<_>>();
            // PLT-005: the bad continuation is processed again as a standalone byte.
            let mut expected = vec![Action::Print('\u{FFFD}'); replacements];
            expected.push(Action::Print('X'));
            assert_eq!(actions, expected);
        }
        assert_eq!(
            Parser::new().feed(b"\xc3\xc3\xa9"),
            vec![Action::Print('\u{FFFD}'), Action::Print('é')]
        );
        // PLT-005: a bare 0x9B is an invalid UTF-8 byte, not the 8-bit CSI.
        assert_eq!(
            Parser::new().feed(b"\x9b2A"),
            vec![
                Action::Print('\u{FFFD}'),
                Action::Print('2'),
                Action::Print('A')
            ]
        );
    }

    // PLT-006: pending ESC survives a feed boundary for every string kind.
    #[test]
    fn plt_006_string_escape_and_abort() {
        for (introducer, action) in [
            (b'X', Action::SOS(vec![b'x'])),
            (b'^', Action::PM(vec![b'x'])),
            (b'_', Action::APC(vec![b'x'])),
            (b'P', Action::DCS(vec![b'x'])),
        ] {
            let mut parser = Parser::new();
            let mut input = vec![0x1b, introducer];
            if introducer == b'P' {
                input.push(b'q');
            }
            input.extend_from_slice(b"x\x1b");
            assert!(parser.feed(&input).is_empty());
            assert_eq!(parser.feed(b"\\X"), vec![action, Action::Print('X')]);
        }
        let mut parser = Parser::new();
        assert!(parser.feed(b"\x1b]2;aborted\x1b").is_empty());
        assert_eq!(
            parser.feed(b"[2AX"),
            vec![Action::CSI(CSIAction::CursorUp(2)), Action::Print('X')]
        );
        assert_eq!(
            parser.feed(b"\x1b]2;new\x07X"),
            vec![
                Action::OSC(OSCAction::SetTitle("new".into())),
                Action::Print('X')
            ]
        );
    }

    #[test]
    fn test_basic_text() {
        let mut parser = Parser::new();
        let actions = parser.feed(b"Hello World");
        assert_eq!(actions.len(), 11);
        assert_eq!(actions[0], Action::Print('H'));
    }

    #[test]
    fn test_csi_cursor_move() {
        let mut parser = Parser::new();

        // Move cursor up 5 lines
        let actions = parser.feed(b"\x1b[5A");
        assert_eq!(actions, vec![Action::CSI(CSIAction::CursorUp(5))]);

        // Move cursor to position 10,20
        let actions = parser.feed(b"\x1b[10;20H");
        assert_eq!(
            actions,
            vec![Action::CSI(CSIAction::CursorPosition { row: 10, col: 20 })]
        );
    }

    #[test]
    fn test_sgr_colors() {
        let mut parser = Parser::new();

        // Red foreground
        let actions = parser.feed(b"\x1b[31m");
        match &actions[0] {
            Action::CSI(CSIAction::SetGraphicsMode(attrs)) => {
                assert_eq!(attrs.len(), 1);
            }
            _ => panic!("Expected SGR action"),
        }

        // Reset
        let actions = parser.feed(b"\x1b[0m");
        match &actions[0] {
            Action::CSI(CSIAction::SetGraphicsMode(attrs)) => {
                assert_eq!(attrs.len(), 1);
            }
            _ => panic!("Expected SGR action"),
        }
    }

    #[test]
    fn test_osc_title() {
        let mut parser = Parser::new();

        // Set window title
        let actions = parser.feed(b"\x1b]2;My Title\x07");
        assert_eq!(
            actions,
            vec![Action::OSC(OSCAction::SetTitle("My Title".to_string()))]
        );
    }

    #[test]
    fn test_mixed_content() {
        let mut parser = Parser::new();

        let input = b"Normal \x1b[1mBold\x1b[0m text";
        let actions = parser.feed(input);

        // Should have: "Normal ", SGR bold, "Bold", SGR reset, " text"
        assert!(actions.len() >= 5);
    }
}
