//! Color ramps: a gradient's stops as a scale from a `0..=1` offset to a
//! color, so a renderer shading a bar or an area never interpolates colors
//! itself (CHT-010, CHT-013).

use super::Rgba;

/// `a` mixed toward `b` by `t` (0 is `a`, 1 is `b`); the alpha stays `a`'s.
pub fn mix(a: Rgba, b: Rgba, t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
        a.3,
    )
}

/// A gradient resolved to colors, kept in offset order.
#[derive(Debug, Clone, PartialEq)]
pub struct Ramp {
    stops: Vec<(f32, Rgba)>,
}

impl Ramp {
    /// Build a ramp from `(offset, color)` stops. Stops are sorted by offset;
    /// one with a non-finite offset is dropped.
    pub fn new(stops: impl IntoIterator<Item = (f32, Rgba)>) -> Self {
        let mut stops: Vec<(f32, Rgba)> = stops
            .into_iter()
            .filter(|(offset, _)| offset.is_finite())
            .collect();
        stops.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { stops }
    }

    /// Whether the ramp has no stops, so there is nothing to shade with.
    pub fn is_empty(&self) -> bool {
        self.stops.is_empty()
    }

    /// The stops in offset order.
    pub fn stops(&self) -> &[(f32, Rgba)] {
        &self.stops
    }

    /// The color at `t`: the first stop's color before the first offset, the
    /// last stop's after the last, and a linear mix of the two stops around
    /// `t` between them. `None` when the ramp has no stops.
    pub fn at(&self, t: f32) -> Option<Rgba> {
        let (first, last) = (self.stops.first()?, self.stops.last()?);
        if t <= first.0 {
            return Some(first.1);
        }
        if t >= last.0 {
            return Some(last.1);
        }
        let after = self.stops.iter().position(|(offset, _)| *offset >= t)?;
        let (a, b) = (self.stops[after - 1], self.stops[after]);
        let span = (b.0 - a.0).max(f32::EPSILON);
        Some(mix(a.1, b.1, (t - a.0) / span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: Rgba = (1.0, 0.0, 0.0, 1.0);
    const BLUE: Rgba = (0.0, 0.0, 1.0, 0.5);

    #[test]
    fn mix_moves_from_a_to_b_and_keeps_the_alpha_of_a() {
        assert_eq!(mix(RED, BLUE, 0.0), RED);
        assert_eq!(mix(RED, BLUE, 1.0), (0.0, 0.0, 1.0, 1.0));
        assert_eq!(mix(RED, BLUE, 0.5), (0.5, 0.0, 0.5, 1.0));
        assert_eq!(mix(RED, BLUE, 7.0), (0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn a_ramp_holds_its_ends_and_mixes_between_stops() {
        let ramp = Ramp::new([(0.25, RED), (0.75, BLUE)]);
        assert_eq!(ramp.at(0.0), Some(RED));
        assert_eq!(ramp.at(0.25), Some(RED));
        assert_eq!(ramp.at(1.0), Some(BLUE));
        assert_eq!(ramp.at(0.5), Some((0.5, 0.0, 0.5, 1.0)));
    }

    #[test]
    fn stops_are_sorted_and_an_empty_ramp_shades_nothing() {
        let ramp = Ramp::new([(1.0, BLUE), (f32::NAN, RED), (0.0, RED)]);
        assert_eq!(ramp.stops(), &[(0.0, RED), (1.0, BLUE)]);
        assert_eq!(ramp.at(0.5), Some((0.5, 0.0, 0.5, 1.0)));
        let empty = Ramp::new([]);
        assert!(empty.is_empty());
        assert_eq!(empty.at(0.5), None);
    }
}
