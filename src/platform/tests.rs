//! Tests for the direct TTY implementation

#[cfg(test)]
mod tests {
    use crate::platform::parser::EscapeSequenceParser;
    use crate::platform::{sequences, ColorScheme, DirectTty, KeyCode, TerminalEvent};

    #[test]
    fn test_capability_detection() {
        let tty = DirectTty::init().expect("Failed to initialize TTY");

        // Basic capabilities should be detected
        let caps = tty.capabilities();

        // These should always be true for modern terminals
        assert!(caps.true_color || caps.hyperlinks || caps.bracketed_paste);

        println!("Detected capabilities:");
        println!("  True Color: {}", caps.true_color);
        println!("  Kitty Graphics: {}", caps.kitty_graphics);
        println!("  Sixel Graphics: {}", caps.sixel_graphics);
        println!("  Hyperlinks: {}", caps.hyperlinks);
        println!("  Synchronized Output: {}", caps.synchronized_output);
    }

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
            _ => panic!("Expected Up key event"),
        }

        // Test mouse event parsing
        let events = parser.parse(b"\x1b[M !"); // Mouse click
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::Mouse { .. } => {}
            _ => panic!("Expected Mouse event"),
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
    fn test_environment_detection() {
        // Test environment-based capability detection
        unsafe {
            std::env::set_var("TERM", "xterm-256color");
            std::env::set_var("COLORTERM", "truecolor");
        }

        let tty = DirectTty::init().expect("Failed to initialize TTY");
        let caps = tty.capabilities();

        // Should detect true color from environment
        assert!(caps.true_color);

        // Clean up
        unsafe {
            std::env::remove_var("TERM");
            std::env::remove_var("COLORTERM");
        }
    }

    #[test]
    fn test_parser_state_machine() {
        let mut parser = EscapeSequenceParser::new();

        // Test partial sequence parsing
        let events1 = parser.parse(b"\x1b[");
        assert_eq!(events1.len(), 0); // Incomplete sequence

        let events2 = parser.parse(b"A");
        assert_eq!(events2.len(), 1); // Complete sequence
        match &events2[0] {
            TerminalEvent::Key {
                code: KeyCode::Up, ..
            } => {}
            _ => panic!("Expected Up key event"),
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
            _ => panic!("Expected F1 key event"),
        }

        // Test paste events
        let events = parser.parse(b"\x1b[200~"); // Paste start
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::PasteStart => {}
            _ => panic!("Expected PasteStart event"),
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
            _ => panic!("Expected FocusGained event"),
        }

        // Test focus lost
        let events = parser.parse(b"\x1b[O");
        assert_eq!(events.len(), 1);
        match &events[0] {
            TerminalEvent::FocusLost => {}
            _ => panic!("Expected FocusLost event"),
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
            _ => panic!("Expected ColorScheme::Dark event"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_tty_basic() {
        use crate::platform::unix::UnixTty;
        use crate::platform::PlatformTty;

        let tty = UnixTty::init().expect("Failed to initialize Unix TTY");

        // Test basic operations
        let (width, height) = tty.size().expect("Failed to get terminal size");
        assert!(width > 0 && height > 0);

        // Test writing
        let written = tty.write(b"test").expect("Failed to write");
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
            _ => panic!("Expected modified Up key event"),
        }
    }
}
