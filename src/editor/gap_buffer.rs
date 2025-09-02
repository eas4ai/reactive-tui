//! Zero-copy gap buffer for efficient text editing
//!
//! A gap buffer is a dynamic array with a gap (empty space) that moves to the
//! location of edits, making insertions and deletions at the cursor O(1).
//!
//! Inspired by r3bl's implementation but adapted for reactive-tui's needs.

use std::fmt;
use std::ops::Range;

/// Text position as line and column
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextPosition {
    /// Line number (0-based)
    pub line: usize,
    /// Column number (0-based)
    pub column: usize,
}

impl TextPosition {
    /// Create a new text position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// A gap buffer for efficient text editing
#[derive(Debug, Clone)]
pub struct GapBuffer {
    /// The buffer containing text with a gap
    buffer: Vec<char>,
    /// Start of the gap (inclusive)
    gap_start: usize,
    /// End of the gap (exclusive)
    gap_end: usize,
    /// Line break indices for efficient line operations
    line_breaks: Vec<usize>,
}

impl GapBuffer {
    /// Create a new empty gap buffer
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new gap buffer with specified initial capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, '\0');

        Self {
            buffer,
            gap_start: 0,
            gap_end: capacity,
            line_breaks: Vec::new(),
        }
    }

    /// Create a gap buffer from a string
    pub fn from_string(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();
        let capacity = len.max(1024);

        let mut buffer = Vec::with_capacity(capacity);
        buffer.extend_from_slice(&chars);
        buffer.resize(capacity, '\0');

        let mut line_breaks = Vec::new();
        for (i, &ch) in chars.iter().enumerate() {
            if ch == '\n' {
                line_breaks.push(i);
            }
        }

        Self {
            buffer,
            gap_start: len,
            gap_end: capacity,
            line_breaks,
        }
    }

    /// Get the total length of text (excluding gap)
    pub fn len(&self) -> usize {
        self.buffer.len() - self.gap_size()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the size of the gap
    fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    /// Convert a logical position to physical position
    fn logical_to_physical(&self, pos: usize) -> usize {
        if pos < self.gap_start {
            pos
        } else {
            pos + self.gap_size()
        }
    }

    /// Move the gap to a specific position
    fn move_gap_to(&mut self, pos: usize) {
        let pos = pos.min(self.len());

        if pos == self.gap_start {
            return; // Already at the right position
        }

        if pos < self.gap_start {
            // Move gap left
            let distance = self.gap_start - pos;
            unsafe {
                let src = self.buffer.as_ptr().add(pos);
                let dst = self.buffer.as_mut_ptr().add(self.gap_end - distance);
                std::ptr::copy(src, dst, distance);
            }
            self.gap_start = pos;
            self.gap_end -= distance;
        } else {
            // Move gap right
            let distance = pos - self.gap_start;
            unsafe {
                let src = self.buffer.as_ptr().add(self.gap_end);
                let dst = self.buffer.as_mut_ptr().add(self.gap_start);
                std::ptr::copy(src, dst, distance);
            }
            self.gap_start = pos;
            self.gap_end += distance;
        }
    }

    /// Ensure the gap has at least the specified capacity
    fn ensure_gap_capacity(&mut self, needed: usize) {
        if self.gap_size() < needed {
            let additional = needed - self.gap_size() + 1024; // Add some extra
            let old_len = self.buffer.len();
            let new_len = old_len + additional;

            // Resize buffer
            self.buffer.resize(new_len, '\0');

            // Move everything after gap_end to the new end
            unsafe {
                let src = self.buffer.as_ptr().add(self.gap_end);
                let dst = self.buffer.as_mut_ptr().add(self.gap_end + additional);
                let count = old_len - self.gap_end;
                std::ptr::copy(src, dst, count);
            }

            self.gap_end += additional;
        }
    }

    /// Insert a character at the specified position
    pub fn insert_char(&mut self, pos: usize, ch: char) {
        self.move_gap_to(pos);
        self.ensure_gap_capacity(1);

        self.buffer[self.gap_start] = ch;
        self.gap_start += 1;

        // Update line breaks if needed
        if ch == '\n' {
            let insert_pos = self.line_breaks.binary_search(&pos).unwrap_or_else(|i| i);
            self.line_breaks.insert(insert_pos, pos);
            // Update subsequent line breaks
            for lb in &mut self.line_breaks[insert_pos + 1..] {
                *lb += 1;
            }
        }
    }

    /// Insert a string at the specified position
    pub fn insert_str(&mut self, pos: usize, s: &str) {
        let chars: Vec<char> = s.chars().collect();
        let len = chars.len();

        self.move_gap_to(pos);
        self.ensure_gap_capacity(len);

        for (i, &ch) in chars.iter().enumerate() {
            self.buffer[self.gap_start + i] = ch;

            if ch == '\n' {
                let line_pos = pos + i;
                let insert_pos = self
                    .line_breaks
                    .binary_search(&line_pos)
                    .unwrap_or_else(|i| i);
                self.line_breaks.insert(insert_pos, line_pos);
            }
        }

        self.gap_start += len;

        // Update subsequent line breaks
        for lb in &mut self.line_breaks {
            if *lb > pos {
                *lb += len;
            }
        }
    }

    /// Delete a character at the specified position
    pub fn delete_char(&mut self, pos: usize) -> Option<char> {
        if pos >= self.len() {
            return None;
        }

        self.move_gap_to(pos);

        let ch = self.buffer[self.gap_end];
        self.gap_end += 1;

        // Update line breaks
        if ch == '\n' {
            if let Ok(idx) = self.line_breaks.binary_search(&pos) {
                self.line_breaks.remove(idx);
            }
            for lb in &mut self.line_breaks {
                if *lb > pos {
                    *lb -= 1;
                }
            }
        }

        Some(ch)
    }

    /// Delete a range of characters
    pub fn delete_range(&mut self, range: Range<usize>) {
        let start = range.start.min(self.len());
        let end = range.end.min(self.len());

        if start >= end {
            return;
        }

        self.move_gap_to(start);

        let delete_count = end - start;

        // Check for deleted line breaks
        let mut deleted_breaks = Vec::new();
        for i in start..end {
            let physical_pos = self.logical_to_physical(i);
            if physical_pos < self.buffer.len() && self.buffer[physical_pos] == '\n' {
                deleted_breaks.push(i);
            }
        }

        // Expand gap to delete
        self.gap_end += delete_count;

        // Remove deleted line breaks and update remaining ones
        self.line_breaks.retain(|&lb| lb < start || lb >= end);
        for lb in &mut self.line_breaks {
            if *lb >= end {
                *lb -= delete_count;
            }
        }
    }

    /// Get a character at the specified position
    pub fn get_char(&self, pos: usize) -> Option<char> {
        if pos >= self.len() {
            return None;
        }

        let physical_pos = self.logical_to_physical(pos);
        Some(self.buffer[physical_pos])
    }

    /// Get a string slice for a range
    pub fn get_range(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.len());
        let end = range.end.min(self.len());

        let mut result = String::with_capacity(end - start);

        for i in start..end {
            if let Some(ch) = self.get_char(i) {
                result.push(ch);
            }
        }

        result
    }

    /// Get the number of lines
    pub fn line_count(&self) -> usize {
        self.line_breaks.len() + 1
    }

    /// Get the start position of a line
    pub fn line_start(&self, line: usize) -> usize {
        if line == 0 {
            0
        } else if line > self.line_breaks.len() {
            self.len()
        } else {
            self.line_breaks[line - 1] + 1
        }
    }

    /// Get the end position of a line (excluding newline)
    pub fn line_end(&self, line: usize) -> usize {
        if line >= self.line_breaks.len() {
            self.len()
        } else {
            self.line_breaks[line]
        }
    }

    /// Get the content of a specific line
    pub fn get_line(&self, line: usize) -> String {
        let start = self.line_start(line);
        let end = self.line_end(line);
        self.get_range(start..end)
    }

    /// Get line and column from a position
    pub fn pos_to_line_col(&self, pos: usize) -> (usize, usize) {
        let line = self.line_breaks.binary_search(&pos).unwrap_or_else(|i| i);
        let line_start = self.line_start(line);
        (line, pos - line_start)
    }

    /// Get text position from a position
    pub fn pos_to_text_position(&self, pos: usize) -> TextPosition {
        let (line, col) = self.pos_to_line_col(pos);
        TextPosition::new(line, col)
    }

    /// Get position from line and column
    pub fn line_col_to_pos(&self, line: usize, col: usize) -> usize {
        let line_start = self.line_start(line);
        let line_end = self.line_end(line);
        (line_start + col).min(line_end)
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_range(0..self.len()))
    }
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Iterator over grapheme clusters in the gap buffer
pub struct GraphemeIterator<'a> {
    _buffer: &'a GapBuffer,
    text: String,
    current: usize,
}

impl<'a> GraphemeIterator<'a> {
    /// Create a new grapheme iterator for the given range
    pub fn new(buffer: &'a GapBuffer, range: Range<usize>) -> Self {
        let text = buffer.get_range(range);

        Self {
            _buffer: buffer,
            text,
            current: 0,
        }
    }

    /// Get the next grapheme cluster from the text
    pub fn next_grapheme(&mut self) -> Option<&str> {
        use unicode_segmentation::UnicodeSegmentation;

        if self.current >= self.text.len() {
            return None;
        }

        let remaining = &self.text[self.current..];
        if let Some((idx, grapheme)) = remaining.grapheme_indices(true).next() {
            self.current += idx + grapheme.len();
            Some(grapheme)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = GapBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_from_string() {
        let buffer = GapBuffer::from_string("Hello, World!");
        assert_eq!(buffer.len(), 13);
        assert_eq!(buffer.to_string(), "Hello, World!");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = GapBuffer::from_string("Hello World");
        buffer.insert_char(5, ',');
        assert_eq!(buffer.to_string(), "Hello, World");
    }

    #[test]
    fn test_insert_string() {
        let mut buffer = GapBuffer::from_string("Hello!");
        buffer.insert_str(5, " World");
        assert_eq!(buffer.to_string(), "Hello World!");
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = GapBuffer::from_string("Hello, World!");
        let ch = buffer.delete_char(5);
        assert_eq!(ch, Some(','));
        assert_eq!(buffer.to_string(), "Hello World!");
    }

    #[test]
    fn test_delete_range() {
        let mut buffer = GapBuffer::from_string("Hello, World!");
        buffer.delete_range(5..7);
        assert_eq!(buffer.to_string(), "HelloWorld!");
    }

    #[test]
    fn test_line_operations() {
        let buffer = GapBuffer::from_string("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(0), "Line 1");
        assert_eq!(buffer.get_line(1), "Line 2");
        assert_eq!(buffer.get_line(2), "Line 3");
    }

    #[test]
    fn test_line_col_conversion() {
        let buffer = GapBuffer::from_string("Hello\nWorld\n!");

        assert_eq!(buffer.pos_to_line_col(0), (0, 0));
        assert_eq!(buffer.pos_to_line_col(5), (0, 5));
        assert_eq!(buffer.pos_to_line_col(6), (1, 0));
        assert_eq!(buffer.pos_to_line_col(11), (1, 5));

        assert_eq!(buffer.line_col_to_pos(0, 0), 0);
        assert_eq!(buffer.line_col_to_pos(1, 0), 6);
        assert_eq!(buffer.line_col_to_pos(2, 0), 12);
    }

    #[test]
    fn test_large_insertions() {
        let mut buffer = GapBuffer::new();
        let large_text = "a".repeat(10000);
        buffer.insert_str(0, &large_text);
        assert_eq!(buffer.len(), 10000);
        assert_eq!(buffer.to_string(), large_text);
    }

    #[test]
    fn test_cursor_movement_efficiency() {
        let mut buffer = GapBuffer::from_string("Hello World");

        // Simulate typical editing pattern
        buffer.insert_char(5, ','); // O(1) after gap move
        buffer.insert_char(6, ' '); // O(1) at gap
        buffer.delete_char(13); // O(1) after gap move

        assert_eq!(buffer.to_string(), "Hello,  World");
    }
}
