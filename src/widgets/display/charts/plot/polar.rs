//! Polar placement for radial charts: category spoke angles through a
//! linear scale, and the grid of concentric level polygons and spokes that
//! radar charts draw under their shapes, the way [`super::Grid`] draws a
//! cartesian grid. Angles are radians clockwise from twelve o'clock, and
//! positions are in canvas dot units, where a dot is as wide as it is tall.

use super::super::mask::{DOTS_X, DOTS_Y};
use super::{ScaleLinear, TextSink};
use std::f64::consts::TAU;

/// The angle of each of `count` categories, evenly spaced from twelve
/// o'clock, clockwise, through a linear scale from category index to angle.
pub fn spoke_angles(count: usize) -> Vec<f64> {
    let scale = ScaleLinear::new((0.0, count.max(1) as f64), (0.0, TAU));
    (0..count).map(|k| scale.map(k as f64)).collect()
}

/// The point at `angle` and `radius` from `center`.
pub fn at((cx, cy): (f64, f64), angle: f64, radius: f64) -> (f64, f64) {
    (cx + angle.sin() * radius, cy - angle.cos() * radius)
}

/// A radar grid: `levels` concentric polygons at equal steps out to
/// `radius`, with a vertex on each spoke, and one spoke line per angle.
#[derive(Debug, Clone, PartialEq)]
pub struct PolarGrid {
    /// The grid's center, in dot units.
    pub center: (f64, f64),
    /// The outermost level's radius, in dot units.
    pub radius: f64,
    /// The spoke angles.
    pub spokes: Vec<f64>,
    /// The number of level polygons; zero draws spokes only.
    pub levels: usize,
}

impl PolarGrid {
    /// Mark the cells the levels and spokes cross with `glyph`, under the
    /// shapes (drawn only where the canvas leaves a cell empty).
    pub fn draw(&self, sink: &mut impl TextSink, glyph: &str) {
        let n = self.spokes.len();
        for level in 1..=self.levels {
            let r = self.radius * level as f64 / self.levels as f64;
            for k in 0..n {
                let a = at(self.center, self.spokes[k], r);
                let b = at(self.center, self.spokes[(k + 1) % n], r);
                segment(sink, a, b, glyph);
            }
        }
        for angle in &self.spokes {
            segment(
                sink,
                self.center,
                at(self.center, *angle, self.radius),
                glyph,
            );
        }
    }
}

/// Mark the cells a segment in dot units crosses.
fn segment(sink: &mut impl TextSink, a: (f64, f64), b: (f64, f64), glyph: &str) {
    let steps = ((b.0 - a.0).abs().max((b.1 - a.1).abs()) * 2.0)
        .ceil()
        .max(1.0) as usize;
    for step in 0..=steps {
        let t = step as f64 / steps as f64;
        let (x, y) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        if x >= 0.0 && y >= 0.0 {
            sink.under(
                (x / DOTS_X as f64) as usize,
                (y / DOTS_Y as f64) as usize,
                glyph,
                None,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spokes_start_at_twelve_and_divide_the_circle_evenly() {
        let angles = spoke_angles(4);
        let quarter = TAU / 4.0;
        for (k, angle) in angles.iter().enumerate() {
            assert!((angle - quarter * k as f64).abs() < 1e-12, "{angles:?}");
        }
        let (x, y) = at((10.0, 10.0), angles[1], 4.0);
        assert!(
            (x - 14.0).abs() < 1e-9 && (y - 10.0).abs() < 1e-9,
            "the second spoke points right"
        );
    }
}
