//! ANSI escape sequence constants and utilities

/// ANSI control characters
pub mod control {
    /// Null character
    pub const NUL: u8 = 0x00;
    /// Start of Heading
    pub const SOH: u8 = 0x01;
    /// Start of Text
    pub const STX: u8 = 0x02;
    /// End of Text
    pub const ETX: u8 = 0x03;
    /// End of Transmission
    pub const EOT: u8 = 0x04;
    /// Enquiry
    pub const ENQ: u8 = 0x05;
    /// Acknowledge
    pub const ACK: u8 = 0x06;
    /// Bell (audible alert)
    pub const BEL: u8 = 0x07;
    /// Backspace
    pub const BS: u8 = 0x08;
    /// Horizontal Tab
    pub const HT: u8 = 0x09;
    /// Line Feed (newline)
    pub const LF: u8 = 0x0A;
    /// Vertical Tab
    pub const VT: u8 = 0x0B;
    /// Form Feed
    pub const FF: u8 = 0x0C;
    /// Carriage Return
    pub const CR: u8 = 0x0D;
    /// Shift Out
    pub const SO: u8 = 0x0E;
    /// Shift In
    pub const SI: u8 = 0x0F;
    /// Data Link Escape
    pub const DLE: u8 = 0x10;
    /// Device Control 1 (XON)
    pub const DC1: u8 = 0x11;
    /// Device Control 2
    pub const DC2: u8 = 0x12;
    /// Device Control 3 (XOFF)
    pub const DC3: u8 = 0x13;
    /// Device Control 4
    pub const DC4: u8 = 0x14;
    /// Negative Acknowledge
    pub const NAK: u8 = 0x15;
    /// Synchronous Idle
    pub const SYN: u8 = 0x16;
    /// End of Transmission Block
    pub const ETB: u8 = 0x17;
    /// Cancel
    pub const CAN: u8 = 0x18;
    /// End of Medium
    pub const EM: u8 = 0x19;
    /// Substitute
    pub const SUB: u8 = 0x1A;
    /// Escape character
    pub const ESC: u8 = 0x1B;
    /// File Separator
    pub const FS: u8 = 0x1C;
    /// Group Separator
    pub const GS: u8 = 0x1D;
    /// Record Separator
    pub const RS: u8 = 0x1E;
    /// Unit Separator
    pub const US: u8 = 0x1F;
    /// Delete character
    pub const DEL: u8 = 0x7F;
}

/// ANSI escape sequences
pub mod sequences {
    // Cursor movement
    /// Move cursor up one line
    pub const CURSOR_UP: &str = "\x1b[A";
    /// Move cursor down one line
    pub const CURSOR_DOWN: &str = "\x1b[B";
    /// Move cursor right one column
    pub const CURSOR_RIGHT: &str = "\x1b[C";
    /// Move cursor left one column
    pub const CURSOR_LEFT: &str = "\x1b[D";
    /// Move cursor to home position (1,1)
    pub const CURSOR_HOME: &str = "\x1b[H";
    /// Template for cursor position escape sequence
    pub const CURSOR_POSITION: &str = "\x1b[{};{}H";

    // Cursor visibility
    /// Show cursor
    pub const CURSOR_SHOW: &str = "\x1b[?25h";
    /// Hide cursor
    pub const CURSOR_HIDE: &str = "\x1b[?25l";

    // Cursor save/restore
    /// Save current cursor position
    pub const CURSOR_SAVE: &str = "\x1b[s";
    /// Restore saved cursor position
    pub const CURSOR_RESTORE: &str = "\x1b[u";

    // Screen clearing
    /// Clear entire screen
    pub const CLEAR_SCREEN: &str = "\x1b[2J";
    /// Clear from cursor to end of screen
    pub const CLEAR_SCREEN_FROM_CURSOR: &str = "\x1b[0J";
    /// Clear from beginning of screen to cursor
    pub const CLEAR_SCREEN_TO_CURSOR: &str = "\x1b[1J";
    /// Clear entire line
    pub const CLEAR_LINE: &str = "\x1b[2K";
    /// Clear from cursor to end of line
    pub const CLEAR_LINE_FROM_CURSOR: &str = "\x1b[0K";
    /// Clear from beginning of line to cursor
    pub const CLEAR_LINE_TO_CURSOR: &str = "\x1b[1K";

    // Scrolling
    /// Scroll screen up one line
    pub const SCROLL_UP: &str = "\x1b[S";
    /// Scroll screen down one line
    pub const SCROLL_DOWN: &str = "\x1b[T";

    // Screen modes
    /// Enter alternate screen buffer
    pub const ALTERNATE_SCREEN_ENTER: &str = "\x1b[?1049h";
    /// Exit alternate screen buffer
    pub const ALTERNATE_SCREEN_EXIT: &str = "\x1b[?1049l";

    // Text attributes
    /// Reset all text attributes
    pub const RESET: &str = "\x1b[0m";
    /// Enable bold text
    pub const BOLD: &str = "\x1b[1m";
    /// Enable dim text
    pub const DIM: &str = "\x1b[2m";
    /// Enable italic text
    pub const ITALIC: &str = "\x1b[3m";
    /// Enable underlined text
    pub const UNDERLINE: &str = "\x1b[4m";
    /// Enable blinking text
    pub const BLINK: &str = "\x1b[5m";
    /// Enable reverse video
    pub const REVERSE: &str = "\x1b[7m";
    /// Enable strikethrough text
    pub const STRIKETHROUGH: &str = "\x1b[9m";

    // Reset attributes
    /// Reset bold/bright text
    pub const RESET_BOLD: &str = "\x1b[22m";
    /// Reset dim text
    pub const RESET_DIM: &str = "\x1b[22m";
    /// Reset italic text
    pub const RESET_ITALIC: &str = "\x1b[23m";
    /// Reset underline text
    pub const RESET_UNDERLINE: &str = "\x1b[24m";
    /// Reset blinking text
    pub const RESET_BLINK: &str = "\x1b[25m";
    /// Reset reverse video
    pub const RESET_REVERSE: &str = "\x1b[27m";
    /// Reset strikethrough text
    pub const RESET_STRIKETHROUGH: &str = "\x1b[29m";

    // Terminal reset
    /// Reset terminal to initial state
    pub const TERMINAL_RESET: &str = "\x1bc";

    // Bell
    /// Terminal bell/beep sound
    pub const BELL: &str = "\x07";
}

/// ANSI color codes
pub mod colors {
    // Standard colors (30-37 for foreground, 40-47 for background)
    /// Black color index
    pub const BLACK: u8 = 0;
    /// Red color index
    pub const RED: u8 = 1;
    /// Green color index
    pub const GREEN: u8 = 2;
    /// Yellow color index
    pub const YELLOW: u8 = 3;
    /// Blue color index
    pub const BLUE: u8 = 4;
    /// Magenta color index
    pub const MAGENTA: u8 = 5;
    /// Cyan color index
    pub const CYAN: u8 = 6;
    /// White color index
    pub const WHITE: u8 = 7;

    // Bright colors (90-97 for foreground, 100-107 for background)
    /// Bright black color index
    pub const BRIGHT_BLACK: u8 = 8;
    /// Bright red color index
    pub const BRIGHT_RED: u8 = 9;
    /// Bright green color index
    pub const BRIGHT_GREEN: u8 = 10;
    /// Bright yellow color index
    pub const BRIGHT_YELLOW: u8 = 11;
    /// Bright blue color index
    pub const BRIGHT_BLUE: u8 = 12;
    /// Bright magenta color index
    pub const BRIGHT_MAGENTA: u8 = 13;
    /// Bright cyan color index
    pub const BRIGHT_CYAN: u8 = 14;
    /// Bright white color index
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
        let title = crate::terminal::sanitize_host_text(title);
        format!("\x1b]0;{title}\x07")
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
