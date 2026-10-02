//! Paths to polygons in picture pixels, for both renderers: curves and arcs
//! flattened to lines within a quarter pixel, strokes expanded into convex
//! pieces of one winding, so that filling the pieces by the non-zero rule
//! draws the stroke.

use super::scene::{LineCap, LineJoin, Path, Segment, Stroke, Transform};

/// A point in picture pixels.
pub(crate) type Point = (f32, f32);

/// A closed polygon; filled by the non-zero rule together with the others
/// of its shape.
pub(crate) type Polygon = Vec<Point>;

/// How far a flattened curve may stray from the true one, in pixels.
const TOLERANCE: f32 = 0.25;

/// The most runs a dash pattern draws on one line: a line four times as
/// long as the largest picture is wide, at the shortest pattern that is
/// drawn as dashes.
const MAX_RUNS: usize = 65_536;

/// One subpath flattened to a polyline, and whether it was closed.
pub(crate) struct Polyline {
    pub points: Vec<Point>,
    pub closed: bool,
}

/// `path` under `transform`, flattened to polylines in picture pixels.
pub(crate) fn flatten(path: &Path, transform: &Transform) -> Vec<Polyline> {
    let mut lines: Vec<Polyline> = Vec::new();
    let mut current: Vec<Point> = Vec::new();
    // In scene coordinates, for curves and arcs, which flatten before the
    // transform so that an arc under a rotation stays an ellipse.
    let mut at: Point = (0.0, 0.0);
    let mut start: Point = (0.0, 0.0);
    let scale = transform.max_scale().max(1e-6);
    let finish = |current: &mut Vec<Point>, lines: &mut Vec<Polyline>, closed: bool| {
        if current.len() > 1 {
            lines.push(Polyline {
                points: std::mem::take(current),
                closed,
            });
        } else {
            current.clear();
        }
    };
    for segment in &path.segments {
        match *segment {
            Segment::MoveTo(p) => {
                finish(&mut current, &mut lines, false);
                current.push(transform.apply(p));
                at = p;
                start = p;
            }
            Segment::LineTo(p) => {
                if current.is_empty() {
                    current.push(transform.apply(at));
                }
                current.push(transform.apply(p));
                at = p;
            }
            Segment::QuadTo(c, p) => {
                if current.is_empty() {
                    current.push(transform.apply(at));
                }
                let from = at;
                let steps = curve_steps(&[from, c, p], scale);
                for step in 1..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let point = (
                        u * u * from.0 + 2.0 * u * t * c.0 + t * t * p.0,
                        u * u * from.1 + 2.0 * u * t * c.1 + t * t * p.1,
                    );
                    current.push(transform.apply(point));
                }
                at = p;
            }
            Segment::CubicTo(c1, c2, p) => {
                if current.is_empty() {
                    current.push(transform.apply(at));
                }
                let from = at;
                let steps = curve_steps(&[from, c1, c2, p], scale);
                for step in 1..=steps {
                    let t = step as f32 / steps as f32;
                    let u = 1.0 - t;
                    let point = (
                        u * u * u * from.0
                            + 3.0 * u * u * t * c1.0
                            + 3.0 * u * t * t * c2.0
                            + t * t * t * p.0,
                        u * u * u * from.1
                            + 3.0 * u * u * t * c1.1
                            + 3.0 * u * t * t * c2.1
                            + t * t * t * p.1,
                    );
                    current.push(transform.apply(point));
                }
                at = p;
            }
            Segment::ArcTo {
                radii,
                rotation,
                large,
                sweep,
                to,
            } => {
                if current.is_empty() {
                    current.push(transform.apply(at));
                }
                for point in arc_points(at, radii, rotation, large, sweep, to, scale) {
                    current.push(transform.apply(point));
                }
                at = to;
            }
            Segment::Close => {
                finish(&mut current, &mut lines, true);
                at = start;
            }
        }
    }
    finish(&mut current, &mut lines, false);
    lines
}

/// Enough line segments that a curve through `points` (its control polygon)
/// strays at most [`TOLERANCE`] pixels at `scale`.
fn curve_steps(points: &[Point], scale: f32) -> usize {
    let mut length = 0.0;
    for pair in points.windows(2) {
        length += distance(pair[0], pair[1]);
    }
    let length = length * scale;
    // A chord of a curve whose control polygon is `length` long strays about
    // length^2 / (8 n^2 r); bounding the bend by the length itself gives
    // n = sqrt(length / (8 tolerance)) at worst, doubled for margin.
    ((length / (8.0 * TOLERANCE)).sqrt() * 2.0)
        .ceil()
        .clamp(1.0, 512.0) as usize
}

fn distance(a: Point, b: Point) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

/// Points along SVG's elliptical arc from `from` to `to`, excluding `from`.
fn arc_points(
    from: Point,
    (rx, ry): (f32, f32),
    rotation: f32,
    large: bool,
    sweep: bool,
    to: Point,
    scale: f32,
) -> Vec<Point> {
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx < 1e-6 || ry < 1e-6 || distance(from, to) < 1e-6 {
        return vec![to];
    }
    let (sin, cos) = rotation.to_radians().sin_cos();
    // Endpoint to centre parameterization (SVG 1.1, appendix F.6.5).
    let dx = (from.0 - to.0) / 2.0;
    let dy = (from.1 - to.1) / 2.0;
    let x1 = cos * dx + sin * dy;
    let y1 = -sin * dx + cos * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let grow = lambda.sqrt();
        rx *= grow;
        ry *= grow;
    }
    let numerator = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1;
    let denominator = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut factor = (numerator / denominator).max(0.0).sqrt();
    if large == sweep {
        factor = -factor;
    }
    let cx1 = factor * rx * y1 / ry;
    let cy1 = -factor * ry * x1 / rx;
    let cx = cos * cx1 - sin * cy1 + (from.0 + to.0) / 2.0;
    let cy = sin * cx1 + cos * cy1 + (from.1 + to.1) / 2.0;
    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 {
            -a
        } else {
            a
        }
    };
    let theta = angle(1.0, 0.0, (x1 - cx1) / rx, (y1 - cy1) / ry);
    let mut delta = angle(
        (x1 - cx1) / rx,
        (y1 - cy1) / ry,
        (-x1 - cx1) / rx,
        (-y1 - cy1) / ry,
    );
    if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    } else if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    }
    // A chord of angle a on radius r strays r (1 - cos(a/2)).
    let radius = rx.max(ry) * scale;
    let step = (2.0 * (1.0 - TOLERANCE / radius).clamp(-1.0, 1.0).acos()).max(1e-3);
    let steps = (delta.abs() / step).ceil().clamp(1.0, 1024.0) as usize;
    (1..=steps)
        .map(|i| {
            if i == steps {
                return to;
            }
            let t = theta + delta * i as f32 / steps as f32;
            let (s, c) = t.sin_cos();
            (
                cx + cos * rx * c - sin * ry * s,
                cy + sin * rx * c + cos * ry * s,
            )
        })
        .collect()
}

/// The polygons of `path` under `transform`, filled by the non-zero rule.
pub(crate) fn fill_polygons(path: &Path, transform: &Transform) -> Vec<Polygon> {
    flatten(path, transform)
        .into_iter()
        .map(|line| line.points)
        .collect()
}

/// The polygons that draw `path`'s outline with `stroke` under
/// `transform`, all wound the same way, so the non-zero rule fills their
/// union. The stroke width scales with the transform's largest stretch.
pub(crate) fn stroke_polygons(path: &Path, stroke: &Stroke, transform: &Transform) -> Vec<Polygon> {
    let half = stroke.width * transform.max_scale() / 2.0;
    let mut pieces = Vec::new();
    if half <= 0.0 {
        return pieces;
    }
    for line in flatten(path, transform) {
        let points = dedupe(&line.points);
        if points.len() < 2 {
            // A zero-length subpath draws its caps, as a dot.
            if let (Some(&p), LineCap::Round | LineCap::Square) = (points.first(), stroke.cap) {
                pieces.push(cap_dot(p, half, stroke.cap));
            }
            continue;
        }
        let dashes = dash(&points, line.closed, &stroke.dash, transform.max_scale());
        for (run, closed) in dashes {
            stroke_run(&run, closed, half, stroke, &mut pieces);
        }
    }
    pieces.into_iter().map(counter_clockwise).collect()
}

/// `points` without consecutive repeats.
fn dedupe(points: &[Point]) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for &p in points {
        if out.last().is_none_or(|&q| distance(p, q) > 1e-4) {
            out.push(p);
        }
    }
    out
}

/// The runs of a polyline that a dash pattern draws, in picture pixels,
/// and whether a run is the whole closed line. The line is drawn solid
/// when the pattern's lengths add up to less than [`TOLERANCE`] pixels, as
/// a pattern of zeros is, and when the pattern would draw more than
/// [`MAX_RUNS`] runs.
fn dash(points: &[Point], closed: bool, pattern: &[f32], scale: f32) -> Vec<(Vec<Point>, bool)> {
    let solid = || vec![(points.to_vec(), closed)];
    let pattern: Vec<f32> = pattern.iter().map(|length| length * scale).collect();
    let period: f32 = pattern.iter().sum();
    if period.is_nan() || period < TOLERANCE {
        return solid();
    }
    let mut path: Vec<Point> = points.to_vec();
    if closed {
        path.push(points[0]);
    }
    let mut runs = Vec::new();
    let mut index = 0;
    let mut left = pattern[0];
    let mut on = true;
    let mut run: Vec<Point> = vec![path[0]];
    for pair in path.windows(2) {
        let (mut a, b) = (pair[0], pair[1]);
        let mut length = distance(a, b);
        while length > left {
            let t = left / length;
            let cut = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            if on {
                run.push(cut);
                runs.push((std::mem::take(&mut run), false));
                // Every second pass ends a run, so this also ends a walk
                // whose length is too large for f32 to take a dash from.
                if runs.len() > MAX_RUNS {
                    return solid();
                }
            } else {
                run = vec![cut];
            }
            length -= left;
            a = cut;
            on = !on;
            index = (index + 1) % pattern.len();
            left = pattern[index];
        }
        left -= length;
        if on {
            run.push(b);
        }
    }
    if on && run.len() > 1 {
        runs.push((run, false));
    }
    runs
}

/// Offset of `half` to the left of the direction from `a` to `b`.
fn normal(a: Point, b: Point, half: f32) -> Point {
    let length = distance(a, b).max(1e-6);
    (-(b.1 - a.1) / length * half, (b.0 - a.0) / length * half)
}

fn add(a: Point, b: Point) -> Point {
    (a.0 + b.0, a.1 + b.1)
}

fn sub(a: Point, b: Point) -> Point {
    (a.0 - b.0, a.1 - b.1)
}

/// A regular polygon approximating a circle of `radius` around `center`,
/// its chords within the flattening tolerance.
fn circle(center: Point, radius: f32) -> Polygon {
    let step = (2.0
        * (1.0 - TOLERANCE / radius.max(TOLERANCE))
            .clamp(-1.0, 1.0)
            .acos())
    .max(0.05);
    let steps = (std::f32::consts::TAU / step).ceil().clamp(8.0, 256.0) as usize;
    (0..steps)
        .map(|i| {
            let t = std::f32::consts::TAU * i as f32 / steps as f32;
            (center.0 + radius * t.cos(), center.1 + radius * t.sin())
        })
        .collect()
}

/// The cap of a zero-length stroke at `p`.
fn cap_dot(p: Point, half: f32, cap: LineCap) -> Polygon {
    match cap {
        LineCap::Round => circle(p, half),
        _ => vec![
            (p.0 - half, p.1 - half),
            (p.0 + half, p.1 - half),
            (p.0 + half, p.1 + half),
            (p.0 - half, p.1 + half),
        ],
    }
}

/// The pieces of one run of a stroke: a quadrilateral per segment, a join
/// at each inner vertex (and at the start of a closed run), and caps at the
/// ends of an open one.
fn stroke_run(
    points: &[Point],
    closed: bool,
    half: f32,
    stroke: &Stroke,
    pieces: &mut Vec<Polygon>,
) {
    let n = points.len();
    let segments: Vec<(Point, Point)> = if closed {
        (0..n).map(|i| (points[i], points[(i + 1) % n])).collect()
    } else {
        points.windows(2).map(|w| (w[0], w[1])).collect()
    };
    for &(a, b) in &segments {
        let off = normal(a, b, half);
        pieces.push(vec![add(a, off), add(b, off), sub(b, off), sub(a, off)]);
    }
    let joins = if closed {
        segments.len()
    } else {
        segments.len().saturating_sub(1)
    };
    for i in 0..joins {
        let (a, b) = segments[i];
        let (_, c) = segments[(i + 1) % segments.len()];
        join(a, b, c, half, stroke, pieces);
    }
    if !closed {
        let (first, second) = segments[0];
        let (before, last) = segments[segments.len() - 1];
        cap(first, second, half, stroke.cap, pieces);
        cap(last, before, half, stroke.cap, pieces);
    }
}

/// The join at `b` between segments a-b and b-c.
fn join(a: Point, b: Point, c: Point, half: f32, stroke: &Stroke, pieces: &mut Vec<Polygon>) {
    let n1 = normal(a, b, half);
    let n2 = normal(b, c, half);
    let turn = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
    if turn.abs() < 1e-6 && (b.0 - a.0) * (c.0 - b.0) + (b.1 - a.1) * (c.1 - b.1) >= 0.0 {
        return;
    }
    // The outer side of the turn is the side the segments' offsets part on.
    let (o1, o2) = if turn > 0.0 {
        (sub(b, n1), sub(b, n2))
    } else {
        (add(b, n1), add(b, n2))
    };
    match stroke.join {
        LineJoin::Round => pieces.push(circle(b, half)),
        LineJoin::Bevel => pieces.push(vec![b, o1, o2]),
        LineJoin::Miter => {
            // The miter's tip, where the two outer edges meet.
            let d1 = sub(b, a);
            let d2 = sub(c, b);
            let cross = d1.0 * d2.1 - d1.1 * d2.0;
            let tip = if cross.abs() > 1e-9 {
                let t = ((o2.0 - o1.0) * d2.1 - (o2.1 - o1.1) * d2.0) / cross;
                Some((o1.0 + d1.0 * t, o1.1 + d1.1 * t))
            } else {
                None
            };
            match tip.filter(|&tip| distance(tip, b) <= stroke.miter_limit * half) {
                Some(tip) => pieces.push(vec![b, o1, tip, o2]),
                None => pieces.push(vec![b, o1, o2]),
            }
        }
    }
}

/// The cap past `end`, for the segment coming from `from`.
fn cap(end: Point, from: Point, half: f32, cap: LineCap, pieces: &mut Vec<Polygon>) {
    match cap {
        LineCap::Butt => {}
        LineCap::Round => pieces.push(circle(end, half)),
        LineCap::Square => {
            let length = distance(from, end).max(1e-6);
            let out = (
                (end.0 - from.0) / length * half,
                (end.1 - from.1) / length * half,
            );
            let off = normal(from, end, half);
            pieces.push(vec![
                add(end, off),
                add(add(end, off), out),
                add(sub(end, off), out),
                sub(end, off),
            ]);
        }
    }
}

/// `polygon` wound counter-clockwise on screen (positive signed area with y
/// growing downwards), so the pieces of a stroke add up.
fn counter_clockwise(mut polygon: Polygon) -> Polygon {
    let area: f32 = (0..polygon.len())
        .map(|i| {
            let (a, b) = (polygon[i], polygon[(i + 1) % polygon.len()]);
            a.0 * b.1 - b.0 * a.1
        })
        .sum();
    if area < 0.0 {
        polygon.reverse();
    }
    polygon
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::scene::PathBuilder;

    #[test]
    fn a_rectangle_flattens_to_its_corners() {
        let lines = flatten(&Path::rect(1.0, 2.0, 3.0, 4.0), &Transform::identity());
        assert_eq!(lines.len(), 1);
        assert!(lines[0].closed);
        assert_eq!(
            lines[0].points,
            vec![(1.0, 2.0), (4.0, 2.0), (4.0, 6.0), (1.0, 6.0)]
        );
    }

    #[test]
    fn an_ellipse_stays_within_the_tolerance_of_its_radius() {
        let lines = flatten(
            &Path::ellipse(50.0, 50.0, 40.0, 20.0),
            &Transform::identity(),
        );
        for &(x, y) in &lines[0].points {
            let r = ((x - 50.0) / 40.0).powi(2) + ((y - 50.0) / 20.0).powi(2);
            assert!((r - 1.0).abs() < 0.03, "({x}, {y}) is off the ellipse: {r}");
        }
        assert!(lines[0].points.len() > 16);
    }

    #[test]
    fn stroke_pieces_are_all_wound_one_way() {
        let path = PathBuilder::new()
            .move_to(0.0, 0.0)
            .line_to(10.0, 0.0)
            .line_to(10.0, 10.0)
            .build();
        for join in [LineJoin::Miter, LineJoin::Round, LineJoin::Bevel] {
            let pieces =
                stroke_polygons(&path, &Stroke::new(2.0).join(join), &Transform::identity());
            assert!(pieces.len() >= 3);
            for piece in pieces {
                let area: f32 = (0..piece.len())
                    .map(|i| {
                        let (a, b) = (piece[i], piece[(i + 1) % piece.len()]);
                        a.0 * b.1 - b.0 * a.1
                    })
                    .sum();
                assert!(area >= 0.0, "{join:?} piece wound the other way");
            }
        }
    }

    #[test]
    fn a_dash_pattern_splits_a_line_into_runs() {
        let runs = dash(&[(0.0, 0.0), (10.0, 0.0)], false, &[3.0, 2.0], 1.0);
        let starts: Vec<f32> = runs.iter().map(|(run, _)| run[0].0).collect();
        assert_eq!(starts, vec![0.0, 5.0]);
        assert!((runs[0].0[1].0 - 3.0).abs() < 1e-5);
    }

    #[test]
    fn a_dash_pattern_the_picture_cannot_show_draws_a_solid_line() {
        let line = [(0.0, 0.0), (300.0, 0.0)];
        // f32 cannot take 0.00001 from a length of 300.
        for pattern in [[0.000_01, 0.000_01], [0.1, 0.1]] {
            assert_eq!(
                dash(&line, false, &pattern, 1.0),
                vec![(line.to_vec(), false)]
            );
        }
        // Under a scale of 0.01 a pattern of 4 and 4 has a period of 0.08 pixels.
        assert_eq!(
            dash(&line, true, &[4.0, 4.0], 0.01),
            vec![(line.to_vec(), true)]
        );
    }

    #[test]
    fn a_dash_pattern_draws_at_most_the_largest_number_of_runs() {
        // f32 cannot take 0.5 from a length of 100 000 000.
        for length in [100_000.0, 100_000_000.0] {
            let line = [(0.0, 0.0), (length, 0.0)];
            assert_eq!(
                dash(&line, false, &[0.5, 0.5], 1.0),
                vec![(line.to_vec(), false)]
            );
        }
        let runs = dash(&[(0.0, 0.0), (1000.0, 0.0)], false, &[0.5, 0.5], 1.0);
        assert_eq!(runs.len(), 1000);
    }
}
