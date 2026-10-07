//! A scalar gap buffer with bounded character storage.
//!
//! An edit at the gap moves no text. Moving the gap or growing the buffer
//! copies the text between the old and new gap in place; growth may allocate
//! storage. Inserts and deletes update later line-break offsets. `get_range`
//! returns an owned `String`. The byte limit includes the gap, at four bytes
//! per character slot, and every constructor and insert checks it before editing.

use std::fmt;
use std::ops::Range;

/// An edit or allocation would exceed the character-storage byte limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GapBufferError {
    /// Requested character storage, including any existing gap.
    LimitExceeded {
        /// Minimum required character-storage bytes.
        requested_bytes: usize,
        /// Configured maximum character-storage bytes.
        limit_bytes: usize,
    },
}

impl fmt::Display for GapBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded {
                requested_bytes,
                limit_bytes,
            } => write!(
                f,
                "gap buffer needs {requested_bytes} bytes; limit is {limit_bytes} bytes"
            ),
        }
    }
}

impl std::error::Error for GapBufferError {}

/// Text position as line and column
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextPosition {
    /// Line number (0-based)
    pub line: usize,
    /// Unicode scalar column (0-based), not a terminal display column.
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
    limit_bytes: usize,
}

impl GapBuffer {
    /// Default bound on character storage (256 MiB, four bytes per slot).
    pub const DEFAULT_LIMIT_BYTES: usize = 256 * 1024 * 1024;

    /// Create an empty buffer with the default byte limit.
    pub fn new() -> Self {
        // 1024 slots are 4 KiB, far inside the default limit; an empty
        // buffer is the fallback a smaller limit would leave anyway.
        Self::with_capacity(1024).unwrap_or_default()
    }

    /// Create an empty buffer with this initial capacity in character slots.
    pub fn with_capacity(capacity: usize) -> Result<Self, GapBufferError> {
        let mut buffer = Self::default();
        buffer.ensure_gap_capacity(capacity)?;
        Ok(buffer)
    }

    /// Create empty storage with a caller-selected byte limit, including zero.
    pub fn with_limit_bytes(limit_bytes: usize) -> Result<Self, GapBufferError> {
        let mut buffer = Self {
            limit_bytes,
            ..Self::default()
        };
        buffer.ensure_gap_capacity(0)?;
        Ok(buffer)
    }

    /// Create a buffer from text under the default byte limit.
    pub fn from_string(s: &str) -> Result<Self, GapBufferError> {
        Self::from_string_with_limit(s, Self::DEFAULT_LIMIT_BYTES)
    }

    /// Create a buffer from text under a caller-selected byte limit.
    pub fn from_string_with_limit(s: &str, limit_bytes: usize) -> Result<Self, GapBufferError> {
        let mut buffer = Self::with_limit_bytes(limit_bytes)?;
        buffer.insert_str(0, s)?;
        Ok(buffer)
    }

    /// Get the number of Unicode scalars (excluding the gap).
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

    /// Move the gap in place without allocating a temporary text copy.
    fn move_gap_to(&mut self, pos: usize) {
        let pos = pos.min(self.len());
        if pos < self.gap_start {
            let distance = self.gap_start - pos;
            self.buffer
                .copy_within(pos..self.gap_start, self.gap_end - distance);
            self.gap_end -= distance;
        } else {
            let distance = pos - self.gap_start;
            self.buffer
                .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_end += distance;
        }
        self.gap_start = pos;
    }

    /// Check the minimum storage before changing anything; clamp spare capacity
    /// to the same byte limit. All allocation and insertion paths use this check.
    fn ensure_gap_capacity(&mut self, needed: usize) -> Result<(), GapBufferError> {
        let minimum = self.len().saturating_add(needed).max(self.buffer.len());
        let requested_bytes = minimum.saturating_mul(std::mem::size_of::<char>());
        if minimum > self.limit_bytes / std::mem::size_of::<char>() {
            return Err(GapBufferError::LimitExceeded {
                requested_bytes,
                limit_bytes: self.limit_bytes,
            });
        }
        if self.gap_size() >= needed {
            return Ok(());
        }
        let old_len = self.buffer.len();
        let new_len = minimum
            .saturating_add(1024)
            .min(self.limit_bytes / std::mem::size_of::<char>());
        self.buffer.reserve_exact(new_len - old_len);
        self.buffer.resize(new_len, '\0');
        let additional = new_len - old_len;
        self.buffer
            .copy_within(self.gap_end..old_len, self.gap_end + additional);
        self.gap_end += additional;
        Ok(())
    }

    /// Insert one scalar without collecting temporary character storage.
    pub fn insert_char(&mut self, pos: usize, ch: char) -> Result<(), GapBufferError> {
        self.insert_str(pos, ch.encode_utf8(&mut [0; 4]))
    }

    /// Insert text at a scalar offset, clamped to the end. A limit error leaves
    /// text, gap and line offsets unchanged.
    pub fn insert_str(&mut self, pos: usize, s: &str) -> Result<(), GapBufferError> {
        let pos = pos.min(self.len());
        let len = s.chars().count();
        self.ensure_gap_capacity(len)?;
        self.move_gap_to(pos);
        for ch in s.chars() {
            self.buffer[self.gap_start] = ch;
            self.gap_start += 1;
        }
        let first = self.line_breaks.partition_point(|&lb| lb < pos);
        for lb in &mut self.line_breaks[first..] {
            *lb += len;
        }
        self.line_breaks.splice(
            first..first,
            s.chars()
                .enumerate()
                .filter_map(|(i, ch)| (ch == '\n').then_some(pos + i)),
        );
        Ok(())
    }

    /// Preflight a replacement so a refused edit cannot delete the selection.
    pub(super) fn replace_range(
        &mut self,
        range: Range<usize>,
        text: &str,
    ) -> Result<(), GapBufferError> {
        let start = range.start.min(self.len());
        let end = range.end.min(self.len()).max(start);
        self.ensure_gap_capacity(text.chars().count().saturating_sub(end - start))?;
        self.delete_range(start..end);
        self.insert_str(start, text)
    }

    /// Delete one Unicode scalar at the specified scalar offset.
    pub fn delete_char(&mut self, pos: usize) -> Option<char> {
        let ch = self.get_char(pos)?;
        self.delete_range(pos..pos + 1);
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

    /// Return an owned String for a scalar range
    pub fn get_range(&self, range: Range<usize>) -> String {
        let start = range.start.min(self.len());
        let end = range.end.min(self.len());

        let mut result = String::with_capacity(end.saturating_sub(start));

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
        let pos = pos.min(self.len());
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
        line_start.saturating_add(col).min(line_end)
    }
}

impl fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_range(0..self.len()))
    }
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            gap_start: 0,
            gap_end: 0,
            line_breaks: Vec::new(),
            limit_bytes: Self::DEFAULT_LIMIT_BYTES,
        }
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
    fn txt_004_growth_in_the_middle_preserves_suffix_and_line_offsets() {
        let original = format!("left\n{}\nright", "a".repeat(2048));
        let mut buffer = GapBuffer::from_string(&original).unwrap();
        let inserted = format!("{}\n", "b".repeat(2048));
        buffer.insert_str(5, &inserted).unwrap();
        assert_eq!(
            buffer.to_string(),
            format!("left\n{inserted}{}\nright", "a".repeat(2048))
        );
        assert_eq!(buffer.line_count(), 4);
        assert_eq!(buffer.get_line(1), "b".repeat(2048));
        assert_eq!(buffer.get_line(2), "a".repeat(2048));
        assert_eq!(buffer.get_line(3), "right");
    }

    #[test]
    fn test_new_buffer() {
        let buffer = GapBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_from_string() {
        let buffer = GapBuffer::from_string("Hello, World!").unwrap();
        assert_eq!(buffer.len(), 13);
        assert_eq!(buffer.to_string(), "Hello, World!");
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = GapBuffer::from_string("Hello World").unwrap();
        buffer.insert_char(5, ',').unwrap();
        assert_eq!(buffer.to_string(), "Hello, World");
    }

    #[test]
    fn test_insert_string() {
        let mut buffer = GapBuffer::from_string("Hello!").unwrap();
        buffer.insert_str(5, " World").unwrap();
        assert_eq!(buffer.to_string(), "Hello World!");
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = GapBuffer::from_string("Hello, World!").unwrap();
        let ch = buffer.delete_char(5);
        assert_eq!(ch, Some(','));
        assert_eq!(buffer.to_string(), "Hello World!");
    }

    #[test]
    fn test_delete_range() {
        let mut buffer = GapBuffer::from_string("Hello, World!").unwrap();
        buffer.delete_range(5..7);
        assert_eq!(buffer.to_string(), "HelloWorld!");
    }

    #[test]
    fn test_line_operations() {
        let buffer = GapBuffer::from_string("Line 1\nLine 2\nLine 3").unwrap();
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(0), "Line 1");
        assert_eq!(buffer.get_line(1), "Line 2");
        assert_eq!(buffer.get_line(2), "Line 3");
    }

    #[test]
    fn test_line_col_conversion() {
        let buffer = GapBuffer::from_string("Hello\nWorld\n!").unwrap();

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
        buffer.insert_str(0, &large_text).unwrap();
        assert_eq!(buffer.len(), 10000);
        assert_eq!(buffer.to_string(), large_text);
    }

    #[test]
    fn test_cursor_movement_efficiency() {
        let mut buffer = GapBuffer::from_string("Hello World").unwrap();

        // Simulate typical editing pattern
        buffer.insert_char(5, ',').unwrap(); // Copies text to move the gap
        buffer.insert_char(6, ' ').unwrap(); // Moves no text at the gap
        buffer.delete_char(13); // Copies text to move the gap

        assert_eq!(buffer.to_string(), "Hello,  World");
    }
}
