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
        let mouse_data = b"\x1b[M !";
        println!("Mouse data: {:?}", mouse_data);
        let events = parser.parse(mouse_data); // Mouse click
        println!("Mouse events: {:?}", events);
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

        // Test stateless parser with buffering approach
        // Simulate what the event loop would do
        let mut buffer = Vec::new();

        // First input: incomplete sequence
        buffer.extend_from_slice(b"\x1b[");
        let events1 = parser.parse(&buffer);
        assert_eq!(events1.len(), 0); // Incomplete sequence - parser returns no events

        // Second input: complete the sequence
        buffer.extend_from_slice(b"A");
        let events2 = parser.parse(&buffer);
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

        // Skip test if no controlling terminal is available
        let tty = match UnixTty::init() {
            Ok(tty) => tty,
            Err(_) => {
                eprintln!("Skipping test_unix_tty_basic: No controlling terminal available");
                return;
            }
        };

        // Test basic operations - skip if not available
        let (width, height) = match tty.size() {
            Ok(size) => size,
            Err(_) => {
                eprintln!("Skipping terminal size test: Terminal operations not available");
                return;
            }
        };
        assert!(width > 0 && height > 0);

        // Test writing - skip if not available
        let written = match tty.write(b"test") {
            Ok(n) => n,
            Err(_) => {
                eprintln!("Skipping write test: Terminal write operations not available");
                return;
            }
        };
        assert_eq!(written, 4);
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
