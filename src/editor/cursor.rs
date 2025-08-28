//! Cursor management for text editor
//!
//! Handles cursor position, movement, and selection within a gap buffer.

use super::gap_buffer::GapBuffer;

/// Cursor position and movement controller
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Current position in the buffer (byte offset)
    pub position: usize,
    /// Selection anchor position (None if no selection)
    pub selection_anchor: Option<usize>,
    /// Preferred column for vertical movement
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
        self.position = position.min(buffer.len());
        self.preferred_column = None;
    }

    /// Move cursor left by one character
    pub fn move_left(&mut self, buffer: &GapBuffer) {
        if self.position > 0 {
            // Move by grapheme cluster for proper Unicode support
            let text = buffer.get_range(0..self.position);
            if let Some((last_idx, _)) = text.grapheme_indices(true).last() {
                self.position = last_idx;
            } else {
                self.position = 0;
            }
        }
        self.preferred_column = None;
    }

    /// Move cursor right by one character
    pub fn move_right(&mut self, buffer: &GapBuffer) {
        if self.position < buffer.len() {
            let text = buffer.get_range(self.position..buffer.len());
            if let Some((idx, grapheme)) = text.grapheme_indices(true).next() {
                self.position += idx + grapheme.len();
            } else {
                self.position = buffer.len();
            }
        }
        self.preferred_column = None;
    }

    /// Move cursor up one line
    pub fn move_up(&mut self, buffer: &GapBuffer) {
        let (line, col) = buffer.pos_to_line_col(self.position);

        if line > 0 {
            let target_line = line - 1;
            let preferred_col = self.preferred_column.unwrap_or(col);

            // Try to maintain column position
            let line_start = buffer.line_start(target_line);
            let line_end = buffer.line_end(target_line);
            let line_len = line_end - line_start;

            let new_col = preferred_col.min(line_len);
            self.position = line_start + new_col;

            // Preserve preferred column for consecutive vertical movements
            if self.preferred_column.is_none() {
                self.preferred_column = Some(col);
            }
        }
    }

    /// Move cursor down one line
    pub fn move_down(&mut self, buffer: &GapBuffer) {
        let (line, col) = buffer.pos_to_line_col(self.position);

        if line < buffer.line_count() - 1 {
            let target_line = line + 1;
            let preferred_col = self.preferred_column.unwrap_or(col);

            // Try to maintain column position
            let line_start = buffer.line_start(target_line);
            let line_end = buffer.line_end(target_line);
            let line_len = line_end - line_start;

            let new_col = preferred_col.min(line_len);
            self.position = line_start + new_col;

            // Preserve preferred column for consecutive vertical movements
            if self.preferred_column.is_none() {
                self.preferred_column = Some(col);
            }
        }
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
        self.position = buffer.line_end(line);
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

    /// Move by word forward
    pub fn move_word_forward(&mut self, buffer: &GapBuffer) {
        let text = buffer.get_range(self.position..buffer.len());
        let mut in_word = false;
        let mut new_pos = self.position;

        for (idx, ch) in text.char_indices() {
            if ch.is_alphanumeric() || ch == '_' {
                in_word = true;
            } else if in_word {
                new_pos = self.position + idx;
                break;
            }
        }

        // If we didn't find a word boundary, move to end
        if new_pos == self.position && !text.is_empty() {
            new_pos = buffer.len();
        }

        self.position = new_pos;
        self.preferred_column = None;
    }

    /// Move by word backward
    pub fn move_word_backward(&mut self, buffer: &GapBuffer) {
        if self.position == 0 {
            return;
        }

        let text = buffer.get_range(0..self.position);
        let mut in_word = false;
        let mut new_pos = 0;

        for (idx, ch) in text.char_indices().rev() {
            if ch.is_alphanumeric() || ch == '_' {
                in_word = true;
            } else if in_word {
                new_pos = idx + 1;
                break;
            }
        }

        self.position = new_pos;
        self.preferred_column = None;
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
#[derive(Debug, Clone, Copy)]
pub enum Movement {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    DocumentStart,
    DocumentEnd,
    WordForward,
    WordBackward,
    PageUp,
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
        let buffer = GapBuffer::from_str("Hello\nWorld");
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
        let buffer = GapBuffer::from_str("12345\n123\n12345");
        let mut cursor = Cursor::new();

        cursor.move_to(3, &buffer); // Position at '4' in first line
        cursor.move_down(&buffer);
        assert_eq!(cursor.position, 9); // End of second line (shorter)

        cursor.move_down(&buffer);
        assert_eq!(cursor.position, 13); // Position at '4' in third line (preferred column preserved)
    }

    #[test]
    fn test_cursor_word_movement() {
        let buffer = GapBuffer::from_str("hello world foo_bar");
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
        let buffer = GapBuffer::from_str("Hello World");
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
