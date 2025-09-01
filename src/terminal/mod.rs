//! Full terminal emulator implementation for reactive-tui
//!
//! Complete ANSI support, PTY management, and virtual screen buffer.

pub mod ansi;
pub mod cell;
pub mod cursor;
pub mod parser;
pub mod pty;
pub mod screen;
pub mod terminal_impl;

pub use ansi::colors;
pub use ansi::sequences;
pub use ansi::utils;
pub use cell::TerminalCell;
pub use cursor::TerminalCursor;
pub use parser::AnsiParser;
pub use pty::PseudoTerminal;
pub use screen::VirtualScreen;
pub use terminal_impl::Terminal;

/// Terminal configuration
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalConfig {
    pub size: (u16, u16),
    pub scrollback_size: usize,
    pub shell: Option<String>,
    pub env: Vec<(String, String)>,
    pub working_directory: Option<String>,
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
    Output(Vec<u8>),
    TitleChanged(String),
    Resized(u16, u16),
    ProcessExited(i32),
    Bell,
    WorkingDirectoryChanged(String),
}

/// Terminal modes
#[derive(Debug, Clone, Default)]
pub struct TerminalModes {
    pub auto_wrap: bool,
    pub cursor_visible: bool,
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    pub bracketed_paste: bool,
    pub mouse_reporting: bool,
    pub alternate_screen: bool,
    pub origin_mode: bool,
    pub insert_mode: bool,
    pub local_echo: bool,
}

/// Scrolling region
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollingRegion {
    pub top: u16,
    pub bottom: u16,
    pub left: u16,
    pub right: u16,
}

impl ScrollingRegion {
    pub fn new(top: u16, bottom: u16, left: u16, right: u16) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn full_screen(width: u16, height: u16) -> Self {
        Self {
            top: 0,
            bottom: height.saturating_sub(1),
            left: 0,
            right: width.saturating_sub(1),
        }
    }

    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.left && col <= self.right && row >= self.top && row <= self.bottom
    }

    pub fn width(&self) -> u16 {
        self.right.saturating_sub(self.left).saturating_add(1)
    }

    pub fn height(&self) -> u16 {
        self.bottom.saturating_sub(self.top).saturating_add(1)
    }
}

/// Terminal colors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TerminalColor {
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Default for TerminalColor {
    fn default() -> Self {
        Self::Default
    }
}

/// Terminal text style
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TerminalStyle {
    pub foreground: TerminalColor,
    pub background: TerminalColor,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub blink: bool,
    pub invisible: bool,
}

impl TerminalStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn with_foreground(mut self, color: TerminalColor) -> Self {
        self.foreground = color;
        self
    }

    pub fn with_background(mut self, color: TerminalColor) -> Self {
        self.background = color;
        self
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn with_underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }
}

/// Terminal errors
#[derive(Debug, thiserror::Error)]
pub enum TerminalError {
    #[error("PTY error: {0}")]
    Pty(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Terminal not initialized")]
    NotInitialized,

    #[error("Invalid terminal size: {width}x{height}")]
    InvalidSize { width: u16, height: u16 },
}

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
