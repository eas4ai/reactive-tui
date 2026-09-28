//! Coverage of polygons by exact area, as font rasterizers compute it: each
//! edge adds the signed area it sweeps to an accumulation row, and a running
//! sum along the row gives each pixel's winding. Its magnitude, capped at
//! one, is the non-zero rule with anti-aliased edges. The software renderer
//! fills every shape with it, and the GPU's glyph atlas holds glyph
//! coverage it computed, so both renderers draw text from the same pixels.

use super::geometry::{Point, Polygon};

/// Per-pixel coverage in `0.0..=1.0` of a window of the picture.
pub(crate) struct Coverage {
    /// The window's left column and top row in the picture.
    pub left: u32,
    pub top: u32,
    pub width: usize,
    pub height: usize,
    /// Row by row.
    pub alpha: Vec<f32>,
}

impl Coverage {
    /// Coverage at picture pixel (`x`, `y`); none outside the window.
    pub fn at(&self, x: u32, y: u32) -> f32 {
        if x < self.left || y < self.top {
            return 0.0;
        }
        let (dx, dy) = ((x - self.left) as usize, (y - self.top) as usize);
        if dx >= self.width || dy >= self.height {
            return 0.0;
        }
        self.alpha[dy * self.width + dx]
    }
}

/// The pixels a shape can cover: whole pixels from `left`, `top` up to but
/// not including `right`, `bottom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Window {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Window {
    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }
}

/// The bounding box of `polygons` within a picture of `size`, in whole
/// pixels; `None` when they lie outside it or have no area to bound.
pub(crate) fn window_of(polygons: &[Polygon], size: (u32, u32)) -> Option<Window> {
    let mut min = (f32::INFINITY, f32::INFINITY);
    let mut max = (f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &(x, y) in polygons.iter().flatten() {
        if !x.is_finite() || !y.is_finite() {
            continue;
        }
        min = (min.0.min(x), min.1.min(y));
        max = (max.0.max(x), max.1.max(y));
    }
    let left = min.0.floor().max(0.0);
    let top = min.1.floor().max(0.0);
    let right = max.0.ceil().min(size.0 as f32);
    let bottom = max.1.ceil().min(size.1 as f32);
    (right > left && bottom > top).then(|| Window {
        left: left as u32,
        top: top as u32,
        right: right as u32,
        bottom: bottom as u32,
    })
}

/// The coverage of `polygons`, filled together by the non-zero rule, over
/// their bounding box within a picture of `size`; `None` when they cover
/// none of it.
pub(crate) fn rasterize(polygons: &[Polygon], size: (u32, u32)) -> Option<Coverage> {
    let window = window_of(polygons, size)?;
    let (left, top) = (window.left as f32, window.top as f32);
    let (width, height) = (window.width() as usize, window.height() as usize);
    let stride = width + 2;
    let mut acc = vec![0f32; stride * height];
    for polygon in polygons {
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            if a.0.is_finite() && a.1.is_finite() && b.0.is_finite() && b.1.is_finite() {
                edge(
                    &mut acc,
                    stride,
                    width,
                    height,
                    (a.0 - left, a.1 - top),
                    (b.0 - left, b.1 - top),
                );
            }
        }
    }
    let mut alpha = vec![0f32; width * height];
    for y in 0..height {
        let mut sum = 0.0f32;
        for x in 0..width {
            sum += acc[y * stride + x];
            alpha[y * width + x] = sum.abs().min(1.0);
        }
    }
    Some(Coverage {
        left: window.left,
        top: window.top,
        width,
        height,
        alpha,
    })
}

/// Add the area the edge from `p0` to `p1` sweeps to `acc`, a window
/// `width` by `height` pixels whose rows are `stride` apart. An edge left
/// of the window counts at its left border, which keeps the running sums
/// right; one right of it lands in the two spare columns and is never read.
fn edge(acc: &mut [f32], stride: usize, width: usize, height: usize, p0: Point, p1: Point) {
    if p0.1 == p1.1 {
        return;
    }
    let (dir, p0, p1) = if p0.1 < p1.1 {
        (1.0, p0, p1)
    } else {
        (-1.0, p1, p0)
    };
    let (y_start, y_end) = (p0.1.max(0.0), p1.1.min(height as f32));
    if y_end <= y_start {
        return;
    }
    let dxdy = (p1.0 - p0.0) / (p1.1 - p0.1);
    let right = width as f32;
    let mut x = p0.0 + (y_start - p0.1) * dxdy;
    for row in (y_start.floor() as usize)..(y_end.ceil() as usize).min(height) {
        let dy = ((row + 1) as f32).min(y_end) - (row as f32).max(y_start);
        if dy <= 0.0 {
            continue;
        }
        let next = x + dxdy * dy;
        let d = dy * dir;
        let (x0, x1) = if x < next { (x, next) } else { (next, x) };
        let (x0, x1) = (x0.clamp(0.0, right), x1.clamp(0.0, right));
        let base = row * stride;
        let x0_floor = x0.floor();
        let x0i = x0_floor as usize;
        let x1_ceil = x1.ceil();
        let x1i = x1_ceil as usize;
        if x1i <= x0i + 1 {
            let mid = 0.5 * (x0 + x1) - x0_floor;
            acc[base + x0i] += d - d * mid;
            acc[base + x0i + 1] += d * mid;
        } else {
            let s = (x1 - x0).recip();
            let x0f = x0 - x0_floor;
            let a0 = 0.5 * s * (1.0 - x0f) * (1.0 - x0f);
            let x1f = x1 - x1_ceil + 1.0;
            let am = 0.5 * s * x1f * x1f;
            acc[base + x0i] += d * a0;
            if x1i == x0i + 2 {
                acc[base + x0i + 1] += d * (1.0 - a0 - am);
            } else {
                let a1 = s * (1.5 - x0f);
                acc[base + x0i + 1] += d * (a1 - a0);
                for xi in x0i + 2..x1i - 1 {
                    acc[base + xi] += d * s;
                }
                let a2 = a1 + (x1i - x0i - 3) as f32 * s;
                acc[base + x1i - 1] += d * (1.0 - a2 - am);
            }
            acc[base + x1i] += d * am;
        }
        x = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x: f32, y: f32, side: f32) -> Polygon {
        vec![(x, y), (x + side, y), (x + side, y + side), (x, y + side)]
    }

    #[test]
    fn a_pixel_aligned_square_covers_its_pixels_fully() {
        let coverage = rasterize(&[square(2.0, 3.0, 4.0)], (10, 10)).unwrap();
        for y in 0..10 {
            for x in 0..10 {
                let inside = (2..6).contains(&x) && (3..7).contains(&y);
                assert_eq!(
                    coverage.at(x, y),
                    if inside { 1.0 } else { 0.0 },
                    "({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn a_half_pixel_offset_covers_edge_pixels_by_half() {
        let coverage = rasterize(&[square(1.5, 1.0, 2.0)], (6, 6)).unwrap();
        assert!((coverage.at(1, 1) - 0.5).abs() < 1e-5);
        assert!((coverage.at(2, 1) - 1.0).abs() < 1e-5);
        assert!((coverage.at(3, 1) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn a_diagonal_halves_the_pixels_it_crosses() {
        let triangle = vec![(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)];
        let coverage = rasterize(&[triangle], (4, 4)).unwrap();
        for i in 0..4 {
            assert!((coverage.at(3 - i, i) - 0.5).abs() < 1e-4, "{i}");
        }
        let total: f32 = coverage.alpha.iter().sum();
        assert!((total - 8.0).abs() < 1e-3, "area {total}");
    }

    #[test]
    fn opposite_windings_cut_a_hole_and_same_windings_do_not() {
        let outer = square(0.0, 0.0, 6.0);
        let mut inner = square(2.0, 2.0, 2.0);
        let same = rasterize(&[outer.clone(), inner.clone()], (6, 6)).unwrap();
        assert_eq!(same.at(3, 3), 1.0);
        inner.reverse();
        let hole = rasterize(&[outer, inner], (6, 6)).unwrap();
        assert_eq!(hole.at(3, 3), 0.0);
        assert_eq!(hole.at(1, 1), 1.0);
    }

    #[test]
    fn a_shape_past_the_picture_edges_is_cut_there() {
        let coverage = rasterize(&[square(-3.0, -3.0, 5.0)], (4, 4)).unwrap();
        assert_eq!((coverage.left, coverage.top), (0, 0));
        assert_eq!(coverage.at(0, 0), 1.0);
        assert_eq!(coverage.at(1, 1), 1.0);
        assert_eq!(coverage.at(2, 2), 0.0);
    }
}
