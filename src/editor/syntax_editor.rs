//! Text editor with integrated syntax highlighting
//!
//! Combines the gap buffer editor with syntax highlighting for a complete
//! code editing experience.

use super::cursor::{Cursor, Movement};
use super::gap_buffer::{GapBuffer, GapBufferError};
use super::painting::{self, LinePainter};
use crate::core::styled_text::{StyledLine, StyledRun};
use crate::core::surface::{Attr, Rgba, Surface};
use crate::syntax::highlighter::{HighlightedLine, SyntaxHighlighter};
use std::ops::Range;

/// A syntax-highlighted text editor
pub struct SyntaxEditor {
    /// The underlying gap buffer
    buffer: GapBuffer,
    /// Cursor position and selection
    cursor: Cursor,
    /// Syntax highlighter
    highlighter: Option<SyntaxHighlighter>,
    /// Visible lines highlighted with the whole document context (TXT-001).
    highlighted_lines: Vec<HighlightedLine>,
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
            buffer: GapBuffer::default(),
            cursor: Cursor::new(),
            highlighter: None,
            highlighted_lines: Vec::new(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            language: None,
        }
    }

    /// Create an editor with initial content and language
    pub fn with_language(content: &str, language: &str) -> Result<Self, GapBufferError> {
        let highlighter = SyntaxHighlighter::new(language);

        let mut editor = Self {
            buffer: GapBuffer::from_string(content)?,
            cursor: Cursor::new(),
            highlighter,
            highlighted_lines: Vec::new(),
            scroll_offset: 0,
            width: 80,
            height: 24,
            show_line_numbers: true,
            line_number_width: 4,
            language: Some(language.to_string()),
        };

        // Pre-highlight visible content
        editor.rehighlight_visible();
        Ok(editor)
    }

    /// Set the language for syntax highlighting
    pub fn set_language(&mut self, language: &str) {
        self.highlighter = SyntaxHighlighter::new(language);
        self.language = Some(language.to_string());
        self.rehighlight_visible();
    }

    /// Set content from file extension (auto-detect language)
    pub fn set_content_from_file(
        &mut self,
        content: &str,
        filename: &str,
    ) -> Result<(), GapBufferError> {
        self.buffer = GapBuffer::from_string(content)?;
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

        self.rehighlight_visible();
        Ok(())
    }

    /// Set the editor dimensions
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.ensure_cursor_visible();
        self.rehighlight_visible();
    }

    /// Get the current content
    pub fn content(&self) -> String {
        self.buffer.to_string()
    }

    /// Insert text, replacing the selected complete graphemes.
    pub fn insert_text(&mut self, text: &str) -> Result<(), GapBufferError> {
        self.cursor.insert(&mut self.buffer, text)?;
        self.ensure_cursor_visible();
        self.rehighlight_visible();
        Ok(())
    }

    /// Insert one Unicode scalar; adjacent combining text remains one grapheme.
    pub fn insert_char(&mut self, ch: char) -> Result<(), GapBufferError> {
        self.insert_text(ch.encode_utf8(&mut [0; 4]))
    }

    /// Delete the selection or the preceding complete grapheme.
    pub fn delete_backward(&mut self) {
        self.cursor.delete(&mut self.buffer, true);
        self.ensure_cursor_visible();
        self.rehighlight_visible();
    }

    /// Delete the selection or the following complete grapheme.
    pub fn delete_forward(&mut self) {
        self.cursor.delete(&mut self.buffer, false);
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
        } else if cursor_line >= self.scroll_offset.saturating_add(self.height.max(1)) {
            self.scroll_offset = cursor_line - self.height.max(1) + 1;
            self.rehighlight_visible();
        }
    }

    /// Get visible lines range
    fn visible_lines(&self) -> Range<usize> {
        let start = self.scroll_offset;
        let end = (start.saturating_add(self.height)).min(self.buffer.line_count());
        start..end
    }

    /// Re-highlight visible lines
    fn rehighlight_visible(&mut self) {
        let content = self.buffer.to_string();
        let range = self.visible_lines();

        // TXT-001: painting uses only whole-document highlighting.
        self.highlighted_lines = if let Some(ref mut highlighter) = self.highlighter {
            highlighter.highlight_lines(&content, range.start, range.end)
        } else {
            Vec::new()
        };
    }

    /// Render complete graphemes to the requested viewport, clearing stale cells.
    pub fn render(&mut self, surface: &mut Surface, x: usize, y: usize) {
        let lines = self.get_styled_lines();
        painting::render(
            surface,
            lines,
            (x, y),
            (self.width, self.height),
            Rgba {
                r: 0.05,
                g: 0.05,
                b: 0.05,
                a: 1.0,
            },
        );
    }

    /// Visible styled lines, clipped to complete terminal graphemes.
    pub fn get_styled_lines(&mut self) -> Vec<StyledLine> {
        // Every edit, scroll and resize highlights the view already; this
        // catches a view the kept lines do not cover (TXT-001).
        if self.highlighter.is_some() && self.highlighted_lines.len() != self.visible_lines().len()
        {
            self.rehighlight_visible();
        }
        let foreground = Rgba {
            r: 0.9,
            g: 0.9,
            b: 0.9,
            a: 1.0,
        };
        let background = Rgba {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        };
        let mut lines = Vec::new();
        for index in self.visible_lines() {
            let text = self.buffer.get_line(index);
            let source = if let Some(highlighted) =
                self.highlighted_lines.get(index - self.scroll_offset)
            {
                highlighted.to_styled_line()
            } else {
                StyledLine::from_run(StyledRun::new(text, foreground, background, Attr::empty()))
            };
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
                        r: 0.4,
                        g: 0.4,
                        b: 0.4,
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

        let editor = SyntaxEditor::with_language("fn main() {}", "Rust").unwrap();
        assert_eq!(editor.content(), "fn main() {}");
        assert_eq!(editor.language, Some("Rust".to_string()));
    }

    #[test]
    fn test_syntax_editor_auto_detect() {
        let mut editor = SyntaxEditor::new();

        let rust_code = r#"fn main() {
    println!("Hello, world!");
}"#;

        editor.set_content_from_file(rust_code, "main.rs").unwrap();
        assert_eq!(editor.content(), rust_code);
        assert!(editor.language.is_some());
    }

    #[test]
    fn test_syntax_editor_editing() {
        let mut editor = SyntaxEditor::with_language("", "Python").unwrap();

        editor.insert_text("def hello():\n    ").unwrap();
        editor.insert_text("print('Hello')\n").unwrap();

        let content = editor.content();
        assert!(content.contains("def hello():"));
        assert!(content.contains("print('Hello')"));
    }

    #[test]
    fn txt_001_scrolling_and_resizing_keep_multiline_string_context() {
        let document = "let s = r#\"\nfn main() {}\n\"#;";
        let mut editor = SyntaxEditor::with_language(document, "Rust").unwrap();
        editor.set_size(80, 1);
        editor.move_cursor(Movement::Down, false);
        editor.move_cursor(Movement::LineEnd, false);
        assert_eq!(editor.scroll_offset, 1);
        let expected = SyntaxHighlighter::new("Rust")
            .unwrap()
            .highlight_text(document)[1]
            .runs
            .iter()
            .flat_map(|run| run.text.chars().map(move |c| (c, run.fg)))
            .collect::<Vec<_>>();
        for height in [1, 2] {
            editor.set_size(80, height);
            let lines = editor.get_styled_lines();
            let actual = lines[0]
                .runs
                .iter()
                .flat_map(|run| run.text.chars().map(move |c| (c, run.fg)))
                .skip(5)
                .take(expected.len())
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_cache_invalidation_on_edit() {
        let mut editor = SyntaxEditor::with_language("line1\nline2\nline3", "Rust").unwrap();

        // Get initial styled lines to populate cache
        let _ = editor.get_styled_lines();

        // Edit middle line
        editor.move_cursor(Movement::Down, false);
        editor.move_cursor(Movement::LineEnd, false);
        editor.insert_text(" // comment").unwrap();

        // Should have invalidated the edited line
        let content = editor.content();
        assert!(content.contains("line2 // comment"));
    }
}
