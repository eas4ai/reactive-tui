//! Efficient row-based diffing for minimal terminal updates
//!
//! Instead of comparing cells one by one, this module compares
//! surfaces row by row using styled spans, reducing ANSI escape sequences.

use super::grapheme_cell::{GraphemeSurface, Span};
use super::surface::{Attr, Rgba};
use std::io::Write as _;

/// Efficient diff writer that outputs minimal ANSI sequences
pub struct SpanDiffWriter {
    output: Vec<u8>,
    current_fg: Option<Rgba>,
    current_bg: Option<Rgba>,
    current_attr: Attr,
    cursor_x: usize,
    cursor_y: usize,
}

impl Default for SpanDiffWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl SpanDiffWriter {
    /// Create a new diff writer
    pub fn new() -> Self {
        Self {
            output: Vec::with_capacity(64 * 1024), // 64KB initial capacity
            current_fg: None,
            current_bg: None,
            current_attr: Attr::empty(),
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    /// Get the output buffer
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Clear the output buffer for reuse
    pub fn clear(&mut self) {
        self.output.clear();
        self.current_fg = None;
        self.current_bg = None;
        self.current_attr = Attr::empty();
        // Set cursor to invalid position to force first move_to to emit escape sequence
        self.cursor_x = usize::MAX;
        self.cursor_y = usize::MAX;
    }

    /// Write raw bytes
    #[inline]
    fn write_raw(&mut self, bytes: &[u8]) {
        self.output.extend_from_slice(bytes);
    }

    /// Write a string
    #[inline]
    fn write_str(&mut self, s: &str) {
        self.write_raw(s.as_bytes());
    }

    /// Move cursor to position
    fn move_to(&mut self, x: usize, y: usize) {
        if self.cursor_x != x || self.cursor_y != y {
            // Use 1-based indexing for terminals
            if write!(&mut self.output, "\x1b[{};{}H", y + 1, x + 1).is_ok() {
                self.cursor_x = x;
                self.cursor_y = y;
            }
            // If write fails, cursor position becomes unknown but we continue
        }
    }

    /// Set foreground color
    fn set_fg(&mut self, color: Rgba) {
        if self.current_fg != Some(color) {
            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            if write!(&mut self.output, "\x1b[38;2;{};{};{}m", r, g, b).is_ok() {
                self.current_fg = Some(color);
            }
            // If write fails, color state becomes inconsistent but we continue
        }
    }

    /// Set background color
    fn set_bg(&mut self, color: Rgba) {
        if self.current_bg != Some(color) {
            let r = (color.r * 255.0) as u8;
            let g = (color.g * 255.0) as u8;
            let b = (color.b * 255.0) as u8;
            if write!(&mut self.output, "\x1b[48;2;{};{};{}m", r, g, b).is_ok() {
                self.current_bg = Some(color);
            }
            // If write fails, color state becomes inconsistent but we continue
        }
    }

    /// Apply text attributes
    fn set_attr(&mut self, attr: Attr) {
        let old = self.current_attr;

        // Reset attributes that are no longer needed
        if old.contains(Attr::BOLD) && !attr.contains(Attr::BOLD) {
            self.write_str("\x1b[22m");
        }
        if old.contains(Attr::ITALIC) && !attr.contains(Attr::ITALIC) {
            self.write_str("\x1b[23m");
        }
        if old.contains(Attr::UNDERLINE) && !attr.contains(Attr::UNDERLINE) {
            self.write_str("\x1b[24m");
        }
        if old.contains(Attr::REVERSE) && !attr.contains(Attr::REVERSE) {
            self.write_str("\x1b[27m");
        }
        if old.contains(Attr::STRIKE) && !attr.contains(Attr::STRIKE) {
            self.write_str("\x1b[29m");
        }

        // Set attributes that are newly needed
        if !old.contains(Attr::BOLD) && attr.contains(Attr::BOLD) {
            self.write_str("\x1b[1m");
        }
        if !old.contains(Attr::ITALIC) && attr.contains(Attr::ITALIC) {
            self.write_str("\x1b[3m");
        }
        if !old.contains(Attr::UNDERLINE) && attr.contains(Attr::UNDERLINE) {
            self.write_str("\x1b[4m");
        }
        if !old.contains(Attr::REVERSE) && attr.contains(Attr::REVERSE) {
            self.write_str("\x1b[7m");
        }
        if !old.contains(Attr::STRIKE) && attr.contains(Attr::STRIKE) {
            self.write_str("\x1b[9m");
        }

        self.current_attr = attr;
    }

    /// Write a styled span
    fn write_span(&mut self, span: &Span, row: usize) {
        // Move to the span's starting position
        self.move_to(span.start_col, row);

        // Apply style
        self.set_fg(span.fg);
        self.set_bg(span.bg);
        self.set_attr(span.attr);

        // Write the text
        self.write_str(&span.text);

        // Update cursor position
        self.cursor_x = span.end_col;
    }

    /// Diff two surfaces and generate optimal update sequences
    pub fn diff(&mut self, old: &GraphemeSurface, new: &GraphemeSurface) {
        self.clear();

        let (width, height) = new.dims();
        let (old_width, old_height) = old.dims();

        // Handle size changes
        if width != old_width || height != old_height {
            // Clear and redraw everything on resize
            self.write_str("\x1b[2J"); // Clear screen
            self.move_to(0, 0);

            for row in 0..height {
                let spans = new.to_row_spans(row);
                for span in &spans {
                    self.write_span(span, row);
                }
            }
            return;
        }

        // Row-by-row diff
        for row in 0..height {
            let old_spans = old.to_row_spans(row);
            let new_spans = new.to_row_spans(row);

            if old_spans != new_spans {
                // Row changed, output new spans
                for span in &new_spans {
                    self.write_span(span, row);
                }
            }
        }
    }
}

/// Compute statistics about a diff for debugging/optimization
pub struct DiffStats {
    /// Total bytes written to output
    pub bytes_written: usize,
    /// Number of text spans written
    pub spans_written: usize,
    /// Number of rows that changed
    pub rows_changed: usize,
    /// Number of style change operations
    pub style_changes: usize,
}

impl SpanDiffWriter {
    /// Generate diff with statistics
    pub fn diff_with_stats(&mut self, old: &GraphemeSurface, new: &GraphemeSurface) -> DiffStats {
        self.clear();

        let mut stats = DiffStats {
            bytes_written: 0,
            spans_written: 0,
            rows_changed: 0,
            style_changes: 0,
        };

        let (_width, height) = new.dims();

        for row in 0..height {
            let old_spans = old.to_row_spans(row);
            let new_spans = new.to_row_spans(row);

            if old_spans != new_spans {
                stats.rows_changed += 1;

                for span in &new_spans {
                    let prev_fg = self.current_fg;
                    let prev_bg = self.current_bg;
                    let prev_attr = self.current_attr;

                    self.write_span(span, row);
                    stats.spans_written += 1;

                    // Count style changes
                    if prev_fg != self.current_fg {
                        stats.style_changes += 1;
                    }
                    if prev_bg != self.current_bg {
                        stats.style_changes += 1;
                    }
                    if prev_attr != self.current_attr {
                        stats.style_changes += 1;
                    }
                }
            }
        }

        stats.bytes_written = self.output.len();
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_diff_no_change() {
        let mut writer = SpanDiffWriter::new();
        let surface = GraphemeSurface::new(10, 5);

        writer.diff(&surface, &surface);
        assert_eq!(writer.output().len(), 0, "No output for identical surfaces");
    }

    #[test]
    fn test_span_diff_single_change() {
        let mut writer = SpanDiffWriter::new();
        let mut old = GraphemeSurface::new(10, 1);
        let mut new = GraphemeSurface::new(10, 1);

        let fg = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let bg = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        old.write_str(0, 0, "AAAA", fg, bg, Attr::empty());
        new.write_str(0, 0, "BBBB", fg, bg, Attr::empty());

        writer.diff(&old, &new);

        let output = String::from_utf8_lossy(writer.output());
        assert!(
            output.contains("BBBB"),
            "Output should contain new text: {}",
            output
        );
        // The cursor positioning format is row;col with 1-based indexing
        assert!(
            output.contains("\x1b["),
            "Should contain escape sequence: {}",
            output
        );
        assert!(
            output.contains("H"),
            "Should contain H cursor command: {}",
            output
        );
    }

    #[test]
    fn test_span_diff_with_stats() {
        let mut writer = SpanDiffWriter::new();
        let old = GraphemeSurface::new(10, 2);
        let mut new = GraphemeSurface::new(10, 2);

        let fg = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let bg = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        new.write_str(0, 0, "Hello", fg, bg, Attr::empty());
        new.write_str(0, 1, "World", fg, bg, Attr::BOLD);

        let stats = writer.diff_with_stats(&old, &new);

        assert_eq!(stats.rows_changed, 2);
        // Each row will have at least one span for the text, plus possibly spacer spans
        assert!(stats.spans_written >= 2);
        assert!(stats.bytes_written > 0);
    }

    #[test]
    fn test_style_coalescing() {
        let mut surface = GraphemeSurface::new(20, 1);
        let fg = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let bg = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        // Write text with same style - adjacent characters should coalesce
        // Note: write_str already writes characters adjacently
        surface.write_str(0, 0, "HelloWorld", fg, bg, Attr::empty());

        let spans = surface.to_row_spans(0);

        // Debug output to understand what spans are being generated
        for (i, span) in spans.iter().enumerate() {
            eprintln!(
                "Span {}: start={}, end={}, text='{}', len={}",
                i,
                span.start_col,
                span.end_col,
                span.text,
                span.text.len()
            );
        }

        // The surface will have one text span and potentially a spacer span for the rest of the row
        let text_spans: Vec<_> = spans.iter().filter(|s| !s.text.trim().is_empty()).collect();

        assert_eq!(
            text_spans.len(),
            1,
            "Should have exactly one non-empty span"
        );
        assert_eq!(text_spans[0].text, "HelloWorld");
    }
}
