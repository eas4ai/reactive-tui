//! Where a cartesian plot's shapes go: the shared mask canvas, in dots, for
//! a plot drawn in cells (CHT-025), or a scene in pixels for a plot drawn
//! as a picture (CHT-037). The renderer computes its geometry in the units
//! the target measures a cell in and hands every shape over here, so the
//! two paths share one layout.

use super::super::super::mask::{Marker, MaskCanvas, DOTS_X, DOTS_Y};
use super::super::super::plot::{mix, polyline, Curve, Ramp, Rect, Rgba, ScaleLinear};
use super::super::super::{FillStyle, LineStyle};
#[cfg(feature = "wgpu-graphics")]
use super::pixels::PlotScene;

/// The target the plot's shapes are drawn into.
pub(super) enum Shapes<'a> {
    /// The shared mask canvas: two by four dots per cell.
    Mask(&'a mut MaskCanvas),
    /// A plot picture: the terminal's pixels per cell.
    #[cfg(feature = "wgpu-graphics")]
    Pixels(PlotScene),
}

impl Shapes<'_> {
    /// How many units a cell measures across and down.
    pub fn units(&self) -> (f64, f64) {
        match self {
            Shapes::Mask(_) => (DOTS_X as f64, DOTS_Y as f64),
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => scene.units(),
        }
    }

    /// Whether the shapes go to a picture.
    pub fn is_pixels(&self) -> bool {
        match self {
            Shapes::Mask(_) => false,
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(_) => true,
        }
    }

    /// Shapes keep inside `inner`, the plot without its axis lines.
    pub fn begin(&mut self, inner: Rect) {
        match self {
            Shapes::Mask(mask) => mask.set_clip_cells(inner.x, inner.y, inner.w, inner.h),
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let (ux, uy) = scene.units();
                scene.begin((
                    (inner.x as f64 * ux) as f32,
                    (inner.y as f64 * uy) as f32,
                    (inner.w as f64 * ux) as f32,
                    (inner.h as f64 * uy) as f32,
                ));
            }
        }
    }

    /// A rectangle between two corners: on the mask with its exact edge
    /// fractions, in a picture at the exact pixel with its corners rounded
    /// by `radius` pixels and its color at `alpha` (CHT-013).
    #[allow(clippy::too_many_arguments, unused_variables)]
    pub fn rect(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
        radius: f32,
        alpha: f32,
    ) {
        match self {
            Shapes::Mask(mask) => mask.rect(x0, y0, x1, y1, color, owner),
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                if let Some(color) = color.or(scene.fallback()) {
                    scene.solid_rect(x0, y0, x1, y1, color, alpha, radius);
                }
            }
        }
    }

    /// A bar from `from` to `to` along its growth axis, between `lane_start`
    /// and `lane_end` across it, filled with `ramp` from base to tip: on the
    /// mask one cell slab at a time, each in the gradient's color where the
    /// slab's middle falls; in a picture as one gradient (CHT-013).
    #[allow(clippy::too_many_arguments, unused_variables)]
    pub fn gradient_bar(
        &mut self,
        from: f64,
        to: f64,
        lane_start: f64,
        lane_end: f64,
        horizontal: bool,
        ramp: &Ramp,
        tint: Option<Rgba>,
        owner: Option<(usize, usize)>,
        radius: f32,
        alpha: f32,
    ) {
        match self {
            Shapes::Mask(mask) => {
                let dots = if horizontal { DOTS_X } else { DOTS_Y } as f64;
                let along = ScaleLinear::new((from, to), (0.0, 1.0));
                let (lo, hi) = (from.min(to), from.max(to));
                let mut at = lo;
                while at < hi {
                    let next = ((at / dots).floor() + 1.0) * dots;
                    let next = next.min(hi);
                    let middle = (at + next) / 2.0;
                    let shade = ramp.at(along.map(middle) as f32).or(tint);
                    if horizontal {
                        mask.rect(at, lane_start, next, lane_end, shade, owner);
                    } else {
                        mask.rect(lane_start, at, lane_end, next, shade, owner);
                    }
                    at = next;
                }
            }
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let middle = (lane_start + lane_end) / 2.0;
                let (start, end) = if horizontal {
                    ((from, middle), (to, middle))
                } else {
                    ((middle, from), (middle, to))
                };
                let stops: Vec<(f32, Rgba)> = ramp.stops().to_vec();
                if horizontal {
                    scene.gradient_rect(
                        from, lane_start, to, lane_end, &stops, start, end, alpha, radius,
                    );
                } else {
                    scene.gradient_rect(
                        lane_start, from, lane_end, to, &stops, start, end, alpha, radius,
                    );
                }
            }
        }
    }

    /// A series' line through `points` following `curve`, in its style
    /// (CHT-012): a dot-dense polyline on the mask, a path of one segment
    /// per pair of points in a picture.
    pub fn polyline(
        &mut self,
        points: &[(f64, f64)],
        curve: Curve,
        tint: Option<Rgba>,
        owner: Option<(usize, usize)>,
        style: &LineStyle,
    ) {
        match self {
            Shapes::Mask(mask) => {
                let dense = polyline(points, curve, 1.0);
                match style {
                    LineStyle::Solid => mask.polyline(&dense, tint, owner),
                    LineStyle::Dashed => {
                        for (k, pair) in dense.windows(2).enumerate() {
                            if k % 8 < 4 {
                                mask.line(pair[0], pair[1], tint, owner);
                            }
                        }
                    }
                    LineStyle::Dotted => {
                        for (k, p) in dense.iter().enumerate() {
                            if k % 3 == 0 {
                                mask.dot(p.0.round() as i64, p.1.round() as i64, tint, owner);
                            }
                        }
                    }
                    LineStyle::None => {}
                }
            }
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let width = scene.stroke_width();
                let half = scene.cell_height() / 2.0;
                let dash: Option<Vec<f32>> = match style {
                    LineStyle::Solid => None,
                    LineStyle::Dashed => Some(vec![half, half]),
                    LineStyle::Dotted => Some(vec![0.01, width * 2.0]),
                    LineStyle::None => return,
                };
                let Some(tint) = tint.or(scene.fallback()) else {
                    return;
                };
                scene.polyline(points, curve, tint, width, dash.as_deref());
            }
        }
    }

    /// A point marker at (`x`, `y`): a glyph in the cell on the mask, a disc
    /// in a picture, half a cell high for a scatter's dot and a third for a
    /// line's (CHT-012).
    pub fn marker(
        &mut self,
        x: f64,
        y: f64,
        marker: Marker,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        match self {
            Shapes::Mask(mask) => mask.marker(x, y, marker, color, owner),
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let radius = match marker {
                    Marker::Dot => scene.cell_height() / 4.0,
                    Marker::Disc => scene.cell_height() / 6.0,
                };
                if let Some(color) = color.or(scene.fallback()) {
                    scene.disc(x, y, radius, color, 1.0);
                }
            }
        }
    }

    /// The area between the `top` and `bottom` lines following `curve`:
    /// `tint` at `opacity`, for a gradient strongest at the stroke and
    /// fading to the baseline (CHT-012). `rows` are the plot's rows in
    /// cells. The cells the fill covers are collected for a pattern's marks.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_between(
        &mut self,
        top: &[(f64, f64)],
        bottom: &[(f64, f64)],
        curve: Curve,
        tint: Option<Rgba>,
        background: Option<Rgba>,
        opacity: f32,
        style: &FillStyle,
        owner: Option<(usize, usize)>,
        covered: &mut Vec<(usize, usize)>,
        rows: (usize, usize),
    ) {
        match self {
            Shapes::Mask(mask) => fill_between(
                mask,
                &polyline(top, curve, 1.0),
                &polyline(bottom, curve, 1.0),
                tint,
                background,
                opacity,
                style,
                owner,
                covered,
                (rows.0 * DOTS_Y, rows.1 * DOTS_Y),
            ),
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let Some(tint) = tint else {
                    return;
                };
                let opacity = opacity.clamp(0.0, 1.0);
                match style {
                    FillStyle::Gradient => {
                        scene.gradient_between(top, bottom, curve, tint, opacity)
                    }
                    FillStyle::Pattern(_) => {}
                    _ => scene.fill_between(top, bottom, curve, tint, opacity),
                }
                // The cells the area covers, one column of cells at a time
                // at the column's middle, for the marks of a pattern; the
                // lines are followed at a vertex per cell.
                let (ux, uy) = scene.units();
                let top = polyline(top, curve, ux);
                let bottom = polyline(bottom, curve, ux);
                if let (Some(first), Some(last)) = (top.first(), top.last()) {
                    let (from, to) = (
                        (first.0.min(last.0) / ux).floor().max(0.0) as usize,
                        (first.0.max(last.0) / ux).ceil().max(0.0) as usize,
                    );
                    for col in from..to {
                        let x = (col as f64 + 0.5) * ux;
                        let y_top = interpolate(&top, x);
                        let y_bottom = interpolate(&bottom, x);
                        let (a, b) = (y_top.min(y_bottom), y_top.max(y_bottom));
                        let start = ((a / uy).floor().max(rows.0 as f64)) as usize;
                        let end = ((b / uy).ceil().min(rows.1 as f64)) as usize;
                        for row in start..end {
                            covered.push((col, row));
                        }
                    }
                }
                covered.sort_unstable();
                covered.dedup();
            }
        }
    }

    /// Clip what follows to the area between `top` and `bottom` following
    /// `curve`, for a pattern's marks; nothing on the mask. Returns whether
    /// to `end_clip`.
    #[allow(unused_variables)]
    pub fn clip_between(
        &mut self,
        top: &[(f64, f64)],
        bottom: &[(f64, f64)],
        curve: Curve,
    ) -> bool {
        match self {
            Shapes::Mask(_) => false,
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => scene.clip_between(top, bottom, curve),
        }
    }

    pub fn end_clip(&mut self) {
        match self {
            Shapes::Mask(_) => {}
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => scene.end_clip(),
        }
    }

    /// A pattern's mark in the cell at (`col`, `row`) of a picture: a
    /// diagonal stroke for `/`, a dot for a dot glyph, the tile glyph
    /// itself otherwise (CHT-012). Returns false on the mask, where the
    /// mark is text.
    #[allow(unused_variables)]
    pub fn pattern_mark(&mut self, col: usize, row: usize, mark: &str, tint: Option<Rgba>) -> bool {
        match self {
            Shapes::Mask(_) => false,
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let (ux, uy) = scene.units();
                // Without any color the mark is left out of the picture.
                let Some(tint) = tint.or(scene.fallback()) else {
                    return true;
                };
                let (left, top) = (col as f64 * ux, row as f64 * uy);
                match mark {
                    " " | "" => {}
                    "/" => scene.line((left, top + uy), (left + ux, top), tint, None, 1.0),
                    "·" | "." => scene.disc(
                        left + ux / 2.0,
                        top + uy / 2.0,
                        (uy / 8.0) as f32,
                        tint,
                        1.0,
                    ),
                    tile => scene.text(left, top + uy * 0.8, uy as f32, tile, tint),
                }
                true
            }
        }
    }

    /// A one-pixel crosshair down `inner` at `x` units in `color` at `alpha`
    /// (CHT-038); nothing on the mask, whose hover is patched into cells.
    #[allow(unused_variables)]
    pub fn crosshair(&mut self, x: f64, inner: Rect, color: Option<Rgba>, alpha: f32) {
        match self {
            Shapes::Mask(_) => {}
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let (_, uy) = scene.units();
                let Some(color) = color.or(scene.fallback()) else {
                    return;
                };
                scene.line(
                    (x, inner.y as f64 * uy),
                    (x, inner.bottom() as f64 * uy),
                    color,
                    None,
                    alpha,
                );
            }
        }
    }

    /// The hovered point of a series: a disc half a cell high inside a halo
    /// a cell high at a fifth of its alpha (CHT-038); nothing on the mask.
    #[allow(unused_variables)]
    pub fn hover_dot(&mut self, x: f64, y: f64, color: Option<Rgba>, alpha: f32) {
        match self {
            Shapes::Mask(_) => {}
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let height = scene.cell_height();
                let Some(tint) = color.or(scene.fallback()) else {
                    return;
                };
                scene.disc(x, y, height / 2.0, tint, 0.2 * alpha);
                scene.disc(x, y, height / 4.0, tint, alpha);
            }
        }
    }

    /// A one-pixel ring a cell high around a scatter's selected point
    /// (CHT-038); nothing on the mask.
    #[allow(unused_variables)]
    pub fn ring(&mut self, x: f64, y: f64, color: Option<Rgba>, alpha: f32) {
        match self {
            Shapes::Mask(_) => {}
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let radius = scene.cell_height() / 2.0;
                if let Some(color) = color.or(scene.fallback()) {
                    scene.ring(x, y, radius, color, alpha);
                }
            }
        }
    }

    /// A one-pixel grid line across the plot at `at` units, vertical or
    /// horizontal, dashed when asked (CHT-037); nothing on the mask, where
    /// the grid is text. `plot` is the plot in cells.
    #[allow(unused_variables)]
    pub fn grid_line(
        &mut self,
        vertical: bool,
        at: f64,
        plot: Rect,
        dashed: bool,
        color: Option<Rgba>,
    ) {
        match self {
            Shapes::Mask(_) => {}
            #[cfg(feature = "wgpu-graphics")]
            Shapes::Pixels(scene) => {
                let (ux, uy) = scene.units();
                let dash = dashed.then(|| vec![(uy / 4.0) as f32, (uy / 4.0) as f32]);
                let (from, to) = if vertical {
                    ((at, plot.y as f64 * uy), (at, plot.bottom() as f64 * uy))
                } else {
                    ((plot.x as f64 * ux, at), (plot.right() as f64 * ux, at))
                };
                let Some(color) = color.or(scene.fallback()) else {
                    return;
                };
                scene.line(from, to, color, dash.as_deref(), 1.0);
            }
        }
    }
}

/// Fill the dots between `top` and `bottom` polylines column by column on
/// the mask: the fill color at `opacity` over `background`, and for a
/// gradient strongest at the stroke and fading to the background at the
/// baseline (CHT-012).
#[allow(clippy::too_many_arguments)]
fn fill_between(
    mask: &mut MaskCanvas,
    top: &[(f64, f64)],
    bottom: &[(f64, f64)],
    tint: Option<Rgba>,
    background: Option<Rgba>,
    opacity: f32,
    style: &FillStyle,
    owner: Option<(usize, usize)>,
    covered: &mut Vec<(usize, usize)>,
    rows: (usize, usize),
) {
    if top.is_empty() || bottom.is_empty() {
        return;
    }
    let (x0, x1) = (near(top.first().unwrap().0), near(top.last().unwrap().0));
    let mut x = x0.round() as i64;
    let end = x1.round() as i64;
    let mut ti = 0;
    let mut bi = 0;
    let (row_top, row_bottom) = (rows.0 as i64 - 1, rows.1 as i64 + 1);
    let opacity = opacity.clamp(0.0, 1.0);
    let shade_at = |t: f32| -> Option<Rgba> {
        let tint = tint?;
        match background {
            // Over a known background the fill shows `opacity` of its color,
            // less toward the baseline for a gradient.
            Some(bg) => Some(mix(bg, tint, opacity * (1.0 - t))),
            // Without one, the alpha carries the opacity.
            None => Some((tint.0, tint.1, tint.2, tint.3 * opacity * (1.0 - t))),
        }
    };
    let solid = shade_at(0.0);
    while x <= end {
        let xf = x as f64;
        while ti + 1 < top.len() && top[ti + 1].0 < xf {
            ti += 1;
        }
        while bi + 1 < bottom.len() && bottom[bi + 1].0 < xf {
            bi += 1;
        }
        let y_top = near(interpolate_at(top, ti, xf));
        let y_bottom = near(interpolate_at(bottom, bi, xf));
        let (a, b) = (y_top.min(y_bottom), y_top.max(y_bottom));
        // Only the rows the plot holds are walked (CHT-026).
        let mut y = (a.round() as i64).max(row_top);
        let last = (b.round() as i64).min(row_bottom);
        let extent = (y_bottom - y_top).abs().max(1.0);
        while y <= last {
            let shade = match style {
                FillStyle::Gradient => {
                    let t = ((y as f64 - y_top).abs() / extent).clamp(0.0, 1.0) as f32;
                    shade_at(t)
                }
                _ => solid,
            };
            mask.dot(x, y, shade, owner);
            let cell = ((x.max(0) as usize) / DOTS_X, (y.max(0) as usize) / DOTS_Y);
            if covered.last() != Some(&cell) {
                covered.push(cell);
            }
            y += 1;
        }
        x += 1;
    }
    covered.sort_unstable();
    covered.dedup();
}

/// The widest a mapped coordinate may wander outside the canvas, in dots:
/// far enough past any plot that clipping is exact, near enough that no
/// conversion overflows (CHT-026).
const FAR: f64 = 1.0e7;

/// A coordinate kept finite and within reach (CHT-026).
fn near(v: f64) -> f64 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(-FAR, FAR)
    }
}

fn interpolate_at(line: &[(f64, f64)], i: usize, x: f64) -> f64 {
    let a = line[i];
    let Some(b) = line.get(i + 1) else { return a.1 };
    let dx = b.0 - a.0;
    if dx.abs() < 1e-9 {
        return b.1;
    }
    let t = ((x - a.0) / dx).clamp(0.0, 1.0);
    a.1 + t * (b.1 - a.1)
}

/// The y of polyline `line` at `x`, from the segment that spans it.
#[cfg(feature = "wgpu-graphics")]
fn interpolate(line: &[(f64, f64)], x: f64) -> f64 {
    let Some(first) = line.first() else {
        return 0.0;
    };
    if line.len() == 1 || x <= first.0.min(line[line.len() - 1].0) {
        return if line[line.len() - 1].0 < first.0 {
            line[line.len() - 1].1
        } else {
            first.1
        };
    }
    let i = line
        .windows(2)
        .position(|pair| (pair[0].0 <= x && x <= pair[1].0) || (pair[1].0 <= x && x <= pair[0].0))
        .unwrap_or(line.len() - 2);
    interpolate_at(line, i, x)
}
