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

/// PLT-002 (the adversary's finding 2): the 1 MiB bound holds for the text
/// the Paste event carries, not only for the bytes collected.
#[test]
fn plt_002_the_paste_bound_holds_for_the_emitted_text() {
    use reactive_tui::platform::parser::MAX_PASTE_BYTES;

    let mut parser = EscapeSequenceParser::new();
    assert!(parser.parse(b"\x1b[200~").is_empty());
    assert!(parser.parse(&vec![0xff; MAX_PASTE_BYTES]).is_empty());
    let events = parser.parse(b"\x1b[201~");
    match events.as_slice() {
        [TerminalEvent::Paste(text)] => assert!(
            text.len() <= MAX_PASTE_BYTES,
            "PLT-002: a paste of 1 MiB of invalid bytes came out as {} bytes of text",
            text.len()
        ),
        other => panic!("PLT-002: the paste of invalid bytes gave {other:?}"),
    }

    let mut parser = EscapeSequenceParser::new();
    let mut payload = vec![b'a'; MAX_PASTE_BYTES - 1];
    payload.extend_from_slice("é".as_bytes());
    assert!(parser.parse(b"\x1b[200~").is_empty());
    assert!(parser.parse(&payload).is_empty());
    let events = parser.parse(b"\x1b[201~");
    match events.as_slice() {
        [TerminalEvent::Paste(text)] => {
            assert!(
                text.len() <= MAX_PASTE_BYTES,
                "PLT-002: a paste ending in a character that does not fit came out as {} bytes",
                text.len()
            );
            assert!(
                text.chars().all(|c| c == 'a'),
                "PLT-002: the character split at the limit came out as {:?}",
                text.chars().last()
            );
        }
        other => panic!("PLT-002: the paste split at the limit gave {other:?}"),
    }
}
