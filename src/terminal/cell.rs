//! Terminal cell implementation

use super::{TerminalColor, TerminalStyle};
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A single cell in the terminal grid
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCell {
    /// Character or grapheme cluster in this cell
    pub character: String,
    /// Display width of the character (1 or 2 for wide chars)
    pub width: u8,
    /// Text style and colors for this cell
    pub style: TerminalStyle,
    /// Whether this cell needs to be redrawn
    pub dirty: bool,
    /// Whether this cell ends a soft-wrapped row (before any wide-glyph padding).
    pub wrapped: bool,
    /// Hyperlink URL if this cell is part of a link
    pub hyperlink: Option<String>,
    /// Hyperlink identifier for grouping
    pub hyperlink_id: Option<String>,
}

impl TerminalCell {
    /// Create a new empty terminal cell
    pub fn new() -> Self {
        Self {
            character: " ".to_string(),
            width: 1,
            style: TerminalStyle::default(),
            dirty: true,
            wrapped: false,
            hyperlink: None,
            hyperlink_id: None,
        }
    }

    /// Create a new terminal cell with a character
    pub fn with_char(ch: char) -> Self {
        let mut cell = Self::new();
        cell.set_char(ch);
        cell
    }

    /// Set the character for this cell
    pub fn set_char(&mut self, ch: char) {
        self.character = ch.to_string();
        self.width = char_width(ch);
        self.dirty = true;
    }

    /// Set the string content for this cell
    pub fn set_string(&mut self, s: String) {
        self.width = string_width(&s);
        self.character = s;
        self.dirty = true;
    }

    /// Set the style for this cell
    pub fn set_style(&mut self, style: TerminalStyle) {
        if self.style != style {
            self.style = style;
            self.dirty = true;
        }
    }

    /// Clear the cell with a background color
    pub fn clear(&mut self, bg_color: TerminalColor) {
        self.character = " ".to_string();
        self.width = 1;
        self.style = TerminalStyle {
            background: bg_color,
            ..TerminalStyle::default()
        };
        self.wrapped = false;
        self.hyperlink = None;
        self.hyperlink_id = None;
        self.dirty = true;
    }

    /// Erase the cell content
    pub fn erase(&mut self) {
        let bg = self.style.background;
        self.clear(bg);
    }

    /// Check if the cell is empty
    pub fn is_empty(&self) -> bool {
        self.character.trim().is_empty()
    }

    /// Check if this cell contains a wide character
    ///
    /// # Returns
    /// `true` if the character is wider than one terminal column
    pub fn is_wide(&self) -> bool {
        self.width > 1
    }

    /// Check if this cell is a continuation of a wide character
    ///
    /// # Returns
    /// `true` if this cell is part of a multi-column character but not the first column
    pub fn is_wide_continuation(&self) -> bool {
        self.width == 0 && self.character.is_empty()
    }

    /// Set a hyperlink for this cell
    ///
    /// # Arguments
    /// * `url` - The URL to link to
    /// * `id` - Optional hyperlink identifier
    pub fn set_hyperlink(&mut self, url: String, id: Option<String>) {
        self.hyperlink = Some(url);
        self.hyperlink_id = id;
        self.dirty = true;
    }

    /// Clear any hyperlink from this cell
    pub fn clear_hyperlink(&mut self) {
        if self.hyperlink.is_some() || self.hyperlink_id.is_some() {
            self.hyperlink = None;
            self.hyperlink_id = None;
            self.dirty = true;
        }
    }

    /// Mark this cell as clean (no longer needing redraw)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Mark this cell as dirty (needing redraw)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Copy all properties from another terminal cell
    ///
    /// # Arguments
    /// * `other` - The cell to copy properties from
    pub fn copy_from(&mut self, other: &TerminalCell) {
        self.character = other.character.clone();
        self.width = other.width;
        self.style = other.style;
        self.wrapped = other.wrapped;
        self.hyperlink = other.hyperlink.clone();
        self.hyperlink_id = other.hyperlink_id.clone();
        self.dirty = true;
    }
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TerminalCell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.character)
    }
}

fn char_width(ch: char) -> u8 {
    UnicodeWidthChar::width(ch).unwrap_or(0) as u8
}

fn string_width(s: &str) -> u8 {
    UnicodeWidthStr::width(s).min(usize::from(u8::MAX)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let cell = TerminalCell::new();
        assert_eq!(cell.character, " ");
        assert_eq!(cell.width, 1);
        assert!(cell.dirty);
    }

    #[test]
    fn test_cell_with_char() {
        let cell = TerminalCell::with_char('A');
        assert_eq!(cell.character, "A");
        assert_eq!(cell.width, 1);
    }

    #[test]
    fn test_cell_wide_char() {
        let cell = TerminalCell::with_char('中');
        assert_eq!(cell.character, "中");
        assert_eq!(cell.width, 2);
        assert!(cell.is_wide());
    }

    #[test]
    fn test_cell_clear() {
        let mut cell = TerminalCell::with_char('A');
        cell.set_style(TerminalStyle {
            bold: true,
            ..TerminalStyle::default()
        });

        cell.clear(TerminalColor::Rgb(255, 0, 0));

        assert_eq!(cell.character, " ");
        assert_eq!(cell.style.background, TerminalColor::Rgb(255, 0, 0));
        assert!(!cell.style.bold);
        assert!(cell.dirty);
    }

    #[test]
    fn test_char_width() {
        assert_eq!(char_width('A'), 1);
        assert_eq!(char_width(' '), 1);
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('\t'), 0);
        assert_eq!(char_width('\n'), 0);
    }

    #[test]
    fn unicode_cell_widths_and_continuations() {
        assert_eq!(TerminalCell::with_char('😀').width, 2);
        assert_eq!(TerminalCell::with_char('\u{301}').width, 0);
        let mut cell = TerminalCell::new();
        for (text, width) in [("e\u{301}", 1), ("👩‍💻", 2), ("🇺🇸", 2), ("", 0)] {
            cell.set_string(text.into());
            assert_eq!(cell.width, width, "{text:?}");
        }
        assert!(cell.is_wide_continuation());
        cell.set_string("x".repeat(512));
        assert_eq!(cell.width, u8::MAX);
    }
}
