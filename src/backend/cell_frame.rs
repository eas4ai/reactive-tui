//! Owned cell frames for terminal screens and other direct cell producers.

use crate::error::{ReactiveError, Result};
use suprtui::{ansi, buffer::OptimizedBuffer};
use unicode_segmentation::UnicodeSegmentation;

pub use suprtui::ansi::{CellDecoration, UnderlineStyle};

/// One owned grapheme. Width zero marks the tail of the previous wide cell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameCell {
    /// One grapheme, or an empty string for a continuation cell.
    pub text: String,
    /// Native cell occupancy: one, two, or zero for a wide tail.
    pub width: u8,
    /// Resolved foreground RGB.
    pub foreground: [u8; 3],
    /// Resolved background RGB.
    pub background: [u8; 3],
    /// Bold, dim, italic, underline, blink, inverse, hidden, strikethrough.
    pub attributes: u8,
    /// Overline and extended underline style/color.
    pub decoration: CellDecoration,
}

/// A complete, validated screen. Fields are private so validation survives sharing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellFrame {
    width: u16,
    height: u16,
    cells: Vec<FrameCell>,
    cursor: Option<(u16, u16)>,
}

impl CellFrame {
    /// Maximum allocated screen area.
    pub const MAX_CELLS: usize = 262_144;
    /// Maximum UTF-8 bytes retained for one displayed grapheme.
    pub const MAX_GRAPHEME_BYTES: usize = 1024;

    /// Validate dimensions, cursor, printable graphemes and wide-cell occupancy.
    pub fn new(
        width: u16,
        height: u16,
        cells: Vec<FrameCell>,
        cursor: Option<(u16, u16)>,
    ) -> Result<Self> {
        Self::validate_size(width, height)?;
        if cells.len() != usize::from(width) * usize::from(height) {
            return Err(ReactiveError::invalid_parameter(
                "cell frame length does not match its dimensions",
            ));
        }
        if cursor.is_some_and(|(x, y)| x >= width || y >= height) {
            return Err(ReactiveError::invalid_parameter(
                "cell frame cursor is outside the screen",
            ));
        }
        for (index, cell) in cells.iter().enumerate() {
            let x = index % usize::from(width);
            let valid = match cell.width {
                0 => x > 0 && cells[index - 1].width == 2 && cell.text.is_empty(),
                1 | 2 => {
                    !cell.text.is_empty()
                        && cell.text.len() <= Self::MAX_GRAPHEME_BYTES
                        && !cell.text.chars().any(char::is_control)
                        && cell.text.graphemes(true).count() == 1
                        && (cell.width == 1
                            || (x + 1 < usize::from(width) && cells[index + 1].width == 0))
                }
                _ => false,
            };
            if !valid {
                return Err(ReactiveError::invalid_parameter(format!(
                    "invalid cell frame grapheme or occupancy at cell {index}"
                )));
            }
        }
        Ok(Self {
            width,
            height,
            cells,
            cursor,
        })
    }

    pub(crate) fn validate_size(width: u16, height: u16) -> Result<()> {
        if width == 0 || height == 0 || usize::from(width) * usize::from(height) > Self::MAX_CELLS {
            return Err(ReactiveError::invalid_parameter(
                "cell frame dimensions must be nonzero with at most 262144 cells",
            ));
        }
        Ok(())
    }

    /// Screen columns and rows.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    /// Row-major cells, including wide continuation cells.
    pub fn cells(&self) -> &[FrameCell] {
        &self.cells
    }
    /// Visible cursor position, or None when hidden.
    pub fn cursor(&self) -> Option<(u16, u16)> {
        self.cursor
    }
    /// Read one cell, returning None outside the screen.
    pub fn cell(&self, x: u16, y: u16) -> Option<&FrameCell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.cells
            .get(usize::from(y) * usize::from(self.width) + usize::from(x))
    }

    pub(crate) fn paint(&self, target: &mut OptimizedBuffer<'_>) -> Result<()> {
        if (target.width(), target.height()) != (u32::from(self.width), u32::from(self.height)) {
            return Err(ReactiveError::invalid_state(
                "cell frame and backend dimensions differ",
            ));
        }
        target.clear(ansi::rgb_color(0, 0, 0, 255), None);
        for (index, cell) in self.cells.iter().enumerate() {
            if cell.width == 0 {
                continue;
            }
            let x = (index % usize::from(self.width)) as u32;
            let y = (index / usize::from(self.width)) as u32;
            let [r, g, b] = cell.foreground;
            let fg = ansi::rgb_color(r, g, b, 255);
            let [r, g, b] = cell.background;
            let bg = ansi::rgb_color(r, g, b, 255);
            target
                .draw_grapheme(
                    cell.text.as_bytes(),
                    cell.width,
                    x,
                    y,
                    fg,
                    bg,
                    u32::from(cell.attributes),
                )
                .map_err(|error| ReactiveError::resource(format!("cell frame paint: {error:?}")))?;
            if let Some(painted) = target.get(x, y) {
                target.set(x, y, painted.with_decoration(cell.decoration));
            }
        }
        Ok(())
    }
}

/// RAS-007: painting a cell frame clears the next buffer exactly once.
#[cfg(test)]
mod ras_007_tests {
    use super::*;
    use ::suprtui::{buffer::InitOptions, uni::pool::GraphemePool};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn ras_007_painting_a_cell_frame_clears_the_next_buffer_once() {
        let cell = |text: &str| FrameCell {
            text: text.into(),
            width: 1,
            foreground: [200, 200, 200],
            background: [0, 0, 40],
            attributes: 0,
            decoration: CellDecoration::default(),
        };
        let frame = CellFrame::new(2, 1, vec![cell("a"), cell("b")], None).unwrap();
        let pool = Rc::new(RefCell::new(GraphemePool::new()));
        let mut buffer = OptimizedBuffer::new(2, 1, InitOptions::new(pool)).unwrap();
        for painted in 1..=2 {
            frame.paint(&mut buffer).unwrap();
            assert_eq!(buffer.clear_count(), painted);
        }
    }
}
