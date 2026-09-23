//! Curve interpolation for line and area strokes, ported from the reference's
//! `StrokeStyle`: `Natural` is a uniform Catmull-Rom spline rendered as cubic
//! Béziers, `Linear` joins points directly, `StepAfter` holds each value until
//! the next point.

/// How consecutive points are joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Curve {
    /// Smooth cubic spline through every point.
    #[default]
    Natural,
    /// Straight segments.
    Linear,
    /// Horizontal run at the old value, then a vertical riser at the new x.
    StepAfter,
}

/// Expand `points` (in range units) into a dense polyline following `curve`,
/// with about one vertex per `resolution` units of x. The first and last
/// input points are always the first and last output points.
pub fn polyline(points: &[(f64, f64)], curve: Curve, resolution: f64) -> Vec<(f64, f64)> {
    let n = points.len();
    if n < 2 {
        return points.to_vec();
    }
    let resolution = if resolution > 0.0 { resolution } else { 1.0 };
    let mut out = Vec::with_capacity(n * 4);
    match curve {
        Curve::Linear => out.extend_from_slice(points),
        Curve::StepAfter => {
            out.push(points[0]);
            for pair in points.windows(2) {
                out.push((pair[1].0, pair[0].1));
                out.push(pair[1]);
            }
        }
        Curve::Natural => {
            out.push(points[0]);
            for i in 0..n - 1 {
                let p0 = if i == 0 { points[0] } else { points[i - 1] };
                let p1 = points[i];
                let p2 = points[i + 1];
                let p3 = if i + 2 < n {
                    points[i + 2]
                } else {
                    points[n - 1]
                };
                let c1 = (p1.0 + (p2.0 - p0.0) / 6.0, p1.1 + (p2.1 - p0.1) / 6.0);
                let c2 = (p2.0 - (p3.0 - p1.0) / 6.0, p2.1 - (p3.1 - p1.1) / 6.0);
                let length = (p2.0 - p1.0).abs() + (p2.1 - p1.1).abs();
                let steps = ((length / resolution).ceil() as usize).clamp(1, 512);
                for s in 1..=steps {
                    let t = s as f64 / steps as f64;
                    out.push(cubic(p1, c1, c2, p2, t));
                }
            }
        }
    }
    out
}

fn cubic(p0: (f64, f64), c1: (f64, f64), c2: (f64, f64), p1: (f64, f64), t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let a = u * u * u;
    let b = 3.0 * u * u * t;
    let c = 3.0 * u * t * t;
    let d = t * t * t;
    (
        a * p0.0 + b * c1.0 + c * c2.0 + d * p1.0,
        a * p0.1 + b * c1.1 + c * c2.1 + d * p1.1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curve_keeps_its_endpoints() {
        let points = [(0.0, 0.0), (10.0, 8.0), (20.0, 2.0)];
        for curve in [Curve::Natural, Curve::Linear, Curve::StepAfter] {
            let line = polyline(&points, curve, 1.0);
            assert_eq!(line.first(), Some(&points[0]), "{curve:?}");
            assert_eq!(line.last(), Some(&points[2]), "{curve:?}");
        }
    }

    #[test]
    fn step_after_holds_the_old_value_until_the_next_x() {
        let line = polyline(&[(0.0, 0.0), (10.0, 8.0)], Curve::StepAfter, 1.0);
        assert_eq!(line, vec![(0.0, 0.0), (10.0, 0.0), (10.0, 8.0)]);
    }

    #[test]
    fn natural_passes_through_points_and_is_denser_than_linear() {
        let points = [(0.0, 0.0), (10.0, 8.0), (20.0, 2.0), (30.0, 9.0)];
        let natural = polyline(&points, Curve::Natural, 1.0);
        let linear = polyline(&points, Curve::Linear, 1.0);
        assert!(natural.len() > linear.len());
        for p in points {
            assert!(
                natural
                    .iter()
                    .any(|q| (q.0 - p.0).abs() < 1e-9 && (q.1 - p.1).abs() < 1e-9),
                "natural curve must pass through {p:?}"
            );
        }
        // The spline is smooth: it overshoots between points, so at the
        // large size class it differs visibly from the straight segments.
        let mid = natural.iter().find(|q| (q.0 - 15.0).abs() < 0.6).unwrap();
        assert!(
            (mid.1 - 5.0).abs() > 0.05,
            "natural must not equal linear at x=15: {mid:?}"
        );
    }

    #[test]
    fn short_inputs_pass_through() {
        assert!(polyline(&[], Curve::Natural, 1.0).is_empty());
        assert_eq!(
            polyline(&[(1.0, 1.0)], Curve::Natural, 1.0),
            vec![(1.0, 1.0)]
        );
    }
}
