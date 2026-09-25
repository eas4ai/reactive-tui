//! Full terminal emulator implementation for reactive-tui
//!
//! Complete ANSI support, PTY management, and virtual screen buffer.

pub mod ansi;
pub mod cell;
pub mod cursor;
pub(crate) mod keyboard;
#[cfg(unix)]
pub(crate) mod owned_pty;
pub mod parser;
pub mod pty;
pub mod screen;
pub mod terminal_impl;
#[cfg(test)]
pub(crate) mod test_terminal;

pub use ansi::colors;
pub use ansi::sequences;
pub use ansi::utils;
pub use cell::TerminalCell;
pub use cursor::TerminalCursor;
pub use parser::AnsiParser;
pub use pty::PseudoTerminal;
pub use screen::VirtualScreen;
pub use terminal_impl::Terminal;

pub(crate) fn sanitize_host_text(text: &str) -> std::borrow::Cow<'_, str> {
    if text.chars().any(char::is_control) {
        std::borrow::Cow::Owned(
            text.chars()
                .map(|character| {
                    if character.is_control() {
                        '\u{fffd}'
                    } else {
                        character
                    }
                })
                .collect(),
        )
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

/// Terminal configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalConfig {
    /// Terminal size (width, height) in cells
    pub size: (u16, u16),
    /// Number of lines to keep in scrollback buffer
    pub scrollback_size: usize,
    /// Shell command to run (defaults to system shell)
    pub shell: Option<String>,
    /// Environment variables to set
    pub env: Vec<(String, String)>,
    /// Working directory for the terminal process
    pub working_directory: Option<String>,
    /// Terminal window title
    pub title: String,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            size: (80, 24),
            scrollback_size: 1000,
            shell: None,
            env: Vec::new(),
            working_directory: None,
            title: "Terminal".to_string(),
        }
    }
}

/// Terminal events
#[derive(Debug, Clone)]
pub enum TerminalEvent {
    /// Output from the terminal process
    Output(Vec<u8>),
    /// Terminal title has changed
    TitleChanged(String),
    /// Terminal has been resized (width, height)
    Resized(u16, u16),
    /// Terminal process has exited with code
    ProcessExited(i32),
    /// Terminal bell was triggered
    Bell,
    /// Working directory has changed
    WorkingDirectoryChanged(String),
}

/// Terminal modes
#[derive(Debug, Clone, Default)]
pub struct TerminalModes {
    /// Auto-wrap at end of line
    pub auto_wrap: bool,
    /// Whether cursor is visible
    pub cursor_visible: bool,
    /// Application cursor keys mode
    pub application_cursor_keys: bool,
    /// Application keypad mode
    pub application_keypad: bool,
    /// Bracketed paste mode
    pub bracketed_paste: bool,
    /// Mouse event reporting enabled
    pub mouse_reporting: bool,
    /// Using alternate screen buffer
    pub alternate_screen: bool,
    /// Origin mode for cursor positioning
    pub origin_mode: bool,
    /// Insert mode (vs replace mode)
    pub insert_mode: bool,
    /// Echo typed characters locally
    pub local_echo: bool,
}

/// Scrolling region
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollingRegion {
    /// Top row of the scrolling region (0-based)
    pub top: u16,
    /// Bottom row of the scrolling region (0-based)
    pub bottom: u16,
    /// Left column of the scrolling region (0-based)
    pub left: u16,
    /// Right column of the scrolling region (0-based)
    pub right: u16,
}

impl ScrollingRegion {
    /// Create a new scrolling region with specified bounds
    pub fn new(top: u16, bottom: u16, left: u16, right: u16) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Create a scrolling region that covers the full screen
    pub fn full_screen(width: u16, height: u16) -> Self {
        Self {
            top: 0,
            bottom: height.saturating_sub(1),
            left: 0,
            right: width.saturating_sub(1),
        }
    }

    /// Check if a position is within this scrolling region
    ///
    /// # Arguments
    /// * `col` - Column position to check
    /// * `row` - Row position to check
    ///
    /// # Returns
    /// true if the position is within the region
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.left && col <= self.right && row >= self.top && row <= self.bottom
    }

    /// Get the width of the scrolling region
    ///
    /// # Returns
    /// Width in columns
    pub fn width(&self) -> u16 {
        self.right.saturating_sub(self.left).saturating_add(1)
    }

    /// Get the height of the scrolling region
    ///
    /// # Returns
    /// Height in rows
    pub fn height(&self) -> u16 {
        self.bottom.saturating_sub(self.top).saturating_add(1)
    }
}

/// Terminal colors
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TerminalColor {
    /// Default terminal color
    #[default]
    Default,
    /// Indexed color (0-255)
    Indexed(u8),
    /// RGB color (red, green, blue)
    Rgb(u8, u8, u8),
}

/// Terminal text style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TerminalStyle {
    /// Foreground text color
    pub foreground: TerminalColor,
    /// Background color
    pub background: TerminalColor,
    /// Whether text is bold
    pub bold: bool,
    /// Whether text is dimmed
    pub dim: bool,
    /// Whether text is italic
    pub italic: bool,
    /// Whether text is underlined
    pub underline: bool,
    /// Whether text has strikethrough
    pub strikethrough: bool,
    /// Whether colors are reversed
    pub reverse: bool,
    /// Whether text is blinking
    pub blink: bool,
    /// Whether text is invisible
    pub invisible: bool,
}

impl TerminalStyle {
    /// Create a new default terminal style
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset style to default values
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Set foreground color
    pub fn with_foreground(mut self, color: TerminalColor) -> Self {
        self.foreground = color;
        self
    }

    /// Set background color
    pub fn with_background(mut self, color: TerminalColor) -> Self {
        self.background = color;
        self
    }

    /// Set bold attribute
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// Set italic attribute
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Set underline attribute
    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

/// Terminal errors
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    /// PTY (pseudo-terminal) related error
    #[error("PTY error: {0}")]
    Pty(String),

    /// Terminal escape sequence parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Process execution error
    #[error("Process error: {0}")]
    Process(String),

    /// I/O operation error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal not properly initialized
    #[error("Terminal not initialized")]
    NotInitialized,

    /// Invalid terminal dimensions
    #[error("Invalid terminal size: {width}x{height}")]
    InvalidSize {
        /// Terminal width that was invalid
        width: u16,
        /// Terminal height that was invalid
        height: u16,
    },
}

/// Result type for terminal operations
pub type TerminalResult<T> = std::result::Result<T, TerminalError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrolling_region() {
        let region = ScrollingRegion::new(5, 15, 10, 70);
        assert_eq!(region.width(), 61);
        assert_eq!(region.height(), 11);
        assert!(region.contains(10, 5));
        assert!(region.contains(70, 15));
        assert!(!region.contains(9, 5));
        assert!(!region.contains(10, 4));
    }

    #[test]
    fn test_terminal_style() {
        let style = TerminalStyle::new()
            .with_foreground(TerminalColor::Rgb(255, 0, 0))
            .with_bold(true)
            .with_italic(true);

        assert_eq!(style.foreground, TerminalColor::Rgb(255, 0, 0));
        assert!(style.bold);
        assert!(style.italic);
        assert!(!style.underline);
    }

    #[test]
    fn test_terminal_config_default() {
        let config = TerminalConfig::default();
        assert_eq!(config.size, (80, 24));
        assert_eq!(config.scrollback_size, 1000);
        assert_eq!(config.title, "Terminal");
    }
}
