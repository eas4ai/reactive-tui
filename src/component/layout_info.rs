//! Presented component geometry, independent of the rectangular hit-test index.

use crate::event::hit::Bounds;

/// What the backend knows of the terminal that shows the frame, as it
/// learned it at startup and again on every resize: whether the terminal
/// takes pictures, and how many pixels a cell measures. A component that
/// can draw pixels reads it from its layout before its first picture
/// (CHT-037); a backend that paints into memory reports none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct TerminalInfo {
    /// The pixels of one cell, where the backend knows them.
    pub cell_pixels: Option<(u16, u16)>,
    /// Whether the terminal takes Kitty graphics.
    pub kitty_graphics: bool,
    /// Whether Kitty graphics may travel through shared memory.
    pub kitty_shared_memory: bool,
    /// Whether the terminal takes Sixel.
    pub sixel: bool,
}

impl TerminalInfo {
    /// A terminal of `cell_pixels` cells that takes the pictures named.
    pub fn new(
        cell_pixels: (u16, u16),
        kitty_graphics: bool,
        kitty_shared_memory: bool,
        sixel: bool,
    ) -> Self {
        Self {
            cell_pixels: Some(cell_pixels),
            kitty_graphics,
            kitty_shared_memory,
            sixel,
        }
    }

    /// Whether the terminal takes any pixel picture.
    pub fn takes_pixels(&self) -> bool {
        self.kitty_graphics || self.sixel
    }
}

/// Layout and placement of a component root in the last presented frame.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct LayoutInfo {
    /// Untransformed width and height allocated by layout, in terminal cells.
    pub size: (f32, f32),
    /// Unclipped content extent, including children that overflow the box.
    pub content_extent: (f32, f32),
    /// Ancestor and viewport clipping rectangle in screen coordinates.
    pub clip: Bounds,
    /// Maps local cell centers to screen coordinates: [a, b, c, d, x, y].
    pub transform: [f32; 6],
    /// Resolved padding and border, in left/top/right/bottom order.
    pub insets: [f32; 4],
    /// What the backend knows of the terminal showing the frame.
    pub terminal: TerminalInfo,
}

impl LayoutInfo {
    /// Geometry for an axis-aligned cell rectangle.
    pub fn from_bounds(bounds: Bounds) -> Self {
        Self {
            size: (bounds.width, bounds.height),
            content_extent: (bounds.width, bounds.height),
            clip: bounds,
            transform: [1.0, 0.0, 0.0, 1.0, bounds.x, bounds.y],
            insets: [0.0; 4],
            terminal: TerminalInfo::default(),
        }
    }

    /// Width and height available to a component's rendered content.
    pub fn content_size(&self) -> (f32, f32) {
        (
            (self.size.0 - self.insets[0] - self.insets[2]).max(0.0),
            (self.size.1 - self.insets[1] - self.insets[3]).max(0.0),
        )
    }

    /// Convert a screen cell center to local coordinates. Collapsed transforms
    /// and points outside the allocated box have no local cell.
    pub fn local_cell(&self, x: f32, y: f32) -> Option<(u16, u16)> {
        let (x, y) = self.local_point(x, y)?;
        (x >= 0.0 && y >= 0.0 && x < self.size.0 && y < self.size.1).then_some((x as u16, y as u16))
    }

    /// The local cell nearest a screen cell, also outside the allocated box,
    /// for the drags and release that stay with a pressed component (INP-003).
    /// Collapsed transforms and empty boxes have none.
    pub(crate) fn nearest_local_cell(&self, x: f32, y: f32) -> Option<(u16, u16)> {
        let (x, y) = self.local_point(x, y)?;
        let (last_x, last_y) = (self.size.0.ceil() - 1.0, self.size.1.ceil() - 1.0);
        (last_x >= 0.0 && last_y >= 0.0)
            .then(|| (x.clamp(0.0, last_x) as u16, y.clamp(0.0, last_y) as u16))
    }

    /// A screen cell center in local coordinates, rounded to a cell.
    fn local_point(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        let [a, b, c, d, tx, ty] = self.transform;
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant.abs() < f32::EPSILON {
            return None;
        }
        let (x, y) = (x - tx, y - ty);
        Some((
            ((d * x - c * y) / determinant).round(),
            ((-b * x + a * y) / determinant).round(),
        ))
    }
}
