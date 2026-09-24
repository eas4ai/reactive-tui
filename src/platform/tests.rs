//! Tests for the direct TTY implementation

#[cfg(test)]
mod tty_tests {
    use crate::platform::parser::EscapeSequenceParser;
    use crate::platform::{sequences, ColorScheme, KeyCode, TerminalEvent};

    #[test]
    fn test_escape_sequence_parser() {
        let mut parser = EscapeSequenceParser::new();

        // Test basic key parsing
        let events = parser.parse(b"\x1b[A"); // Up arrow
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::Key {
                code: KeyCode::Up, ..
            } => {}
            other => {
                panic!("Expected Up key event, got: {:?}", other);
            }
        }

        // Test mouse event parsing
        let events = parser.parse(b"\x1b[M !");
        assert!(events.is_empty(), "incomplete X10 input must stay buffered");
        let events = parser.parse(b"!");
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::Mouse { .. } => {}
            other => {
                panic!("Expected Mouse event, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_control_sequences() {
        // Test that control sequences are properly formatted
        assert_eq!(sequences::PRIMARY_DEVICE_ATTRS, b"\x1b[c");
        assert_eq!(sequences::KITTY_GRAPHICS_QUERY, b"\x1b_Gi=1,a=q\x1b\\");
        assert_eq!(sequences::ENABLE_MOUSE, b"\x1b[?1002;1003;1004;1006h");

        // Test hyperlink formatting
        let link = sequences::hyperlink_start("https://example.com");
        assert_eq!(link, "\x1b]8;;https://example.com\x1b\\");
    }

    #[test]
    fn test_kitty_image_sequence() {
        let data = b"test_image_data";
        let sequence = sequences::kitty_image_transmit(1, 100, 50, data);

        // Should contain the proper Kitty graphics escape sequence
        assert!(sequence.starts_with(b"\x1b_G"));
        assert!(sequence.ends_with(b"\x1b\\"));

        // Should contain the image metadata
        let seq_str = String::from_utf8_lossy(&sequence);
        assert!(seq_str.contains("i=1"));
        assert!(seq_str.contains("s=100"));
        assert!(seq_str.contains("v=50"));
    }

    #[test]
    fn test_parser_state_machine() {
        let mut parser = EscapeSequenceParser::new();

        // First input: incomplete sequence
        let events1 = parser.parse(b"\x1b[");
        assert_eq!(events1.len(), 0); // Incomplete sequence - parser returns no events

        // Second input: complete the sequence
        let events2 = parser.parse(b"A");
        assert_eq!(events2.len(), 1); // Complete sequence
        match &events2[0] {
            TerminalEvent::Key {
                code: KeyCode::Up, ..
            } => {}
            other => {
                panic!("Expected Up key event, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_special_keys() {
        let mut parser = EscapeSequenceParser::new();

        // Test function keys
        let events = parser.parse(b"\x1b[11~"); // F1
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::Key {
                code: KeyCode::F(1),
                ..
            } => {}
            other => {
                panic!("Expected F1 key event, got: {:?}", other);
            }
        }

        // Test paste events
        let events = parser.parse(b"\x1b[200~"); // Paste start
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::PasteStart => {}
            other => {
                panic!("Expected PasteStart event, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_focus_events() {
        let mut parser = EscapeSequenceParser::new();

        // Test focus gained
        let events = parser.parse(b"\x1b[I");
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::FocusGained => {}
            other => {
                panic!("Expected FocusGained event, got: {:?}", other);
            }
        }

        // Test focus lost
        let events = parser.parse(b"\x1b[O");
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::FocusLost => {}
            other => {
                panic!("Expected FocusLost event, got: {:?}", other);
            }
        }
    }

    #[test]
    fn test_osc_parsing() {
        let mut parser = EscapeSequenceParser::new();

        // Test color scheme response
        let events = parser.parse(b"\x1b]996;dark\x07");
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::ColorScheme(ColorScheme::Dark) => {}
            other => {
                panic!("Expected ColorScheme::Dark event, got: {:?}", other);
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_tty_basic() {
        use crate::platform::unix::UnixTty;
        use crate::platform::PlatformTty;
        use crate::terminal::test_terminal::{on_terminal, COLUMNS, ROWS};

        on_terminal("platform::tests::tty_tests::test_unix_tty_basic", || {
            let tty = UnixTty::init().unwrap();
            assert_eq!(tty.size().unwrap(), (COLUMNS, ROWS));
            assert_eq!(tty.write(b"test").unwrap(), 4);
        });
    }

    #[test]
    fn test_modifiers_parsing() {
        let mut parser = EscapeSequenceParser::new();

        // Test Ctrl+A (with modifiers)
        let events = parser.parse(b"\x1b[1;5A"); // Ctrl+Up
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::Key {
                code: KeyCode::Up,
                modifiers,
                ..
            } => {
                assert!(modifiers.ctrl);
                assert!(!modifiers.shift);
                assert!(!modifiers.alt);
            }
            other => {
                panic!("Expected modified Up key event, got: {:?}", other);
            }
        }
    }
}

/// CHT-028: startup detection asks the terminal for mode 2027, so the
/// charts receive a glyph capability report from a real answer.
#[test]
fn startup_queries_ask_the_terminal_for_unicode_mode() {
    use super::sequences::{STARTUP_QUERIES, UNICODE_QUERY};
    assert!(
        STARTUP_QUERIES.contains(&UNICODE_QUERY),
        "the startup queries must include the mode 2027 query"
    );
    assert_eq!(UNICODE_QUERY, b"\x1b[?2027$p");
}
