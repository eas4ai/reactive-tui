//! Cursor management for text editor
//!
//! Handles cursor position, movement, and selection within a gap buffer.

use super::gap_buffer::GapBuffer;
use super::positions;

/// Cursor position and movement controller
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Current Unicode scalar offset; movement snaps to grapheme boundaries.
    pub position: usize,
    /// Selection anchor position (None if no selection)
    pub selection_anchor: Option<usize>,
    /// Preferred terminal display column for vertical movement
    preferred_column: Option<usize>,
}

impl Cursor {
    /// Create a new cursor at the beginning
    pub fn new() -> Self {
        Self {
            position: 0,
            selection_anchor: None,
            preferred_column: None,
        }
    }

    /// Move cursor to an absolute position
    pub fn move_to(&mut self, position: usize, buffer: &GapBuffer) {
        self.position = positions::floor(buffer, position);
        self.preferred_column = None;
    }

    /// Move left by one complete grapheme.
    pub fn move_left(&mut self, buffer: &GapBuffer) {
        self.position = positions::previous(buffer, self.position);
        self.preferred_column = None;
    }

    /// Move right by one complete grapheme.
    pub fn move_right(&mut self, buffer: &GapBuffer) {
        self.position = positions::next(buffer, self.position);
        self.preferred_column = None;
    }

    /// Move up, preserving the preferred terminal display column.
    pub fn move_up(&mut self, buffer: &GapBuffer) {
        let line = buffer.pos_to_line_col(self.position).0;
        self.move_vertical(buffer, line.saturating_sub(1));
    }

    /// Move down, preserving the preferred terminal display column.
    pub fn move_down(&mut self, buffer: &GapBuffer) {
        let line = buffer.pos_to_line_col(self.position).0;
        self.move_vertical(buffer, (line + 1).min(buffer.line_count() - 1));
    }

    fn move_vertical(&mut self, buffer: &GapBuffer, line: usize) {
        let column = *self
            .preferred_column
            .get_or_insert_with(|| positions::display_column(buffer, self.position));
        self.position = positions::at_column(buffer, line, column);
    }

    /// Move to beginning of line
    pub fn move_to_line_start(&mut self, buffer: &GapBuffer) {
        let (line, _) = buffer.pos_to_line_col(self.position);
        self.position = buffer.line_start(line);
        self.preferred_column = None;
    }

    /// Move to end of line
    pub fn move_to_line_end(&mut self, buffer: &GapBuffer) {
        let (line, _) = buffer.pos_to_line_col(self.position);
        self.position = positions::floor(buffer, buffer.line_end(line));
        self.preferred_column = None;
    }

    /// Move to beginning of buffer
    pub fn move_to_start(&mut self) {
        self.position = 0;
        self.preferred_column = None;
    }

    /// Move to end of buffer
    pub fn move_to_end(&mut self, buffer: &GapBuffer) {
        self.position = buffer.len();
        self.preferred_column = None;
    }

    /// Skip separators, then a word of alphanumeric/underscore graphemes.
    pub fn move_word_forward(&mut self, buffer: &GapBuffer) {
        self.position = positions::floor(buffer, self.position);
        let text = buffer.get_range(self.position..buffer.len());
        let mut in_word = false;
        for grapheme in text.graphemes(true) {
            let word = grapheme.chars().any(|ch| ch.is_alphanumeric() || ch == '_');
            if in_word && !word {
                break;
            }
            in_word |= word;
            self.position += grapheme.chars().count();
        }
        self.preferred_column = None;
    }

    /// Skip separators backward, then move to the beginning of a word.
    pub fn move_word_backward(&mut self, buffer: &GapBuffer) {
        self.position = positions::floor(buffer, self.position);
        let text = buffer.get_range(0..self.position);
        let mut in_word = false;
        for grapheme in text.graphemes(true).rev() {
            let word = grapheme.chars().any(|ch| ch.is_alphanumeric() || ch == '_');
            if in_word && !word {
                break;
            }
            in_word |= word;
            self.position -= grapheme.chars().count();
        }
        self.preferred_column = None;
    }

    /// Replace the selection or insert at a complete grapheme boundary.
    pub(super) fn insert(&mut self, buffer: &mut GapBuffer, text: &str) {
        self.delete_selection(buffer);
        self.position = positions::floor(buffer, self.position);
        buffer.insert_str(self.position, text);
        self.position = positions::ceil(buffer, self.position + text.chars().count());
        self.preferred_column = None;
    }

    pub(super) fn delete(&mut self, buffer: &mut GapBuffer, backward: bool) {
        if !self.delete_selection(buffer) {
            let position = positions::floor(buffer, self.position);
            let (start, end) = if backward {
                (positions::previous(buffer, position), position)
            } else {
                (position, positions::next(buffer, position))
            };
            buffer.delete_range(start..end);
            self.position = positions::floor(buffer, start);
        }
        self.preferred_column = None;
    }

    fn delete_selection(&mut self, buffer: &mut GapBuffer) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };
        let start = positions::floor(buffer, start);
        let end = positions::ceil(buffer, end);
        buffer.delete_range(start..end);
        self.position = positions::floor(buffer, start);
        self.clear_selection();
        true
    }

    /// Start selection at current position
    pub fn start_selection(&mut self) {
        self.selection_anchor = Some(self.position);
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    /// Get current selection range (if any)
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_anchor.map(|anchor| {
            if anchor < self.position {
                (anchor, self.position)
            } else {
                (self.position, anchor)
            }
        })
    }

    /// Check if there's an active selection
    pub fn has_selection(&self) -> bool {
        self.selection_anchor.is_some()
    }

    /// Get the selected text
    pub fn selected_text(&self, buffer: &GapBuffer) -> Option<String> {
        self.selection_range()
            .map(|(start, end)| buffer.get_range(start..end))
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

/// Cursor movement commands
/// Cursor movement directions and types
#[derive(Debug, Clone, Copy)]
pub enum Movement {
    /// Move cursor one position left
    Left,
    /// Move cursor one position right
    Right,
    /// Move cursor one line up
    Up,
    /// Move cursor one line down
    Down,
    /// Move cursor to start of current line
    LineStart,
    /// Move cursor to end of current line
    LineEnd,
    /// Move cursor to start of document
    DocumentStart,
    /// Move cursor to end of document
    DocumentEnd,
    /// Move cursor forward by one word
    WordForward,
    /// Move cursor backward by one word
    WordBackward,
    /// Move cursor up by one page
    PageUp,
    /// Move cursor down by one page
    PageDown,
}

impl Cursor {
    /// Execute a movement command
    pub fn execute_movement(&mut self, movement: Movement, buffer: &GapBuffer) {
        match movement {
            Movement::Left => self.move_left(buffer),
            Movement::Right => self.move_right(buffer),
            Movement::Up => self.move_up(buffer),
            Movement::Down => self.move_down(buffer),
            Movement::LineStart => self.move_to_line_start(buffer),
            Movement::LineEnd => self.move_to_line_end(buffer),
            Movement::DocumentStart => self.move_to_start(),
            Movement::DocumentEnd => self.move_to_end(buffer),
            Movement::WordForward => self.move_word_forward(buffer),
            Movement::WordBackward => self.move_word_backward(buffer),
            Movement::PageUp => {
                // Move up by viewport height (approximate)
                for _ in 0..20 {
                    self.move_up(buffer);
                }
            }
            Movement::PageDown => {
                // Move down by viewport height (approximate)
                for _ in 0..20 {
                    self.move_down(buffer);
                }
            }
        }
    }
}

use unicode_segmentation::UnicodeSegmentation;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_basic_movement() {
        let buffer = GapBuffer::from_string("Hello\nWorld");
        let mut cursor = Cursor::new();

        assert_eq!(cursor.position, 0);

        cursor.move_right(&buffer);
        assert_eq!(cursor.position, 1);

        cursor.move_to_line_end(&buffer);
        assert_eq!(cursor.position, 5); // End of "Hello"

        cursor.move_down(&buffer);
        let (line, col) = buffer.pos_to_line_col(cursor.position);
        assert_eq!(line, 1); // Should be on second line
        assert_eq!(
            cursor.position,
            buffer.line_start(1) + col.min(buffer.line_end(1) - buffer.line_start(1))
        );
    }

    #[test]
    fn test_cursor_vertical_movement() {
        let buffer = GapBuffer::from_string("12345\n123\n12345");
        let mut cursor = Cursor::new();

        cursor.move_to(3, &buffer); // Position at '4' in first line
        cursor.move_down(&buffer);
        assert_eq!(cursor.position, 9); // End of second line (shorter)

        cursor.move_down(&buffer);
        assert_eq!(cursor.position, 13); // Position at '4' in third line (preferred column preserved)
    }

    #[test]
    fn test_cursor_word_movement() {
        let buffer = GapBuffer::from_string("hello world foo_bar");
        let mut cursor = Cursor::new();

        cursor.move_word_forward(&buffer);
        assert_eq!(cursor.position, 5); // After "hello"

        cursor.move_word_forward(&buffer);
        assert_eq!(cursor.position, 11); // After "world"

        cursor.move_word_backward(&buffer);
        assert_eq!(cursor.position, 6); // Beginning of "world"
    }

    #[test]
    fn test_cursor_selection() {
        let buffer = GapBuffer::from_string("Hello World");
        let mut cursor = Cursor::new();

        cursor.move_to(6, &buffer);
        cursor.start_selection();
        cursor.move_to(11, &buffer);

        assert_eq!(cursor.selection_range(), Some((6, 11)));
        assert_eq!(cursor.selected_text(&buffer), Some("World".to_string()));

        cursor.clear_selection();
        assert!(!cursor.has_selection());
    }
}
