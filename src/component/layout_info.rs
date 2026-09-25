//! Presented component geometry, independent of the rectangular hit-test index.

use crate::event::hit::Bounds;

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
        let [a, b, c, d, tx, ty] = self.transform;
        let determinant = a * d - b * c;
        if !determinant.is_finite() || determinant.abs() < f32::EPSILON {
            return None;
        }
        let (x, y) = (x - tx, y - ty);
        let (x, y) = (
            ((d * x - c * y) / determinant).round(),
            ((-b * x + a * y) / determinant).round(),
        );
        (x >= 0.0 && y >= 0.0 && x < self.size.0 && y < self.size.1).then_some((x as u16, y as u16))
    }
}
