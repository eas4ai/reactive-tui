//! Terminal cell implementation

use super::{TerminalColor, TerminalStyle};
use std::fmt;

/// A single cell in the terminal grid
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCell {
    pub character: String,
    pub width: u8,
    pub style: TerminalStyle,
    pub dirty: bool,
    pub wrapped: bool,
    pub hyperlink: Option<String>,
    pub hyperlink_id: Option<String>,
}

impl TerminalCell {
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

    pub fn with_char(ch: char) -> Self {
        let mut cell = Self::new();
        cell.set_char(ch);
        cell
    }

    pub fn set_char(&mut self, ch: char) {
        self.character = ch.to_string();
        self.width = char_width(ch);
        self.dirty = true;
    }

    pub fn set_string(&mut self, s: String) {
        self.width = string_width(&s);
        self.character = s;
        self.dirty = true;
    }

    pub fn set_style(&mut self, style: TerminalStyle) {
        if self.style != style {
            self.style = style;
            self.dirty = true;
        }
    }

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

    pub fn erase(&mut self) {
        let bg = self.style.background;
        self.clear(bg);
    }

    pub fn is_empty(&self) -> bool {
        self.character.trim().is_empty()
    }

    pub fn is_wide(&self) -> bool {
        self.width > 1
    }

    pub fn is_wide_continuation(&self) -> bool {
        self.width == 0 && !self.character.is_empty()
    }

    pub fn set_hyperlink(&mut self, url: String, id: Option<String>) {
        self.hyperlink = Some(url);
        self.hyperlink_id = id;
        self.dirty = true;
    }

    pub fn clear_hyperlink(&mut self) {
        if self.hyperlink.is_some() || self.hyperlink_id.is_some() {
            self.hyperlink = None;
            self.hyperlink_id = None;
            self.dirty = true;
        }
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

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
    match ch {
        '\0'..='\x1F' | '\x7F' => 0,
        ' '..='~' => 1,
        '\u{1100}'..='\u{115F}'
        | '\u{2329}'..='\u{232A}'
        | '\u{2E80}'..='\u{2EFF}'
        | '\u{2F00}'..='\u{2FDF}'
        | '\u{2FF0}'..='\u{2FFF}'
        | '\u{3000}'..='\u{303E}'
        | '\u{3041}'..='\u{3096}'
        | '\u{30A1}'..='\u{30FA}'
        | '\u{3105}'..='\u{312D}'
        | '\u{3131}'..='\u{318E}'
        | '\u{3190}'..='\u{31BA}'
        | '\u{31C0}'..='\u{31E3}'
        | '\u{31F0}'..='\u{31FF}'
        | '\u{3200}'..='\u{32FF}'
        | '\u{3300}'..='\u{33FF}'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{A000}'..='\u{A48C}'
        | '\u{A490}'..='\u{A4C6}'
        | '\u{AC00}'..='\u{D7A3}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{FE10}'..='\u{FE19}'
        | '\u{FE30}'..='\u{FE6F}'
        | '\u{FF00}'..='\u{FF60}'
        | '\u{FFE0}'..='\u{FFE6}'
        | '\u{20000}'..='\u{2FFFD}'
        | '\u{30000}'..='\u{3FFFD}' => 2,
        _ => 1,
    }
}

fn string_width(s: &str) -> u8 {
    s.chars().map(char_width).sum::<u8>().max(1)
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
}
