//! ANSI escape sequence constants and utilities

/// ANSI control characters
pub mod control {
    pub const NUL: u8 = 0x00; // Null
    pub const SOH: u8 = 0x01; // Start of Heading
    pub const STX: u8 = 0x02; // Start of Text
    pub const ETX: u8 = 0x03; // End of Text
    pub const EOT: u8 = 0x04; // End of Transmission
    pub const ENQ: u8 = 0x05; // Enquiry
    pub const ACK: u8 = 0x06; // Acknowledge
    pub const BEL: u8 = 0x07; // Bell
    pub const BS: u8 = 0x08; // Backspace
    pub const HT: u8 = 0x09; // Horizontal Tab
    pub const LF: u8 = 0x0A; // Line Feed
    pub const VT: u8 = 0x0B; // Vertical Tab
    pub const FF: u8 = 0x0C; // Form Feed
    pub const CR: u8 = 0x0D; // Carriage Return
    pub const SO: u8 = 0x0E; // Shift Out
    pub const SI: u8 = 0x0F; // Shift In
    pub const DLE: u8 = 0x10; // Data Link Escape
    pub const DC1: u8 = 0x11; // Device Control 1 (XON)
    pub const DC2: u8 = 0x12; // Device Control 2
    pub const DC3: u8 = 0x13; // Device Control 3 (XOFF)
    pub const DC4: u8 = 0x14; // Device Control 4
    pub const NAK: u8 = 0x15; // Negative Acknowledge
    pub const SYN: u8 = 0x16; // Synchronous Idle
    pub const ETB: u8 = 0x17; // End of Transmission Block
    pub const CAN: u8 = 0x18; // Cancel
    pub const EM: u8 = 0x19; // End of Medium
    pub const SUB: u8 = 0x1A; // Substitute
    pub const ESC: u8 = 0x1B; // Escape
    pub const FS: u8 = 0x1C; // File Separator
    pub const GS: u8 = 0x1D; // Group Separator
    pub const RS: u8 = 0x1E; // Record Separator
    pub const US: u8 = 0x1F; // Unit Separator
    pub const DEL: u8 = 0x7F; // Delete
}

/// ANSI escape sequences
pub mod sequences {
    // Cursor movement
    pub const CURSOR_UP: &str = "\x1b[A";
    pub const CURSOR_DOWN: &str = "\x1b[B";
    pub const CURSOR_RIGHT: &str = "\x1b[C";
    pub const CURSOR_LEFT: &str = "\x1b[D";
    pub const CURSOR_HOME: &str = "\x1b[H";
    pub const CURSOR_POSITION: &str = "\x1b[{};{}H";

    // Cursor visibility
    pub const CURSOR_SHOW: &str = "\x1b[?25h";
    pub const CURSOR_HIDE: &str = "\x1b[?25l";

    // Cursor save/restore
    pub const CURSOR_SAVE: &str = "\x1b[s";
    pub const CURSOR_RESTORE: &str = "\x1b[u";

    // Screen clearing
    pub const CLEAR_SCREEN: &str = "\x1b[2J";
    pub const CLEAR_SCREEN_FROM_CURSOR: &str = "\x1b[0J";
    pub const CLEAR_SCREEN_TO_CURSOR: &str = "\x1b[1J";
    pub const CLEAR_LINE: &str = "\x1b[2K";
    pub const CLEAR_LINE_FROM_CURSOR: &str = "\x1b[0K";
    pub const CLEAR_LINE_TO_CURSOR: &str = "\x1b[1K";

    // Scrolling
    pub const SCROLL_UP: &str = "\x1b[S";
    pub const SCROLL_DOWN: &str = "\x1b[T";

    // Screen modes
    pub const ALTERNATE_SCREEN_ENTER: &str = "\x1b[?1049h";
    pub const ALTERNATE_SCREEN_EXIT: &str = "\x1b[?1049l";

    // Text attributes
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
    pub const ITALIC: &str = "\x1b[3m";
    pub const UNDERLINE: &str = "\x1b[4m";
    pub const BLINK: &str = "\x1b[5m";
    pub const REVERSE: &str = "\x1b[7m";
    pub const STRIKETHROUGH: &str = "\x1b[9m";

    // Reset attributes
    pub const RESET_BOLD: &str = "\x1b[22m";
    pub const RESET_DIM: &str = "\x1b[22m";
    pub const RESET_ITALIC: &str = "\x1b[23m";
    pub const RESET_UNDERLINE: &str = "\x1b[24m";
    pub const RESET_BLINK: &str = "\x1b[25m";
    pub const RESET_REVERSE: &str = "\x1b[27m";
    pub const RESET_STRIKETHROUGH: &str = "\x1b[29m";

    // Terminal reset
    pub const TERMINAL_RESET: &str = "\x1bc";

    // Bell
    pub const BELL: &str = "\x07";
}

/// ANSI color codes
pub mod colors {
    // Standard colors (30-37 for foreground, 40-47 for background)
    pub const BLACK: u8 = 0;
    pub const RED: u8 = 1;
    pub const GREEN: u8 = 2;
    pub const YELLOW: u8 = 3;
    pub const BLUE: u8 = 4;
    pub const MAGENTA: u8 = 5;
    pub const CYAN: u8 = 6;
    pub const WHITE: u8 = 7;

    // Bright colors (90-97 for foreground, 100-107 for background)
    pub const BRIGHT_BLACK: u8 = 8;
    pub const BRIGHT_RED: u8 = 9;
    pub const BRIGHT_GREEN: u8 = 10;
    pub const BRIGHT_YELLOW: u8 = 11;
    pub const BRIGHT_BLUE: u8 = 12;
    pub const BRIGHT_MAGENTA: u8 = 13;
    pub const BRIGHT_CYAN: u8 = 14;
    pub const BRIGHT_WHITE: u8 = 15;
}

/// Utility functions for generating ANSI sequences
pub mod utils {

    /// Generate cursor position sequence
    pub fn cursor_position(row: u16, col: u16) -> String {
        format!("\x1b[{};{}H", row + 1, col + 1)
    }

    /// Generate cursor move up sequence
    pub fn cursor_up(n: u16) -> String {
        if n == 1 {
            "\x1b[A".to_string()
        } else {
            format!("\x1b[{}A", n)
        }
    }

    /// Generate cursor move down sequence
    pub fn cursor_down(n: u16) -> String {
        if n == 1 {
            "\x1b[B".to_string()
        } else {
            format!("\x1b[{}B", n)
        }
    }

    /// Generate cursor move right sequence
    pub fn cursor_right(n: u16) -> String {
        if n == 1 {
            "\x1b[C".to_string()
        } else {
            format!("\x1b[{}C", n)
        }
    }

    /// Generate cursor move left sequence
    pub fn cursor_left(n: u16) -> String {
        if n == 1 {
            "\x1b[D".to_string()
        } else {
            format!("\x1b[{}D", n)
        }
    }

    /// Generate foreground color sequence
    pub fn fg_color(color: u8) -> String {
        if color < 8 {
            format!("\x1b[{}m", 30 + color)
        } else if color < 16 {
            format!("\x1b[{}m", 82 + color)
        } else {
            format!("\x1b[38;5;{}m", color)
        }
    }

    /// Generate background color sequence
    pub fn bg_color(color: u8) -> String {
        if color < 8 {
            format!("\x1b[{}m", 40 + color)
        } else if color < 16 {
            format!("\x1b[{}m", 92 + color)
        } else {
            format!("\x1b[48;5;{}m", color)
        }
    }

    /// Generate RGB foreground color sequence
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }

    /// Generate RGB background color sequence
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }

    /// Generate scrolling region sequence
    pub fn set_scrolling_region(top: u16, bottom: u16) -> String {
        format!("\x1b[{};{}r", top + 1, bottom + 1)
    }

    /// Generate insert lines sequence
    pub fn insert_lines(n: u16) -> String {
        if n == 1 {
            "\x1b[L".to_string()
        } else {
            format!("\x1b[{}L", n)
        }
    }

    /// Generate delete lines sequence
    pub fn delete_lines(n: u16) -> String {
        if n == 1 {
            "\x1b[M".to_string()
        } else {
            format!("\x1b[{}M", n)
        }
    }

    /// Generate OSC sequence for setting title
    pub fn set_title(title: &str) -> String {
        format!("\x1b]0;{}\x07", title)
    }

    /// Generate OSC sequence for setting working directory
    pub fn set_working_directory(path: &str) -> String {
        format!("\x1b]7;file://{}\x07", path)
    }

    /// Generate hyperlink sequence
    pub fn hyperlink(url: &str, text: &str, id: Option<&str>) -> String {
        match id {
            Some(id) => format!("\x1b]8;id={};{}\x07{}\x1b]8;;\x07", id, url, text),
            None => format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_position() {
        assert_eq!(utils::cursor_position(0, 0), "\x1b[1;1H");
        assert_eq!(utils::cursor_position(10, 20), "\x1b[11;21H");
    }

    #[test]
    fn test_color_sequences() {
        assert_eq!(utils::fg_color(colors::RED), "\x1b[31m");
        assert_eq!(utils::bg_color(colors::BLUE), "\x1b[44m");
        assert_eq!(utils::fg_rgb(255, 128, 64), "\x1b[38;2;255;128;64m");
    }

    #[test]
    fn test_cursor_movement() {
        assert_eq!(utils::cursor_up(1), "\x1b[A");
        assert_eq!(utils::cursor_up(5), "\x1b[5A");
        assert_eq!(utils::cursor_down(1), "\x1b[B");
        assert_eq!(utils::cursor_right(3), "\x1b[3C");
    }

    #[test]
    fn test_title_sequence() {
        assert_eq!(utils::set_title("Test"), "\x1b]0;Test\x07");
    }

    #[test]
    fn test_hyperlink() {
        let result = utils::hyperlink("https://example.com", "Example", Some("link1"));
        assert!(result.contains("https://example.com"));
        assert!(result.contains("Example"));
        assert!(result.contains("id=link1"));
    }
}
