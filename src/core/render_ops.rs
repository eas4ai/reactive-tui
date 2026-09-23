//! RenderOps Pipeline - Decouples rendering commands from terminal output
//!
//! This module provides an intermediate representation for rendering operations,
//! allowing for better testability, optimization, and backend flexibility.

use super::surface::{Attr, Rgba};

/// A single rendering operation
#[derive(Debug, Clone, PartialEq)]
pub enum RenderOp {
    /// Move cursor to absolute position (0-based)
    MoveTo {
        /// X coordinate (column)
        x: u16,
        /// Y coordinate (row)
        y: u16,
    },

    /// Set foreground color
    SetFgColor(Rgba),

    /// Set background color
    SetBgColor(Rgba),

    /// Set text attributes (bold, italic, etc.)
    SetAttributes(Attr),

    /// Reset all styles to default
    ResetStyle,

    /// Print a run of text at current cursor position
    PrintRun(String),

    /// Clear a rectangular area with optional background color
    ClearArea {
        /// X coordinate of top-left corner
        x: u16,
        /// Y coordinate of top-left corner
        y: u16,
        /// Width of the area to clear
        width: u16,
        /// Height of the area to clear
        height: u16,
        /// Optional background color to fill with
        bg: Option<Rgba>,
    },

    /// Clear entire screen
    ClearScreen,

    /// Clear from cursor to end of line
    ClearToEndOfLine,

    /// Show/hide cursor
    SetCursorVisible(bool),

    /// Enable/disable synchronized output (reduces flicker)
    SetSynchronizedOutput(bool),

    /// Save current cursor position
    SaveCursorPosition,

    /// Restore previously saved cursor position
    RestoreCursorPosition,
}

/// A collection of render operations for a frame
#[derive(Debug, Default)]
pub struct RenderOps {
    ops: Vec<RenderOp>,
}

impl RenderOps {
    /// Create a new empty RenderOps collection
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    /// Create with estimated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            ops: Vec::with_capacity(capacity),
        }
    }

    /// Add a render operation
    pub fn push(&mut self, op: RenderOp) {
        self.ops.push(op);
    }

    /// Add multiple operations
    pub fn extend<I>(&mut self, ops: I)
    where
        I: IntoIterator<Item = RenderOp>,
    {
        self.ops.extend(ops);
    }

    /// Get the operations
    pub fn ops(&self) -> &[RenderOp] {
        &self.ops
    }

    /// Take ownership of the operations
    pub fn into_ops(self) -> Vec<RenderOp> {
        self.ops
    }

    /// Clear all operations
    pub fn clear(&mut self) {
        self.ops.clear();
    }

    /// Get the number of operations
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// Builder for generating RenderOps from various sources
pub struct RenderOpsBuilder {
    pub(crate) ops: RenderOps,
    current_fg: Option<Rgba>,
    current_bg: Option<Rgba>,
    current_attr: Attr,
}

impl RenderOpsBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            ops: RenderOps::with_capacity(1024),
            current_fg: None,
            current_bg: None,
            current_attr: Attr::empty(),
        }
    }

    /// Move to a position
    pub fn move_to(&mut self, x: u16, y: u16) -> &mut Self {
        self.ops.push(RenderOp::MoveTo { x, y });
        self
    }

    /// Set foreground color if different from current
    pub fn set_fg(&mut self, color: Rgba) -> &mut Self {
        if self.current_fg != Some(color) {
            self.ops.push(RenderOp::SetFgColor(color));
            self.current_fg = Some(color);
        }
        self
    }

    /// Set background color if different from current
    pub fn set_bg(&mut self, color: Rgba) -> &mut Self {
        if self.current_bg != Some(color) {
            self.ops.push(RenderOp::SetBgColor(color));
            self.current_bg = Some(color);
        }
        self
    }

    /// Set text attributes if different from current
    pub fn set_attr(&mut self, attr: Attr) -> &mut Self {
        if self.current_attr != attr {
            self.ops.push(RenderOp::SetAttributes(attr));
            self.current_attr = attr;
        }
        self
    }

    /// Print text at current position
    pub fn print(&mut self, text: impl Into<String>) -> &mut Self {
        self.ops.push(RenderOp::PrintRun(text.into()));
        self
    }

    /// Print styled text (sets style then prints)
    pub fn print_styled(
        &mut self,
        text: impl Into<String>,
        fg: Rgba,
        bg: Rgba,
        attr: Attr,
    ) -> &mut Self {
        self.set_fg(fg).set_bg(bg).set_attr(attr).print(text)
    }

    /// Clear an area
    pub fn clear_area(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        bg: Option<Rgba>,
    ) -> &mut Self {
        self.ops.push(RenderOp::ClearArea {
            x,
            y,
            width,
            height,
            bg,
        });
        self
    }

    /// Clear the screen
    pub fn clear_screen(&mut self) -> &mut Self {
        self.ops.push(RenderOp::ClearScreen);
        self.current_fg = None;
        self.current_bg = None;
        self.current_attr = Attr::empty();
        self
    }

    /// Reset all styles
    pub fn reset_style(&mut self) -> &mut Self {
        self.ops.push(RenderOp::ResetStyle);
        self.current_fg = None;
        self.current_bg = None;
        self.current_attr = Attr::empty();
        self
    }

    /// Build the final RenderOps
    pub fn build(self) -> RenderOps {
        self.ops
    }
}

impl Default for RenderOpsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert spans to RenderOps
pub fn spans_to_render_ops(spans: &[crate::core::grapheme_cell::Span]) -> RenderOps {
    let mut builder = RenderOpsBuilder::new();

    for span in spans {
        builder
            .move_to(span.start_col as u16, 0)
            .print_styled(&span.text, span.fg, span.bg, span.attr);
    }

    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_ops_builder() {
        let mut builder = RenderOpsBuilder::new();
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

        builder.move_to(10, 5).set_fg(fg).set_bg(bg).print("Hello");

        let ops = builder.build();
        assert_eq!(ops.len(), 4);
        assert_eq!(ops.ops()[0], RenderOp::MoveTo { x: 10, y: 5 });
        assert_eq!(ops.ops()[1], RenderOp::SetFgColor(fg));
        assert_eq!(ops.ops()[2], RenderOp::SetBgColor(bg));
        assert_eq!(ops.ops()[3], RenderOp::PrintRun("Hello".to_string()));
    }

    #[test]
    fn test_style_deduplication() {
        let mut builder = RenderOpsBuilder::new();
        let fg = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        builder.set_fg(fg).set_fg(fg).set_fg(fg);

        let ops = builder.build();
        assert_eq!(ops.len(), 1, "Should only set color once");
    }

    #[test]
    fn test_print_styled() {
        let mut builder = RenderOpsBuilder::new();
        let fg = Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        };
        let bg = Rgba {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };

        builder.print_styled("Test", fg, bg, Attr::BOLD);

        let ops = builder.build();
        assert_eq!(ops.len(), 4);
        assert!(matches!(ops.ops()[0], RenderOp::SetFgColor(_)));
        assert!(matches!(ops.ops()[1], RenderOp::SetBgColor(_)));
        assert!(matches!(ops.ops()[2], RenderOp::SetAttributes(_)));
        assert!(matches!(ops.ops()[3], RenderOp::PrintRun(_)));
    }
}
