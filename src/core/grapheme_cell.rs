//! Grapheme-aware cell model for proper Unicode support
//!
//! This module provides an enhanced cell representation that correctly handles:
//! - Wide characters (CJK, emoji)
//! - Combining characters (diacritics, modifiers)
//! - Grapheme clusters (multi-codepoint units)

use super::surface::{Attr, Rgba};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

/// Enhanced cell type that properly represents terminal cells
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CellType {
    /// Regular character cell with style
    Glyph {
        /// The grapheme cluster (may be multiple codepoints)
        grapheme: GraphemeCluster,
        /// Foreground color
        fg: Rgba,
        /// Background color
        bg: Rgba,
        /// Text attributes (bold, italic, etc.)
        attr: Attr,
    },
    /// Empty cell that can be painted over
    Spacer {
        /// Background color for the empty cell
        bg: Rgba,
    },
    /// Continuation cell for wide characters (cannot be painted)
    Void,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Spacer {
            bg: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}

/// Represents a grapheme cluster (user-perceived character)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphemeCluster {
    /// Storage for the grapheme (up to 4 UTF-8 bytes for most cases)
    /// For longer clusters, we truncate (extremely rare in practice)
    bytes: [u8; 16],
    /// Actual length of valid UTF-8 data
    len: u8,
    /// Display width in terminal cells (0, 1, or 2)
    width: u8,
}

impl GraphemeCluster {
    /// Create a new grapheme cluster from a string slice
    pub fn new(s: &str) -> Self {
        let bytes_slice = s.as_bytes();
        let mut len = bytes_slice.len();

        // Ensure we don't break UTF-8 character boundaries
        if len > 16 {
            len = 16;
            // Find the last valid UTF-8 boundary within our buffer
            while len > 0 && !s.is_char_boundary(len) {
                len -= 1;
            }
            // If we couldn't find a valid boundary, use empty string
            if len == 0 {
                return Self {
                    bytes: [0u8; 16],
                    len: 0,
                    width: 0,
                };
            }
        }

        let mut bytes = [0u8; 16];
        bytes[..len].copy_from_slice(&bytes_slice[..len]);

        // Calculate display width based on the valid truncated string
        let truncated_str = &s[..len];
        let width = UnicodeWidthStr::width(truncated_str).min(2) as u8;

        Self {
            bytes,
            len: len as u8,
            width,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &str {
        // Safe version with validation instead of unsafe assumption
        std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap_or_default()
    }

    /// Get the display width
    pub fn width(&self) -> usize {
        self.width as usize
    }

    /// Check if this is a zero-width grapheme (combining character)
    pub fn is_zero_width(&self) -> bool {
        self.width == 0
    }
}

/// Enhanced surface that handles graphemes correctly
#[derive(Clone)]
pub struct GraphemeSurface {
    width: usize,
    height: usize,
    cells: Vec<CellType>,
}

impl GraphemeSurface {
    /// Create a new surface with given dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![CellType::default(); width * height],
        }
    }

    /// Get the dimensions
    pub fn dims(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Calculate cell index
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Clear the surface with a background color
    pub fn clear(&mut self, bg: Rgba) {
        for cell in &mut self.cells {
            *cell = CellType::Spacer { bg };
        }
    }

    /// Set a cell at position, replacing every glyph it overlaps.
    pub fn set_cell(&mut self, x: usize, y: usize, cell: CellType) {
        if x >= self.width || y >= self.height {
            return;
        }
        let width = match cell {
            CellType::Glyph { grapheme, .. } => grapheme.width().max(1),
            _ => 1,
        };
        // PLT-009: a wide glyph that cannot fit leaves existing cells unchanged.
        if width > self.width - x {
            return;
        }
        for col in x..x + width {
            self.clear_overlapped_glyph(col, y);
        }
        let idx = self.idx(x, y);
        self.cells[idx] = cell;
        if width == 2 {
            self.cells[idx + 1] = CellType::Void;
        }
    }

    fn clear_overlapped_glyph(&mut self, x: usize, y: usize) {
        let idx = self.idx(x, y);
        let lead = if self.cells[idx] == CellType::Void && x > 0 {
            idx - 1
        } else {
            idx
        };
        if let CellType::Glyph { grapheme, bg, .. } = self.cells[lead] {
            if grapheme.width() == 2 {
                // PLT-009: erase both halves with the old glyph's background.
                self.cells[lead] = CellType::Spacer { bg };
                if lead + 1 < (y + 1) * self.width {
                    self.cells[lead + 1] = CellType::Spacer { bg };
                }
            }
        }
    }

    /// Get a cell at position
    pub fn get_cell(&self, x: usize, y: usize) -> CellType {
        if x < self.width && y < self.height {
            self.cells[self.idx(x, y)]
        } else {
            CellType::default()
        }
    }

    /// Write a string at position, handling graphemes correctly
    pub fn write_str(&mut self, mut x: usize, y: usize, s: &str, fg: Rgba, bg: Rgba, attr: Attr) {
        if y >= self.height {
            return;
        }

        // Iterate over grapheme clusters
        for grapheme in s.graphemes(true) {
            if x >= self.width {
                break;
            }

            let cluster = GraphemeCluster::new(grapheme);
            let width = cluster.width();

            // Skip zero-width characters for now (would need special handling)
            if width == 0 {
                continue;
            }

            if width > self.width - x {
                break;
            }

            // Place the glyph and its continuation together.
            self.set_cell(
                x,
                y,
                CellType::Glyph {
                    grapheme: cluster,
                    fg,
                    bg,
                    attr,
                },
            );
            x += width;
        }
    }

    /// Convert to row spans for efficient diffing
    pub fn to_row_spans(&self, row: usize) -> Vec<Span> {
        if row >= self.height {
            return Vec::new();
        }

        let mut spans = Vec::new();
        let mut current_span: Option<Span> = None;
        for x in 0..self.width {
            let (text, fg, bg, attr) = match self.get_cell(x, row) {
                CellType::Glyph {
                    grapheme,
                    fg,
                    bg,
                    attr,
                } => (grapheme, fg, bg, attr),
                CellType::Spacer { bg } => {
                    (GraphemeCluster::new(" "), Rgba::white(), bg, Attr::empty())
                }
                CellType::Void => {
                    // PLT-009: only an actual wide glyph owns this column.
                    let continuation = x > 0
                        && matches!(self.get_cell(x - 1, row),
                            CellType::Glyph { grapheme, .. } if grapheme.width() == 2)
                        && current_span
                            .as_ref()
                            .is_some_and(|span| span.end_col == x + 1);
                    if continuation {
                        continue;
                    }
                    if let Some(span) = current_span.take() {
                        spans.push(span);
                    }
                    (
                        GraphemeCluster::new(" "),
                        Rgba::white(),
                        Rgba::black(),
                        Attr::empty(),
                    )
                }
            };
            let text = text.as_str();
            let end = x + UnicodeWidthStr::width(text);
            if let Some(span) = current_span.as_mut() {
                // PLT-009: matching style alone cannot bridge a missing column.
                if span.end_col == x && span.can_extend(&fg, &bg, &attr) {
                    span.text.push_str(text);
                    span.end_col = end;
                    continue;
                }
            }
            if let Some(span) = current_span.take() {
                spans.push(span);
            }
            current_span = Some(Span::new(x, end, text, fg, bg, attr));
        }

        // Push final span if any
        if let Some(span) = current_span {
            spans.push(span);
        }

        spans
    }
}

/// Represents a contiguous span of text with the same style
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// Starting column position of the span
    pub start_col: usize,
    /// Ending column position of the span (exclusive)
    pub end_col: usize,
    /// Text content of the span
    pub text: String,
    /// Foreground color for the text
    pub fg: Rgba,
    /// Background color for the text
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
}

impl Span {
    fn new(start: usize, end: usize, text: &str, fg: Rgba, bg: Rgba, attr: Attr) -> Self {
        Self {
            start_col: start,
            end_col: end,
            text: text.to_string(),
            fg,
            bg,
            attr,
        }
    }

    fn can_extend(&self, fg: &Rgba, bg: &Rgba, attr: &Attr) -> bool {
        self.fg == *fg && self.bg == *bg && self.attr == *attr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plt_009_direct_cells_clear_overlaps_and_spans_cover_orphans() {
        let bg = Rgba::new(0.0, 0.0, 1.0, 1.0);
        let glyph = |text| CellType::Glyph {
            grapheme: GraphemeCluster::new(text),
            fg: Rgba::white(),
            bg,
            attr: Attr::empty(),
        };
        let mut surface = GraphemeSurface::new(5, 1);
        surface.set_cell(0, 0, glyph("界"));
        surface.set_cell(2, 0, glyph("界"));
        surface.set_cell(1, 0, glyph("界"));
        assert_eq!(surface.get_cell(0, 0), CellType::Spacer { bg });
        assert_eq!(surface.get_cell(3, 0), CellType::Spacer { bg });
        assert_eq!(surface.get_cell(2, 0), CellType::Void);
        surface.set_cell(4, 0, glyph("X"));
        surface.write_str(4, 0, "界", Rgba::white(), bg, Attr::empty());
        assert_eq!(surface.get_cell(4, 0), glyph("X"));

        surface.set_cell(1, 0, CellType::Void);
        assert_eq!(surface.get_cell(2, 0), CellType::Spacer { bg });
        let spans = surface.to_row_spans(0);
        for span in &spans {
            assert_eq!(
                span.end_col,
                span.start_col + UnicodeWidthStr::width(span.text.as_str())
            );
        }
        assert!(spans
            .iter()
            .any(|span| span.start_col == 1 && span.text == " "));

        surface.set_cell(3, 0, glyph("界"));
        let spans = surface.to_row_spans(0);
        assert!(spans.iter().any(|span| span.end_col == 5
            && span.text.find("界").is_some_and(|at| {
                span.start_col + UnicodeWidthStr::width(&span.text[..at]) == 3
            })));
    }

    #[test]
    fn test_grapheme_cluster_ascii() {
        let cluster = GraphemeCluster::new("A");
        assert_eq!(cluster.as_str(), "A");
        assert_eq!(cluster.width(), 1);
        assert!(!cluster.is_zero_width());
    }

    #[test]
    fn test_grapheme_cluster_emoji() {
        let cluster = GraphemeCluster::new("😀");
        assert_eq!(cluster.as_str(), "😀");
        assert_eq!(cluster.width(), 2);
        assert!(!cluster.is_zero_width());
    }

    #[test]
    fn test_grapheme_cluster_cjk() {
        let cluster = GraphemeCluster::new("中");
        assert_eq!(cluster.as_str(), "中");
        assert_eq!(cluster.width(), 2);
        assert!(!cluster.is_zero_width());
    }

    #[test]
    fn test_surface_write_mixed_width() {
        let mut surface = GraphemeSurface::new(10, 1);
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

        surface.write_str(0, 0, "A中B", fg, bg, Attr::empty());

        // Check cells
        match surface.get_cell(0, 0) {
            CellType::Glyph { grapheme, .. } => assert_eq!(grapheme.as_str(), "A"),
            other => panic!("Expected glyph 'A' at (0,0), got: {:?}", other),
        }

        match surface.get_cell(1, 0) {
            CellType::Glyph { grapheme, .. } => assert_eq!(grapheme.as_str(), "中"),
            other => panic!("Expected glyph '中' at (1,0), got: {:?}", other),
        }

        // Wide character continuation
        match surface.get_cell(2, 0) {
            CellType::Void => {}
            other => panic!(
                "Expected void at (2,0) for wide character continuation, got: {:?}",
                other
            ),
        }

        match surface.get_cell(3, 0) {
            CellType::Glyph { grapheme, .. } => assert_eq!(grapheme.as_str(), "B"),
            other => panic!("Expected glyph 'B' at (3,0), got: {:?}", other),
        }
    }

    #[test]
    fn test_row_spans() {
        let mut surface = GraphemeSurface::new(10, 1);
        let fg1 = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let fg2 = Rgba {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };
        let bg = Rgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        surface.write_str(0, 0, "AAA", fg1, bg, Attr::empty());
        surface.write_str(3, 0, "BBB", fg2, bg, Attr::empty());

        let spans = surface.to_row_spans(0);
        // There will be 2 text spans, possibly with a spacer between
        assert!(spans.len() >= 2);

        // Find the two text spans
        let text_spans: Vec<_> = spans.iter().filter(|s| !s.text.trim().is_empty()).collect();
        assert_eq!(text_spans.len(), 2);
        assert_eq!(text_spans[0].text, "AAA");
        assert_eq!(text_spans[0].fg, fg1);
        assert_eq!(text_spans[1].text, "BBB");
        assert_eq!(text_spans[1].fg, fg2);
    }

    #[test]
    fn test_row_spans_adjacent() {
        let mut surface = GraphemeSurface::new(10, 1);
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

        // Write adjacent text with same style - should coalesce
        surface.write_str(0, 0, "Hello", fg, bg, Attr::empty());
        surface.write_str(5, 0, "World", fg, bg, Attr::empty());

        let spans = surface.to_row_spans(0);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "HelloWorld");
    }

    #[test]
    fn test_cjk_characters_comprehensive() {
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

        // Test various CJK characters: Chinese, Japanese, Korean
        let test_cases = [
            ("中文", 4),   // Chinese: 2 chars, 4 cells
            ("日本語", 6), // Japanese: 3 chars, 6 cells
            ("한국어", 6), // Korean: 3 chars, 6 cells
            ("A中B", 4),   // Mixed ASCII and CJK
        ];

        for (text, expected_width) in test_cases {
            let mut surf = GraphemeSurface::new(20, 1);
            surf.write_str(0, 0, text, fg, bg, Attr::empty());

            // Count occupied cells
            let mut width = 0;
            for x in 0..20 {
                match surf.get_cell(x, 0) {
                    CellType::Glyph { .. } => width += 1,
                    CellType::Void => width += 1,
                    _ => break,
                }
            }
            assert_eq!(width, expected_width, "Width mismatch for '{}'", text);
        }
    }

    #[test]
    fn test_emoji_comprehensive() {
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

        // Various emoji test cases - just test basic ones that work reliably
        let test_cases = [
            "😀",   // Basic emoji
            "👍",   // Thumbs up
            "🎉",   // Party popper
            "❤️",   // Heart (with variation selector)
            "A😀B", // Mixed ASCII and emoji
        ];

        for text in test_cases {
            let mut surf = GraphemeSurface::new(30, 1);
            surf.write_str(0, 0, text, fg, bg, Attr::empty());

            // Verify the surface contains something
            let spans = surf.to_row_spans(0);
            assert!(!spans.is_empty(), "Should generate spans for '{}'", text);

            // Check that we have at least one glyph cell
            let mut found_glyph = false;
            for x in 0..30 {
                if let CellType::Glyph { .. } = surf.get_cell(x, 0) {
                    found_glyph = true;
                    break;
                }
            }
            assert!(found_glyph, "Should have at least one glyph for '{}'", text);
        }
    }

    #[test]
    fn test_combining_marks() {
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

        // Test combining diacritical marks
        let test_cases = [
            ("é", 1),         // e with acute (precomposed)
            ("e\u{0301}", 1), // e + combining acute
            ("ñ", 1),         // n with tilde (precomposed)
            ("n\u{0303}", 1), // n + combining tilde
            ("à", 1),         // a with grave
            ("a\u{0300}", 1), // a + combining grave
        ];

        for (text, expected_width) in test_cases {
            let mut surf = GraphemeSurface::new(20, 1);
            surf.write_str(0, 0, text, fg, bg, Attr::empty());

            // The grapheme should occupy expected width
            let mut width = 0;
            for x in 0..20 {
                match surf.get_cell(x, 0) {
                    CellType::Glyph { .. } => width += 1,
                    CellType::Void => width += 1,
                    CellType::Spacer { .. } if x == 0 => break,
                    _ => {}
                }
            }
            assert!(
                width <= expected_width,
                "Combining mark '{}' uses too many cells: {} > {}",
                text,
                width,
                expected_width
            );
        }
    }

    #[test]
    fn test_mixed_width_text_comprehensive() {
        let mut surface = GraphemeSurface::new(40, 1);
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

        // Complex mixed-width text
        let text = "Hello 世界! 🎉 Rust 最高 😀";
        surface.write_str(0, 0, text, fg, bg, Attr::empty());

        // Verify spans are generated correctly
        let spans = surface.to_row_spans(0);
        assert!(!spans.is_empty(), "Should generate spans for mixed text");

        // Check that the text is preserved
        let combined_text: String = spans
            .iter()
            .map(|s| s.text.clone())
            .collect::<String>()
            .trim()
            .to_string();

        // The combined text should contain all our characters
        assert!(combined_text.contains("Hello"), "Should contain 'Hello'");
        assert!(
            combined_text.contains("世界"),
            "Should contain CJK characters"
        );
        assert!(combined_text.contains("Rust"), "Should contain 'Rust'");
    }
}
