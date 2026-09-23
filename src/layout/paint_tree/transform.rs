//! Affine placement of cell centers. Glyph bitmaps stay upright.

use crate::layout::motion::CellTransform;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Affine {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    x: f32,
    y: f32,
}

impl Default for Affine {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Affine {
    pub(super) fn coefficients(self) -> [f32; 6] {
        [self.a, self.b, self.c, self.d, self.x, self.y]
    }
    /// Whether the placement only translates: no rotation, scale or skew.
    /// For such a placement `inverse` is an exact subtraction of `offset`
    /// and `point` an exact addition, which the painter's fast path uses.
    pub(super) fn is_translation(self) -> bool {
        self.a == 1.0 && self.b == 0.0 && self.c == 0.0 && self.d == 1.0
    }

    pub(super) fn offset(self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub(super) fn point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.x,
            self.b * x + self.d * y + self.y,
        )
    }
    pub(super) fn inverse(self, x: f32, y: f32) -> Option<(f32, f32)> {
        let det = self.a * self.d - self.b * self.c;
        if !det.is_finite() || det.abs() < f32::EPSILON {
            return None;
        }
        let x = x - self.x;
        let y = y - self.y;
        Some((
            (self.d * x - self.c * y) / det,
            (-self.b * x + self.a * y) / det,
        ))
    }
    pub(super) fn placed(
        self,
        location: (f32, f32),
        size: (f32, f32),
        motion: CellTransform,
    ) -> Self {
        let (sin, cos) = motion.rotation.to_radians().sin_cos();
        let sx = motion.skew_x.to_radians().tan();
        let sy = motion.skew_y.to_radians().tan();
        let ra = (cos - sin * sy) * motion.scale_x;
        let rb = (sin + cos * sy) * motion.scale_x;
        let rc = (cos * sx - sin) * motion.scale_y;
        let rd = (sin * sx + cos) * motion.scale_y;
        let [ma, mb, mc, md, mx, my] = motion.matrix;
        let a = ra * ma + rc * mb;
        let b = rb * ma + rd * mb;
        let c = ra * mc + rc * md;
        let d = rb * mc + rd * md;
        let cx = (size.0 - 1.0) / 2.0;
        let cy = (size.1 - 1.0) / 2.0;
        let (x, y) = self.point(
            location.0 + motion.x + motion.x_percent * size.0 + ra * mx + rc * my + cx
                - a * cx
                - c * cy,
            location.1 + motion.y + motion.y_percent * size.1 + rb * mx + rd * my + cy
                - b * cx
                - d * cy,
        );
        Self {
            a: self.a * a + self.c * b,
            b: self.b * a + self.d * b,
            c: self.a * c + self.c * d,
            d: self.b * c + self.d * d,
            x,
            y,
        }
    }
    pub(super) fn bounds(self, width: i32, height: i32) -> (i32, i32, i32, i32) {
        if width <= 0 || height <= 0 || self.inverse(0.0, 0.0).is_none() {
            return (0, 0, 0, 0);
        }
        let corners = [
            self.point(-0.5, -0.5),
            self.point(width as f32 - 0.5, -0.5),
            self.point(-0.5, height as f32 - 0.5),
            self.point(width as f32 - 0.5, height as f32 - 0.5),
        ];
        let min_x = corners.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let max_x = corners
            .iter()
            .map(|p| p.0)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = corners.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_y = corners
            .iter()
            .map(|p| p.1)
            .fold(f32::NEG_INFINITY, f32::max);
        (
            min_x.ceil() as i32,
            min_y.ceil() as i32,
            max_x.ceil() as i32,
            max_y.ceil() as i32,
        )
    }
}
