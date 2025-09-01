//! Styled text models for rich text rendering
//!
//! This module provides shared models for representing styled text that can be
//! used across syntax highlighting, markdown rendering, and editor components.

use super::render_ops::{RenderOps, RenderOpsBuilder};
use super::surface::{Attr, Rgba};

/// A run of text with consistent styling
#[derive(Debug, Clone, PartialEq)]
pub struct StyledRun {
    /// The text content
    pub text: String,
    /// Foreground color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Text attributes (bold, italic, etc.)
    pub attr: Attr,
}

impl StyledRun {
    /// Create a new styled run
    pub fn new(text: impl Into<String>, fg: Rgba, bg: Rgba, attr: Attr) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            attr,
        }
    }

    /// Create a plain text run with default styling
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            fg: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            bg: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            attr: Attr::empty(),
        }
    }

    /// Get the display width of the text (accounting for wide characters)
    pub fn width(&self) -> usize {
        use unicode_width::UnicodeWidthStr;
        self.text.width()
    }
}

/// A line of styled text composed of multiple runs
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StyledLine {
    /// The runs that make up this line
    pub runs: Vec<StyledRun>,
}

impl StyledLine {
    /// Create a new empty styled line
    pub fn new() -> Self {
        Self { runs: Vec::new() }
    }

    /// Create a styled line from a single run
    pub fn from_run(run: StyledRun) -> Self {
        Self { runs: vec![run] }
    }

    /// Create a plain text line
    pub fn plain(text: impl Into<String>) -> Self {
        Self::from_run(StyledRun::plain(text))
    }

    /// Add a run to the line
    pub fn push(&mut self, run: StyledRun) {
        // Merge with previous run if styles match
        if let Some(last) = self.runs.last_mut() {
            if last.fg == run.fg && last.bg == run.bg && last.attr == run.attr {
                last.text.push_str(&run.text);
                return;
            }
        }
        self.runs.push(run);
    }

    /// Get the total display width of the line
    pub fn width(&self) -> usize {
        self.runs.iter().map(|r| r.width()).sum()
    }

    /// Get the plain text content (without styling)
    pub fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

/// Convert a styled line to render operations
pub fn styled_line_to_render_ops(line: &StyledLine, x: u16, y: u16) -> RenderOps {
    let mut builder = RenderOpsBuilder::new();
    let mut _current_x = x;

    builder.move_to(x, y);

    for run in &line.runs {
        builder
            .set_fg(run.fg)
            .set_bg(run.bg)
            .set_attr(run.attr)
            .print(&run.text);

        _current_x += run.width() as u16;
    }

    builder.build()
}

/// Convert multiple styled lines to render operations
pub fn styled_lines_to_render_ops(lines: &[StyledLine], x: u16, y: u16) -> RenderOps {
    let mut builder = RenderOpsBuilder::new();

    for (i, line) in lines.iter().enumerate() {
        let line_y = y + i as u16;
        builder.move_to(x, line_y);

        for run in &line.runs {
            builder
                .set_fg(run.fg)
                .set_bg(run.bg)
                .set_attr(run.attr)
                .print(&run.text);
        }
    }

    builder.build()
}

/// Builder for creating styled lines
pub struct StyledLineBuilder {
    line: StyledLine,
    current_fg: Rgba,
    current_bg: Rgba,
    current_attr: Attr,
}

impl StyledLineBuilder {
    /// Create a new builder with default styling
    pub fn new() -> Self {
        Self {
            line: StyledLine::new(),
            current_fg: Rgba {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            current_bg: Rgba {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            current_attr: Attr::empty(),
        }
    }

    /// Set the current foreground color
    pub fn fg(mut self, color: Rgba) -> Self {
        self.current_fg = color;
        self
    }

    /// Set the current background color
    pub fn bg(mut self, color: Rgba) -> Self {
        self.current_bg = color;
        self
    }

    /// Set the current attributes
    pub fn attr(mut self, attr: Attr) -> Self {
        self.current_attr = attr;
        self
    }

    /// Add text with current styling
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.line.push(StyledRun::new(
            text,
            self.current_fg,
            self.current_bg,
            self.current_attr,
        ));
        self
    }

    /// Add a styled run directly
    pub fn run(mut self, run: StyledRun) -> Self {
        self.line.push(run);
        self
    }

    /// Build the final styled line
    pub fn build(self) -> StyledLine {
        self.line
    }
}

impl Default for StyledLineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::render_ops::RenderOp;

    #[test]
    fn test_styled_run_width() {
        let run = StyledRun::plain("Hello");
        assert_eq!(run.width(), 5);

        let run_cjk = StyledRun::plain("世界");
        assert_eq!(run_cjk.width(), 4); // Each CJK char is 2 cells

        let run_emoji = StyledRun::plain("😀");
        assert_eq!(run_emoji.width(), 2);
    }

    #[test]
    fn test_styled_line_merging() {
        let mut line = StyledLine::new();
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

        line.push(StyledRun::new("Hello", fg, bg, Attr::empty()));
        line.push(StyledRun::new(" ", fg, bg, Attr::empty()));
        line.push(StyledRun::new("World", fg, bg, Attr::empty()));

        assert_eq!(
            line.runs.len(),
            1,
            "Adjacent runs with same style should merge"
        );
        assert_eq!(line.text(), "Hello World");
    }

    #[test]
    fn test_styled_line_no_merging() {
        let mut line = StyledLine::new();
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

        line.push(StyledRun::new("Hello", fg1, bg, Attr::empty()));
        line.push(StyledRun::new(" ", fg2, bg, Attr::empty()));
        line.push(StyledRun::new("World", fg1, bg, Attr::empty()));

        assert_eq!(line.runs.len(), 3, "Different styles should not merge");
    }

    #[test]
    fn test_builder() {
        let fg_red = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let fg_green = Rgba {
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

        let line = StyledLineBuilder::new()
            .fg(fg_red)
            .bg(bg)
            .attr(Attr::BOLD)
            .text("Bold Red")
            .fg(fg_green)
            .attr(Attr::ITALIC)
            .text(" Italic Green")
            .build();

        assert_eq!(line.runs.len(), 2);
        assert_eq!(line.runs[0].text, "Bold Red");
        assert_eq!(line.runs[0].attr, Attr::BOLD);
        assert_eq!(line.runs[1].text, " Italic Green");
        assert_eq!(line.runs[1].attr, Attr::ITALIC);
    }

    #[test]
    fn test_to_render_ops() {
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

        let line = StyledLine::from_run(StyledRun::new("Test", fg, bg, Attr::BOLD));
        let ops = styled_line_to_render_ops(&line, 10, 5);

        assert!(ops
            .ops()
            .iter()
            .any(|op| matches!(op, RenderOp::MoveTo { x: 10, y: 5 })));
        assert!(ops
            .ops()
            .iter()
            .any(|op| matches!(op, RenderOp::SetFgColor(_))));
        assert!(ops
            .ops()
            .iter()
            .any(|op| matches!(op, RenderOp::SetBgColor(_))));
        assert!(ops
            .ops()
            .iter()
            .any(|op| matches!(op, RenderOp::SetAttributes(_))));
        assert!(ops
            .ops()
            .iter()
            .any(|op| matches!(op, RenderOp::PrintRun(s) if s == "Test")));
    }
}
