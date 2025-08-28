//! Text editor widget with gap buffer backend
//!
//! A complete text editor component with cursor, selection, and efficient editing.

use super::cursor::{Cursor, Movement};
use super::gap_buffer::GapBuffer;
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Cell, Rgba, Surface};
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
            buffer: GapBuffer::from_str(content),
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
        self.buffer = GapBuffer::from_str(content);
        self.cursor = Cursor::new();
        self.scroll_offset = 0;
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        // Delete selection if any
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();
        }

        self.buffer.insert_str(self.cursor.position, text);
        self.cursor.position += text.len();
        self.ensure_cursor_visible();
    }

    /// Insert a single character
    pub fn insert_char(&mut self, ch: char) {
        // Delete selection if any
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();
        }

        self.buffer.insert_char(self.cursor.position, ch);
        self.cursor.position += 1;
        self.ensure_cursor_visible();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) {
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();
        } else if self.cursor.position > 0 {
            self.cursor.position -= 1;
            self.buffer.delete_char(self.cursor.position);
        }
        self.ensure_cursor_visible();
    }

    /// Delete character at cursor (delete)
    pub fn delete_forward(&mut self) {
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();
        } else {
            self.buffer.delete_char(self.cursor.position);
        }
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
        } else if cursor_line >= self.scroll_offset + self.height {
            self.scroll_offset = cursor_line - self.height + 1;
        }
    }

    /// Get visible lines range
    fn visible_lines(&self) -> Range<usize> {
        let start = self.scroll_offset;
        let end = (start + self.height).min(self.buffer.line_count());
        start..end
    }

    /// Render to a surface
    pub fn render(&self, surface: &mut Surface, x: usize, y: usize) {
        let text_fg = Rgba {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };
        let text_bg = Rgba {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };
        let line_num_fg = Rgba {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        let selection_bg = Rgba {
            r: 0.2,
            g: 0.4,
            b: 0.6,
            a: 1.0,
        };
        let cursor_bg = Rgba {
            r: 0.8,
            g: 0.8,
            b: 0.8,
            a: 1.0,
        };

        let selection_range = self.cursor.selection_range();
        let cursor_pos = self.cursor.position;

        let mut current_y = y;
        let text_start_x = if self.show_line_numbers {
            x + self.line_number_width + 1
        } else {
            x
        };

        for line_idx in self.visible_lines() {
            // Render line number
            if self.show_line_numbers {
                let line_num = format!("{:>width$}", line_idx + 1, width = self.line_number_width);
                surface.write_str(x, current_y, &line_num, line_num_fg, text_bg, Attr::empty());
                surface.write_str(
                    x + self.line_number_width,
                    current_y,
                    " ",
                    line_num_fg,
                    text_bg,
                    Attr::empty(),
                );
            }

            // Get line content
            let line_start = self.buffer.line_start(line_idx);
            let line_end = self.buffer.line_end(line_idx);
            let line_text = self.buffer.get_range(line_start..line_end);

            // Render line content with selection and cursor
            let mut current_x = text_start_x;

            for (char_idx, ch) in line_text.chars().enumerate() {
                let pos = line_start + char_idx;
                let mut bg = text_bg;
                let mut fg = text_fg;
                let attr = Attr::empty();

                // Check if character is in selection
                if let Some((sel_start, sel_end)) = selection_range {
                    if pos >= sel_start && pos < sel_end {
                        bg = selection_bg;
                    }
                }

                // Check if this is cursor position
                if pos == cursor_pos {
                    bg = cursor_bg;
                    fg = Rgba {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    };
                }

                if current_x < x + self.width {
                    surface.set(current_x, current_y, Cell { ch, fg, bg, attr });
                    current_x += 1;
                }
            }

            // Show cursor at end of line if needed
            if cursor_pos == line_end && line_idx == self.buffer.pos_to_line_col(cursor_pos).0 {
                if current_x < x + self.width {
                    surface.set(
                        current_x,
                        current_y,
                        Cell {
                            ch: ' ',
                            fg: Rgba {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            },
                            bg: cursor_bg,
                            attr: Attr::empty(),
                        },
                    );
                }
            }

            current_y += 1;
            if current_y >= y + self.height {
                break;
            }
        }
    }

    /// Get styled lines for the visible portion (for RenderOps pipeline)
    pub fn get_styled_lines(&self) -> Vec<StyledLine> {
        let text_fg = Rgba {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };
        let text_bg = Rgba {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };
        let line_num_fg = Rgba {
            r: 0.5,
            g: 0.5,
            b: 0.5,
            a: 1.0,
        };
        let selection_bg = Rgba {
            r: 0.2,
            g: 0.4,
            b: 0.6,
            a: 1.0,
        };

        let selection_range = self.cursor.selection_range();
        let mut lines = Vec::new();

        for line_idx in self.visible_lines() {
            let mut line = StyledLine::new();

            // Add line number
            if self.show_line_numbers {
                let line_num = format!("{:>width$} ", line_idx + 1, width = self.line_number_width);
                line.push(StyledRun::new(
                    line_num,
                    line_num_fg,
                    text_bg,
                    Attr::empty(),
                ));
            }

            // Add line content
            let line_start = self.buffer.line_start(line_idx);
            let line_end = self.buffer.line_end(line_idx);
            let line_text = self.buffer.get_range(line_start..line_end);

            // Check if any part of line is selected
            if let Some((sel_start, sel_end)) = selection_range {
                if line_end > sel_start && line_start < sel_end {
                    // Line has selection - split into runs
                    let mut pos = line_start;
                    let mut text_iter = line_text.chars();
                    let mut current_text = String::new();

                    while pos < line_end {
                        if let Some(ch) = text_iter.next() {
                            if pos >= sel_start && pos < sel_end {
                                // In selection
                                if !current_text.is_empty() {
                                    line.push(StyledRun::new(
                                        current_text.clone(),
                                        text_fg,
                                        text_bg,
                                        Attr::empty(),
                                    ));
                                    current_text.clear();
                                }

                                // Add selected character
                                line.push(StyledRun::new(
                                    ch.to_string(),
                                    text_fg,
                                    selection_bg,
                                    Attr::empty(),
                                ));
                            } else {
                                current_text.push(ch);
                            }
                            pos += 1;
                        } else {
                            break;
                        }
                    }

                    // Add remaining text
                    if !current_text.is_empty() {
                        line.push(StyledRun::new(
                            current_text,
                            text_fg,
                            text_bg,
                            Attr::empty(),
                        ));
                    }
                } else {
                    // No selection on this line
                    if !line_text.is_empty() {
                        line.push(StyledRun::new(line_text, text_fg, text_bg, Attr::empty()));
                    }
                }
            } else {
                // No selection at all
                if !line_text.is_empty() {
                    line.push(StyledRun::new(line_text, text_fg, text_bg, Attr::empty()));
                }
            }

            lines.push(line);
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
