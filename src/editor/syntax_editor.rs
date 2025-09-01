//! Text editor with integrated syntax highlighting
//!
//! Combines the gap buffer editor with syntax highlighting for a complete
//! code editing experience.

use super::cursor::{Cursor, Movement};
use super::gap_buffer::GapBuffer;
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Cell, Rgba, Surface};
use crate::syntax::cache::{hash_line_content, LineCache};
use crate::syntax::highlighter::SyntaxHighlighter;
use std::ops::Range;

/// A syntax-highlighted text editor
pub struct SyntaxEditor {
    /// The underlying gap buffer
    buffer: GapBuffer,
    /// Cursor position and selection
    cursor: Cursor,
    /// Syntax highlighter
    highlighter: Option<SyntaxHighlighter>,
    /// Cache for highlighted lines
    cache: LineCache,
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
    /// Language for syntax highlighting
    language: Option<String>,
}

impl SyntaxEditor {
    /// Create a new syntax-highlighted editor
    pub fn new() -> Self {
        Self {
            buffer: GapBuffer::new(),
            cursor: Cursor::new(),
            highlighter: None,
            cache: LineCache::default(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            language: None,
        }
    }

    /// Create an editor with initial content and language
    pub fn with_language(content: &str, language: &str) -> Self {
        let highlighter = SyntaxHighlighter::new(language);

        let mut editor = Self {
            buffer: GapBuffer::from_string(content),
            cursor: Cursor::new(),
            highlighter,
            cache: LineCache::default(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            language: Some(language.to_string()),
        };

        // Pre-highlight visible content
        editor.rehighlight_visible();
        editor
    }

    /// Set the language for syntax highlighting
    pub fn set_language(&mut self, language: &str) {
        self.highlighter = SyntaxHighlighter::new(language);
        self.language = Some(language.to_string());
        self.cache.clear();
        self.rehighlight_visible();
    }

    /// Set content from file extension (auto-detect language)
    pub fn set_content_from_file(&mut self, content: &str, filename: &str) {
        self.buffer = GapBuffer::from_string(content);
        self.cursor = Cursor::new();
        self.scroll_offset = 0;

        // Auto-detect language from extension
        if let Some(ext) = std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
        {
            self.highlighter = SyntaxHighlighter::from_extension(ext);
            if let Some(ref highlighter) = self.highlighter {
                self.language = Some(highlighter.language().to_string());
            }
        }

        self.cache.clear();
        self.rehighlight_visible();
    }

    /// Set the editor dimensions
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.rehighlight_visible();
    }

    /// Get the current content
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// Insert text at cursor position
    pub fn insert_text(&mut self, text: &str) {
        // Delete selection if any
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();

            // Invalidate affected lines
            let start_line = self.buffer.pos_to_line_col(start).0;
            let end_line = self.buffer.pos_to_line_col(end).0;
            self.cache.invalidate_range(start_line, end_line + 1);
        }

        let insert_pos = self.cursor.position;
        let insert_line = self.buffer.pos_to_line_col(insert_pos).0;

        self.buffer.insert_str(self.cursor.position, text);
        self.cursor.position += text.len();

        // Invalidate affected lines
        let lines_added = text.chars().filter(|&c| c == '\n').count();
        self.cache
            .invalidate_range(insert_line, insert_line + lines_added + 1);

        self.ensure_cursor_visible();
        self.rehighlight_visible();
    }

    /// Insert a single character
    pub fn insert_char(&mut self, ch: char) {
        // Delete selection if any
        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();
        }

        let insert_line = self.buffer.pos_to_line_col(self.cursor.position).0;

        self.buffer.insert_char(self.cursor.position, ch);
        self.cursor.position += 1;

        // Invalidate current line (and next if newline)
        if ch == '\n' {
            self.cache.invalidate_range(insert_line, insert_line + 2);
        } else {
            self.cache.invalidate_range(insert_line, insert_line + 1);
        }

        self.ensure_cursor_visible();
        self.rehighlight_visible();
    }

    /// Delete character before cursor (backspace)
    pub fn delete_backward(&mut self) {
        let line_before = self.buffer.pos_to_line_col(self.cursor.position).0;

        if let Some((start, end)) = self.cursor.selection_range() {
            self.buffer.delete_range(start..end);
            self.cursor.position = start;
            self.cursor.clear_selection();

            let start_line = self.buffer.pos_to_line_col(start).0;
            let end_line = self.buffer.pos_to_line_col(end.min(self.buffer.len())).0;
            self.cache.invalidate_range(start_line, end_line + 1);
        } else if self.cursor.position > 0 {
            self.cursor.position -= 1;
            let deleted = self.buffer.delete_char(self.cursor.position);

            // Invalidate affected lines
            if deleted == Some('\n') {
                self.cache
                    .invalidate_range(line_before - 1, line_before + 1);
            } else {
                self.cache.invalidate_range(line_before, line_before + 1);
            }
        }

        self.ensure_cursor_visible();
        self.rehighlight_visible();
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
            self.rehighlight_visible();
        } else if cursor_line >= self.scroll_offset + self.height {
            self.scroll_offset = cursor_line - self.height + 1;
            self.rehighlight_visible();
        }
    }

    /// Get visible lines range
    fn visible_lines(&self) -> Range<usize> {
        let start = self.scroll_offset;
        let end = (start + self.height).min(self.buffer.line_count());
        start..end
    }

    /// Re-highlight visible lines
    fn rehighlight_visible(&mut self) {
        let content = self.buffer.to_string();
        let range = self.visible_lines();

        if let Some(ref mut highlighter) = self.highlighter {
            // Only highlight visible lines
            highlighter.highlight_lines(&content, range.start, range.end);
        }
    }

    /// Get styled lines for the visible portion
    pub fn get_styled_lines(&mut self) -> Vec<StyledLine> {
        let text_bg = Rgba {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        };
        let line_num_fg = Rgba {
            r: 0.4,
            g: 0.4,
            b: 0.4,
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

            let line_start = self.buffer.line_start(line_idx);
            let line_end = self.buffer.line_end(line_idx);
            let line_text = self.buffer.get_range(line_start..line_end);

            // Check if we have syntax highlighting
            if let Some(ref mut highlighter) = self.highlighter {
                // Try cache first
                let hash = hash_line_content(&line_text);

                let highlighted = if let Some(cached) = self.cache.get(line_idx, hash) {
                    cached.clone()
                } else {
                    // Re-highlight this line
                    let highlighted = highlighter.rehighlight_line(&line_text, line_idx);
                    self.cache.insert(line_idx, highlighted.clone(), hash);
                    highlighted
                };

                // Apply selection overlay
                for run in highlighted.runs {
                    if let Some((sel_start, sel_end)) = selection_range {
                        // Check if this run overlaps with selection
                        let run_start = line_start;
                        let run_end = line_start + run.text.len();

                        if run_end > sel_start && run_start < sel_end {
                            // Run has selection - might need to split
                            line.push(StyledRun::new(run.text, run.fg, selection_bg, run.attr));
                        } else {
                            line.push(run);
                        }
                    } else {
                        line.push(run);
                    }
                }
            } else {
                // No syntax highlighting - use plain text
                let fg = Rgba {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                };

                if let Some((sel_start, sel_end)) = selection_range {
                    if line_end > sel_start && line_start < sel_end {
                        // Line has selection
                        line.push(StyledRun::new(line_text, fg, selection_bg, Attr::empty()));
                    } else {
                        line.push(StyledRun::new(line_text, fg, text_bg, Attr::empty()));
                    }
                } else {
                    line.push(StyledRun::new(line_text, fg, text_bg, Attr::empty()));
                }
            }

            // Show cursor at end of line if needed
            if cursor_pos == line_end && line_idx == self.buffer.pos_to_line_col(cursor_pos).0 {
                line.push(StyledRun::new(
                    " ".to_string(),
                    Rgba::black(),
                    cursor_bg,
                    Attr::empty(),
                ));
            }

            lines.push(line);
        }

        lines
    }

    /// Render to a surface
    pub fn render(&mut self, surface: &mut Surface, x: usize, y: usize) {
        let styled_lines = self.get_styled_lines();

        let mut current_y = y;
        for line in styled_lines {
            let mut current_x = x;

            for run in line.runs.iter() {
                for ch in run.text.chars() {
                    if current_x < x + self.width {
                        surface.set(
                            current_x,
                            current_y,
                            Cell {
                                ch,
                                fg: run.fg,
                                bg: run.bg,
                                attr: run.attr,
                                image_id: None,
                                image_placement: None,
                            },
                        );
                        current_x += 1;
                    }
                }
            }

            current_y += 1;
            if current_y >= y + self.height {
                break;
            }
        }
    }
}

impl Default for SyntaxEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_editor_creation() {
        let editor = SyntaxEditor::new();
        assert_eq!(editor.content(), "");

        let editor = SyntaxEditor::with_language("fn main() {}", "Rust");
        assert_eq!(editor.content(), "fn main() {}");
        assert_eq!(editor.language, Some("Rust".to_string()));
    }

    #[test]
    fn test_syntax_editor_auto_detect() {
        let mut editor = SyntaxEditor::new();

        let rust_code = r#"fn main() {
    println!("Hello, world!");
}"#;

        editor.set_content_from_file(rust_code, "main.rs");
        assert_eq!(editor.content(), rust_code);
        assert!(editor.language.is_some());
    }

    #[test]
    fn test_syntax_editor_editing() {
        let mut editor = SyntaxEditor::with_language("", "Python");

        editor.insert_text("def hello():\n    ");
        editor.insert_text("print('Hello')\n");

        let content = editor.content();
        assert!(content.contains("def hello():"));
        assert!(content.contains("print('Hello')"));
    }

    #[test]
    fn test_cache_invalidation_on_edit() {
        let mut editor = SyntaxEditor::with_language("line1\nline2\nline3", "Rust");

        // Get initial styled lines to populate cache
        let _ = editor.get_styled_lines();

        // Edit middle line
        editor.move_cursor(Movement::Down, false);
        editor.move_cursor(Movement::LineEnd, false);
        editor.insert_text(" // comment");

        // Should have invalidated the edited line
        let content = editor.content();
        assert!(content.contains("line2 // comment"));
    }
}
