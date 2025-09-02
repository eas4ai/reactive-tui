//! Terminal escape sequence parsing and handling
//!
//! This module provides parsers for various terminal escape sequences including
//! CSI (Control Sequence Introducer), ESC codes, and OSC (Operating System Command).

/// CSI (Control Sequence Introducer) sequence parsing
pub mod csi;
/// ESC escape code parsing and handling
pub mod esc;
/// OSC (Operating System Command) sequence parsing
pub mod osc;
/// Main escape sequence parser and state machine
pub mod parser;

use std::fmt;

/// Unified action enum representing all possible escape sequence outcomes
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Print a single character
    Print(char),

    /// Execute a C0 control character
    Execute(u8),

    /// Control Sequence Introducer (CSI) sequences
    CSI(csi::CSIAction),

    /// Operating System Command (OSC) sequences
    OSC(osc::OSCAction),

    /// ESC sequences (non-CSI)
    ESC(esc::ESCAction),

    /// Device Control String (DCS) sequences
    DCS(Vec<u8>),

    /// Application Program Command (APC)
    APC(Vec<u8>),

    /// Privacy Message (PM)
    PM(Vec<u8>),

    /// Start of String (SOS)
    SOS(Vec<u8>),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Print(c) => write!(f, "Print('{c}')"),
            Action::Execute(b) => write!(f, "Execute(0x{b:02x})"),
            Action::CSI(csi) => write!(f, "CSI({csi:?})"),
            Action::OSC(osc) => write!(f, "OSC({osc:?})"),
            Action::ESC(esc) => write!(f, "ESC({esc:?})"),
            Action::DCS(data) => write!(f, "DCS({} bytes)", data.len()),
            Action::APC(data) => write!(f, "APC({} bytes)", data.len()),
            Action::PM(data) => write!(f, "PM({} bytes)", data.len()),
            Action::SOS(data) => write!(f, "SOS({} bytes)", data.len()),
        }
    }
}
