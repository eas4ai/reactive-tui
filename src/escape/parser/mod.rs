use super::{csi::CSIAction, esc::ESCAction, osc::OSCAction, Action};
use std::mem;

/// Maximum sizes for buffers to prevent DoS attacks
const MAX_OSC_STRING_SIZE: usize = 8192; // 8KB limit for OSC strings
const MAX_DCS_STRING_SIZE: usize = 16384; // 16KB limit for DCS strings
const MAX_INTERMEDIATE_SIZE: usize = 8; // VT standard allows max 2 intermediates
const MAX_PARAMS_SIZE: usize = 32; // VT standard allows max 16 params

/// VT parser state machine for ANSI escape sequences
/// Based on Paul Williams' state machine design: <https://vt100.net/emu/dec_ansi_parser>
pub struct Parser {
    state: State,
    intermediate_bytes: Vec<u8>,
    params: Vec<u16>,
    osc_string: Vec<u8>,
    dcs_string: Vec<u8>,
    current_param: Option<u16>,
    actions: Vec<Action>,
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
                    // ESC - Start escape sequence
                    self.transition_to(State::Escape);
                    return;
                }
                _ => {} // Other C0 controls - ignore in most states
            }
        }

        // C1 control characters (0x80-0x9F) in 8-bit mode
        if (0x80..=0x9F).contains(&byte) {
            match byte {
                0x90 => {
                    // DCS
                    self.transition_to(State::DCSEntry);
                    return;
                }
                0x9B => {
                    // CSI
                    self.transition_to(State::CSIEntry);
                    return;
                }
                0x9D => {
                    // OSC
                    self.transition_to(State::OSCString);
                    return;
                }
                0x98 | 0x9E | 0x9F => {
                    // SOS, PM, APC
                    self.transition_to(State::SOSPMAPCString);
                    return;
                }
                _ => {} // Other C1 controls
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
            State::OSCString => {
                self.osc_string.clear();
            }
            _ => {}
        }
        self.state = new_state;
    }

    fn ground(&mut self, byte: u8) {
        if (0x20..0x7F).contains(&byte) {
            self.actions.push(Action::Print(byte as char));
        } else if byte >= 0xA0 {
            // UTF-8 or extended ASCII
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
        // Collect DCS data until ST (String Terminator)
        if byte == 0x9C || (self.dcs_string.last() == Some(&0x1B) && byte == b'\\') {
            // End of DCS
            if self.dcs_string.last() == Some(&0x1B) {
                self.dcs_string.pop(); // Remove ESC
            }
            self.dcs_dispatch();
            self.transition_to(State::Ground);
        } else {
            // Prevent unbounded buffer growth
            if self.dcs_string.len() < MAX_DCS_STRING_SIZE {
                self.dcs_string.push(byte);
            } else {
                // Buffer full - silently drop excess data and mark as in error state
                #[cfg(debug_assertions)]
                log::warn!("DCS string buffer full; dropping data");
                self.transition_to(State::DCSIgnore);
            }
        }
    }

    fn dcs_ignore(&mut self, byte: u8) {
        // Ignore until ST
        if byte == 0x9C || (byte == b'\\' && self.dcs_string.last() == Some(&0x1B)) {
            self.transition_to(State::Ground);
        } else if byte != 0x1B {
            self.dcs_string.clear();
        } else {
            // Even in ignore state, prevent unbounded growth
            if self.dcs_string.len() < MAX_DCS_STRING_SIZE {
                self.dcs_string.push(byte);
            }
        }
    }

    fn osc_string(&mut self, byte: u8) {
        // Collect OSC data until ST or BEL
        if byte == 0x07 || byte == 0x9C || (self.osc_string.last() == Some(&0x1B) && byte == b'\\')
        {
            // End of OSC
            if self.osc_string.last() == Some(&0x1B) {
                self.osc_string.pop(); // Remove ESC
            }
            self.osc_end();
            self.transition_to(State::Ground);
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
        // For now, just collect and ignore until ST
        if byte == 0x9C || (byte == b'\\' && self.osc_string.last() == Some(&0x1B)) {
            self.transition_to(State::Ground);
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
    }

    fn dcs_dispatch(&mut self) {
        // For now, just store raw DCS data
        if !self.dcs_string.is_empty() {
            self.actions.push(Action::DCS(self.dcs_string.clone()));
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
