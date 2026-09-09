//! Text editor widget with gap buffer backend
//!
//! A complete text editor component with cursor, selection, and efficient editing.

use super::cursor::{Cursor, Movement};
use super::gap_buffer::GapBuffer;
use super::painting::{self, LinePainter};
use crate::core::geometry::{Point, Size};
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba, Surface};
use std::ops::Range;

/// A text editor widget
#[derive(Debug)]
pub struct TextEditor {
    /// The underlying gap buffer
    buffer: GapBuffer,
    /// Cursor position and selection
    cursor: Cursor,
    /// Viewport scroll offset (line number)
    scroll_offset: usize,
    /// Width of the editor
    width: usize,
    /// Height of the editor
    height: usize,
    /// Line numbers enabled
    show_line_numbers: bool,
    /// Line number width
    line_number_width: usize,
    /// Syntax highlighting enabled
    syntax_highlighting: bool,
}

impl TextEditor {
    /// Create a new text editor
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::new(),
            cursor: Cursor::new(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            syntax_highlighting: false,
        }
    }

    /// Create a text editor with initial content
    pub fn with_content(content: &str) -> Self {
        Self {
            buffer: GapBuffer::from_string(content),
            cursor: Cursor::new(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            syntax_highlighting: false,
        }
    }

    /// Set the editor dimensions
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.ensure_cursor_visible();
    }

    /// Set editor viewport using Size
    pub fn set_viewport(&mut self, size: Size) {
        self.width = size.width;
        self.height = size.height;
        self.ensure_cursor_visible();
    }

    /// Enable or disable line numbers
    pub fn set_show_line_numbers(&mut self, show: bool) {
        self.show_line_numbers = show;
    }

    /// Enable or disable syntax highlighting
    pub fn set_syntax_highlighting(&mut self, enabled: bool) {
        self.syntax_highlighting = enabled;
    }

    /// Get the current content
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// Set the content
    pub fn set_content(&mut self, content: &str) {
        self.buffer = GapBuffer::from_string(content);
        self.cursor = Cursor::new();
        self.scroll_offset = 0;
    }

    /// Insert text, replacing the selected complete graphemes.
    pub fn insert_text(&mut self, text: &str) {
        self.cursor.insert(&mut self.buffer, text);
        self.ensure_cursor_visible();
    }

    /// Insert one Unicode scalar; adjacent combining text remains one grapheme.
    pub fn insert_char(&mut self, ch: char) {
        self.insert_text(ch.encode_utf8(&mut [0; 4]));
    }

    /// Delete the selection or the preceding complete grapheme.
    pub fn delete_backward(&mut self) {
        self.cursor.delete(&mut self.buffer, true);
        self.ensure_cursor_visible();
    }

    /// Delete the selection or the following complete grapheme.
    pub fn delete_forward(&mut self) {
        self.cursor.delete(&mut self.buffer, false);
        self.ensure_cursor_visible();
    }

    /// Move cursor
    pub fn move_cursor(&mut self, movement: Movement, select: bool) {
        if select && !self.cursor.has_selection() {
            self.cursor.start_selection();
        } else if !select && self.cursor.has_selection() {
            self.cursor.clear_selection();
        }

        self.cursor.execute_movement(movement, &self.buffer);
        self.ensure_cursor_visible();
    }

    /// Ensure cursor is visible in viewport
    fn ensure_cursor_visible(&mut self) {
        let (cursor_line, _) = self.buffer.pos_to_line_col(self.cursor.position);

        if cursor_line < self.scroll_offset {
            self.scroll_offset = cursor_line;
        } else if cursor_line >= self.scroll_offset.saturating_add(self.height.max(1)) {
            self.scroll_offset = cursor_line - self.height.max(1) + 1;
        }
    }

    /// Get visible lines range
    fn visible_lines(&self) -> Range<usize> {
        let start = self.scroll_offset;
        let end = (start.saturating_add(self.height)).min(self.buffer.line_count());
        start..end
    }

    /// Render to a surface at a point
    pub fn render_at(&self, surface: &mut Surface, origin: Point) {
        self.render(surface, origin.x, origin.y)
    }

    /// Render complete graphemes to the requested viewport, clearing stale cells.
    pub fn render(&self, surface: &mut Surface, x: usize, y: usize) {
        let lines = self.get_styled_lines();
        painting::render(
            surface,
            lines,
            (x, y),
            (self.width, self.height),
            Rgba {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
        );
    }

    /// Visible styled lines, clipped to complete terminal graphemes.
    pub fn get_styled_lines(&self) -> Vec<StyledLine> {
        let foreground = Rgba {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };
        let background = Rgba {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };
        let mut lines = Vec::new();
        for index in self.visible_lines() {
            let text = self.buffer.get_line(index);
            let source =
                StyledLine::from_run(StyledRun::new(text, foreground, background, Attr::empty()));
            lines.push(
                LinePainter {
                    buffer: &self.buffer,
                    cursor: &self.cursor,
                    width: self.width,
                    gutter: if self.show_line_numbers {
                        self.line_number_width + 1
                    } else {
                        0
                    },
                    number_fg: Rgba {
                        r: 0.5,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    },
                    background,
                }
                .paint(index, source),
            );
        }
        lines
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = TextEditor::new();
        assert_eq!(editor.content(), "");

        let editor = TextEditor::with_content("Hello World");
        assert_eq!(editor.content(), "Hello World");
    }

    #[test]
    fn test_editor_insert() {
        let mut editor = TextEditor::new();

        editor.insert_text("Hello");
        assert_eq!(editor.content(), "Hello");

        editor.insert_char(' ');
        assert_eq!(editor.content(), "Hello ");

        editor.insert_text("World");
        assert_eq!(editor.content(), "Hello World");
    }

    #[test]
    fn test_editor_delete() {
        let mut editor = TextEditor::with_content("Hello World");

        editor.move_cursor(Movement::DocumentEnd, false);
        editor.delete_backward();
        assert_eq!(editor.content(), "Hello Worl");

        editor.move_cursor(Movement::DocumentStart, false);
        editor.delete_forward();
        assert_eq!(editor.content(), "ello Worl");
    }

    #[test]
    fn test_editor_selection() {
        let mut editor = TextEditor::with_content("Hello World");

        editor.move_cursor(Movement::WordForward, false);
        editor.move_cursor(Movement::Right, false);
        editor.move_cursor(Movement::WordForward, true);

        // Should have "World" selected
        assert_eq!(
            editor.cursor.selected_text(&editor.buffer),
            Some("World".to_string())
        );

        editor.insert_text("Rust");
        assert_eq!(editor.content(), "Hello Rust");
    }
}
