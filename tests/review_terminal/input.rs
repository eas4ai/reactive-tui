//! Part of tests/review_terminal.rs: the direct backend's input parser.

use reactive_tui::platform::parser::EscapeSequenceParser;
use reactive_tui::platform::TerminalEvent;

/// PLT-002: a bracketed paste is one `Paste` event with the text as pasted,
/// whole or split across reads, and no key comes out of its payload.
#[test]
fn plt_002_a_bracketed_paste_is_one_paste_event() {
    let mut parser = EscapeSequenceParser::new();
    let events = parser.parse(b"\x1b[200~a\r\x1b[201~");
    assert!(
        matches!(events.as_slice(), [TerminalEvent::Paste(text)] if text == "a\r"),
        "PLT-002: a bracketed paste of `a CR` gave {events:?} instead of one Paste(\"a\\r\")"
    );

    let mut parser = EscapeSequenceParser::new();
    let mut events = parser.parse(b"\x1b[200~a");
    events.extend(parser.parse(b"\r"));
    events.extend(parser.parse(b"\x1b[201~"));
    assert!(
        matches!(events.as_slice(), [TerminalEvent::Paste(text)] if text == "a\r"),
        "PLT-002: the same paste split across three reads gave {events:?}"
    );
}
