//! The shared mask canvas every chart type rasterizes through (CHT-025).
//!
//! Shapes are written as per-dot membership at two by four dots per cell,
//! each dot carrying a color. Axis-aligned rectangles also record the exact
//! fraction of each edge cell they cover, which the dot grid alone cannot
//! carry. The canvas alone resolves each cell to a glyph and a color:
//!
//! - a full block when every dot is covered,
//! - an eighth block when one rectangle edge crosses the cell at a multiple
//!   of one eighth and the rest of the cell is empty,
//! - a marker glyph when the cell holds only a marker,
//! - braille otherwise.
//!
//! Filled shapes (pie and donut sectors, radar fills) are sampled instead at
//! the pixel grid of the blitter the canvas was given, each sample keeping
//! its own color. A cell that holds fill and no dot, edge or marker resolves
//! to that blitter's glyph with the two-color split `blit_block` chooses
//! (docs/spec/blitters.md, BLT-001), an uncovered sample counting as
//! transparent.
//!
//! With the ASCII glyph set (CHT-028) the same coverage resolves to `#`, `|`,
//! `-` and `.` instead, with the same geometry; fills then sample the two by
//! four dot grid and resolve the same way.

use super::plot::Rgba;
use ::suprtui::blit::{blit_block, Blitter};

/// Dots per cell horizontally.
pub const DOTS_X: usize = 2;
/// Dots per cell vertically.
pub const DOTS_Y: usize = 4;

/// Which glyphs the canvas may emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlyphSet {
    /// Braille, block and eighth-block characters.
    #[default]
    Unicode,
    /// `#`, `|`, `-` and `.` only, for terminals without block or braille
    /// glyphs or when the builder forces ASCII.
    Ascii,
}

/// The marker glyph size for scatter and line dots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marker {
    /// A small dot (`•`), the scatter marker.
    Dot,
    /// A larger disc (`●`), the line-chart point dot.
    Disc,
}

/// A rectangle edge that crossed a cell at a fraction of its height (a
/// vertical bar's tip) or width (a horizontal bar's tip).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Edge {
    /// The rectangle covers the bottom `fraction` of the cell.
    Bottom(f64),
    /// The rectangle covers the top `fraction` of the cell.
    Top(f64),
    /// The rectangle covers the left `fraction` of the cell.
    Left(f64),
    /// The rectangle covers the right `fraction` of the cell.
    Right(f64),
}

#[derive(Debug, Clone, Default)]
struct Cell {
    /// Bit `i` set when dot `i` (row-major, two per row) is covered.
    dots: u8,
    color: Option<Rgba>,
    edge: Option<Edge>,
    marker: Option<Marker>,
    /// Which (series, point) painted this cell, for hit testing.
    owner: Option<(usize, usize)>,
}

/// A resolved cell.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    /// The glyph, or `None` for an empty cell.
    pub glyph: Option<&'static str>,
    /// The glyph color.
    pub color: Option<Rgba>,
    /// The background under the glyph: a fill cell's second color, or
    /// `None` to show what is below the chart.
    pub background: Option<Rgba>,
    /// The (series, point) that painted the cell.
    pub owner: Option<(usize, usize)>,
}

/// One cell's fill samples: an index into the canvas's fill palette for each
/// pixel of the fill blitter's grid in reading order, 0 where uncovered.
type FillSamples = [u16; 8];

/// A fill palette entry: the fill's color and the (series, point) it draws.
type FillEntry = (Option<Rgba>, Option<(usize, usize)>);

/// The mask canvas over a `cols` by `rows` cell grid.
#[derive(Debug, Clone)]
pub struct MaskCanvas {
    cols: usize,
    rows: usize,
    cells: Vec<Cell>,
    /// Dot-space clip: (left, top, right, bottom), exclusive on the far side.
    clip: (i64, i64, i64, i64),
    /// The blitter fill-only cells resolve through (CHT-025).
    fill_blitter: Blitter,
    /// Fill samples per cell, allocated at the first fill.
    fills: Vec<FillSamples>,
    /// The color and owner of each fill palette index, from index 1.
    fill_palette: Vec<FillEntry>,
}

/// A fill sample's color as an opaque pixel for the blitter.
fn fill_pixel((r, g, b, _): Rgba) -> [u8; 4] {
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    [channel(r), channel(g), channel(b), u8::MAX]
}

fn pixel_color(pixel: [u8; 3]) -> Rgba {
    let channel = |v: u8| f32::from(v) / 255.0;
    (channel(pixel[0]), channel(pixel[1]), channel(pixel[2]), 1.0)
}

/// The glyph a blitter draws for a pattern, as a static string.
fn blit_glyph(blitter: Blitter, pattern: u8) -> &'static str {
    use std::sync::OnceLock;
    static TABLES: OnceLock<Vec<Vec<&'static str>>> = OnceLock::new();
    let tables = TABLES.get_or_init(|| {
        Blitter::TIERS
            .iter()
            .map(|tier| {
                (0..=255u8)
                    .map(|pattern| {
                        let glyph: &'static str =
                            Box::leak(tier.glyph(pattern).to_string().into_boxed_str());
                        glyph
                    })
                    .collect()
            })
            .collect()
    });
    let tier = Blitter::TIERS
        .iter()
        .position(|tier| *tier == blitter)
        .unwrap_or(0);
    tables[tier][usize::from(pattern)]
}

const BRAILLE_BITS: [[u8; DOTS_X]; DOTS_Y] =
    [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];
const LOWER_EIGHTHS: [&str; 8] = ["", "▁", "▂", "▃", "▄", "▅", "▆", "▇"];
const UPPER_EIGHTHS: [&str; 8] = ["", "▔", "🮂", "🮃", "▀", "🮄", "🮅", "🮆"];
const LEFT_EIGHTHS: [&str; 8] = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"];
const RIGHT_EIGHTHS: [&str; 8] = ["", "▕", "🮇", "🮈", "▐", "🮉", "🮊", "🮋"];
const FULL: &str = "█";
const FULL_DOTS: u8 = 0xFF;

/// Braille glyph for a dot bitmask (row-major, two dots per row).
fn braille(dots: u8) -> &'static str {
    // 256 static strings: build once.
    use std::sync::OnceLock;
    static TABLE: OnceLock<Vec<&'static str>> = OnceLock::new();
    let table = TABLE.get_or_init(|| {
        (0..=255u8)
            .map(|mask| {
                let mut code = 0u32;
                for (row, bits) in BRAILLE_BITS.iter().enumerate() {
                    for (col, bit) in bits.iter().enumerate() {
                        if mask & (1 << (row * DOTS_X + col)) != 0 {
                            code |= *bit as u32;
                        }
                    }
                }
                let s: String = char::from_u32(0x2800 + code).unwrap_or(' ').to_string();
                Box::leak(s.into_boxed_str()) as &'static str
            })
            .collect()
    });
    table[dots as usize]
}

fn eighths(fraction: f64) -> Option<usize> {
    let scaled = fraction * 8.0;
    let n = scaled.round();
    ((scaled - n).abs() < 1e-6 && (1.0..8.0).contains(&n)).then_some(n as usize)
}

impl MaskCanvas {
    /// An empty canvas over `cols` by `rows` cells.
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![Cell::default(); cols.saturating_mul(rows)],
            clip: (0, 0, (cols * DOTS_X) as i64, (rows * DOTS_Y) as i64),
            fill_blitter: Blitter::Sextant,
            fills: Vec::new(),
            fill_palette: Vec::new(),
        }
    }

    /// Resolve fill-only cells through `blitter` and sample later fills at
    /// its pixel grid (CHT-025). Fills already written keep their samples,
    /// so set this before the first fill.
    pub fn set_fill_blitter(&mut self, blitter: Blitter) {
        self.fill_blitter = blitter;
    }

    /// The blitter fill-only cells resolve through.
    pub fn fill_blitter(&self) -> Blitter {
        self.fill_blitter
    }

    /// Restrict every later shape to the cell rectangle from (`col`, `row`)
    /// spanning `w` by `h` cells, so shapes never spill over axes or labels.
    pub fn set_clip_cells(&mut self, col: usize, row: usize, w: usize, h: usize) {
        self.clip = (
            (col * DOTS_X) as i64,
            (row * DOTS_Y) as i64,
            ((col + w) * DOTS_X) as i64,
            ((row + h) * DOTS_Y) as i64,
        );
    }

    fn inside(&self, x: i64, y: i64) -> bool {
        x >= self.clip.0 && x < self.clip.2 && y >= self.clip.1 && y < self.clip.3
    }

    /// Width in cells.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Height in cells.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Width in dots.
    pub fn width(&self) -> f64 {
        (self.cols * DOTS_X) as f64
    }

    /// Height in dots.
    pub fn height(&self) -> f64 {
        (self.rows * DOTS_Y) as f64
    }

    fn cell_mut(&mut self, col: usize, row: usize) -> Option<&mut Cell> {
        (col < self.cols && row < self.rows).then(|| &mut self.cells[row * self.cols + col])
    }

    /// Cover one dot at dot coordinates (`x`, `y`).
    pub fn dot(&mut self, x: i64, y: i64, color: Option<Rgba>, owner: Option<(usize, usize)>) {
        if !self.inside(x, y) {
            return;
        }
        let (col, row) = (x as usize / DOTS_X, y as usize / DOTS_Y);
        let bit = 1u8 << ((y as usize % DOTS_Y) * DOTS_X + x as usize % DOTS_X);
        if let Some(cell) = self.cell_mut(col, row) {
            cell.dots |= bit;
            cell.color = color.or(cell.color);
            cell.owner = owner.or(cell.owner);
        }
    }

    /// Fill the axis-aligned rectangle from (`x0`, `y0`) to (`x1`, `y1`) in
    /// dot units (any order), recording the exact fraction on the edge cells
    /// so a bar tip resolves to an eighth block.
    pub fn rect(
        &mut self,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        let (left, right) = (
            x0.min(x1).max(self.clip.0 as f64),
            x0.max(x1).min(self.clip.2 as f64),
        );
        let (top, bottom) = (
            y0.min(y1).max(self.clip.1 as f64),
            y0.max(y1).min(self.clip.3 as f64),
        );
        if right <= left || bottom <= top {
            return;
        }
        let first_col = (left / DOTS_X as f64).floor() as usize;
        let last_col = ((right / DOTS_X as f64).ceil() as usize).min(self.cols);
        let first_row = (top / DOTS_Y as f64).floor() as usize;
        let last_row = ((bottom / DOTS_Y as f64).ceil() as usize).min(self.rows);
        for row in first_row..last_row {
            let cell_top = (row * DOTS_Y) as f64;
            let cell_bottom = cell_top + DOTS_Y as f64;
            let cover_top = top.max(cell_top);
            let cover_bottom = bottom.min(cell_bottom);
            let vertical = (cover_bottom - cover_top) / DOTS_Y as f64;
            for col in first_col..last_col {
                let cell_left = (col * DOTS_X) as f64;
                let cell_right = cell_left + DOTS_X as f64;
                let cover_left = left.max(cell_left);
                let cover_right = right.min(cell_right);
                let horizontal = (cover_right - cover_left) / DOTS_X as f64;
                if vertical <= 1e-9 || horizontal <= 1e-9 {
                    continue;
                }
                let full_h = horizontal >= 1.0 - 1e-9;
                let full_v = vertical >= 1.0 - 1e-9;
                let edge = if full_h && !full_v {
                    if cover_bottom >= cell_bottom - 1e-9 {
                        Some(Edge::Bottom(vertical))
                    } else if cover_top <= cell_top + 1e-9 {
                        Some(Edge::Top(vertical))
                    } else {
                        None
                    }
                } else if full_v && !full_h {
                    if cover_left <= cell_left + 1e-9 {
                        Some(Edge::Left(horizontal))
                    } else if cover_right >= cell_right - 1e-9 {
                        Some(Edge::Right(horizontal))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let Some(cell) = self.cell_mut(col, row) else {
                    continue;
                };
                let had_dots = cell.dots != 0;
                for dy in 0..DOTS_Y {
                    let dot_top = cell_top + dy as f64;
                    if dot_top + 0.5 < cover_top || dot_top + 0.5 > cover_bottom {
                        continue;
                    }
                    for dx in 0..DOTS_X {
                        let dot_left = cell_left + dx as f64;
                        if dot_left + 0.5 < cover_left || dot_left + 0.5 > cover_right {
                            continue;
                        }
                        cell.dots |= 1 << (dy * DOTS_X + dx);
                    }
                }
                cell.edge = match (had_dots, edge) {
                    (false, e) => e,
                    (true, _) => None,
                };
                cell.color = color.or(cell.color);
                cell.owner = owner.or(cell.owner);
            }
        }
    }

    /// Stroke the polyline `points` (dot units) one dot wide.
    pub fn polyline(
        &mut self,
        points: &[(f64, f64)],
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        if points.len() == 1 {
            self.dot(
                points[0].0.round() as i64,
                points[0].1.round() as i64,
                color,
                owner,
            );
        }
        for pair in points.windows(2) {
            self.line(pair[0], pair[1], color, owner);
        }
    }

    /// Stroke one segment (dot units) with Bresenham steps.
    pub fn line(
        &mut self,
        a: (f64, f64),
        b: (f64, f64),
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        let (x0, y0) = (a.0.round() as i64, a.1.round() as i64);
        let (x1, y1) = (b.0.round() as i64, b.1.round() as i64);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        let limit = (dx - dy) as usize + 2;
        for _ in 0..limit.min(1 << 20) {
            self.dot(x, y, color, owner);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Fill the polygon `points` (dot units) with even-odd scanlines.
    pub fn polygon(
        &mut self,
        points: &[(f64, f64)],
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        if points.len() < 3 {
            return;
        }
        let top = points
            .iter()
            .map(|p| p.1)
            .fold(f64::INFINITY, f64::min)
            .max(0.0);
        let bottom = points
            .iter()
            .map(|p| p.1)
            .fold(f64::NEG_INFINITY, f64::max)
            .min(self.height());
        if (bottom - top).is_nan() || bottom <= top {
            return;
        }
        let mut crossings = Vec::new();
        let mut y = top.floor() as i64;
        while (y as f64) < bottom {
            let scan = y as f64 + 0.5;
            crossings.clear();
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                if (a.1 <= scan) != (b.1 <= scan) {
                    let t = (scan - a.1) / (b.1 - a.1);
                    crossings.push(a.0 + t * (b.0 - a.0));
                }
            }
            crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for pair in crossings.chunks(2) {
                if let [l, r] = pair {
                    let (start, end) = (l.round() as i64, r.round() as i64);
                    for x in start.max(0)..end.min(self.width() as i64) {
                        self.dot(x, y, color, owner);
                    }
                }
            }
            y += 1;
        }
    }

    /// Cover the dots of the annular sector centred on (`cx`, `cy`) between
    /// the `inner` and `outer` radii and the angles `start` to `end` in
    /// radians, clockwise from twelve o'clock, all in dot units. A dot is as
    /// wide as it is tall on screen, so a full sector is a circle twice as
    /// many columns wide as it is rows tall. Charts fill sectors with
    /// [`MaskCanvas::fill_sector`]; this covers dots like a stroke.
    #[allow(clippy::too_many_arguments)]
    pub fn sector(
        &mut self,
        cx: f64,
        cy: f64,
        inner: f64,
        outer: f64,
        start: f64,
        end: f64,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        if (end - start).is_nan() || end <= start || outer <= 0.0 {
            return;
        }
        let x0 = ((cx - outer).floor().max(0.0)) as i64;
        let x1 = ((cx + outer).ceil().min(self.width())) as i64;
        let y0 = ((cy - outer).floor().max(0.0)) as i64;
        let y1 = ((cy + outer).ceil().min(self.height())) as i64;
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = x as f64 + 0.5 - cx;
                let dy = y as f64 + 0.5 - cy;
                let r = (dx * dx + dy * dy).sqrt();
                if r > outer || r < inner {
                    continue;
                }
                let angle = dx.atan2(-dy).rem_euclid(std::f64::consts::TAU);
                if angle >= start && angle < end {
                    self.dot(x, y, color, owner);
                }
            }
        }
    }

    fn fill_index(&mut self, color: Option<Rgba>, owner: Option<(usize, usize)>) -> u16 {
        if let Some(i) = self
            .fill_palette
            .iter()
            .position(|entry| *entry == (color, owner))
        {
            return (i + 1) as u16;
        }
        if self.fill_palette.len() >= usize::from(u16::MAX) - 1 {
            return u16::MAX - 1;
        }
        self.fill_palette.push((color, owner));
        self.fill_palette.len() as u16
    }

    /// Fill every sample whose center lies inside the dot-space box (`x0`,
    /// `y0`) to (`x1`, `y1`), from the first edges up to but not including
    /// the second, and inside the clip, for which `inside` holds at the
    /// center, in dot units. A box edge inside a cell leaves that cell partly
    /// covered. A later fill covers an earlier one where both hold. A fill needs a
    /// color to split on, so a fill with none draws nothing. Returns the
    /// number of samples it set.
    pub fn fill_where(
        &mut self,
        (x0, y0, x1, y1): (f64, f64, f64, f64),
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
        inside: impl Fn(f64, f64) -> bool,
    ) -> usize {
        if color.is_none()
            || [x0, y0, x1, y1].iter().any(|v| v.is_nan())
            || self.cols == 0
            || self.rows == 0
        {
            return 0;
        }
        let (left, top, right, bottom) = (
            self.clip.0 as f64,
            self.clip.1 as f64,
            self.clip.2 as f64,
            self.clip.3 as f64,
        );
        let (x0, x1) = (x0.max(left), x1.min(right));
        let (y0, y1) = (y0.max(top), y1.min(bottom));
        if x1 <= x0 || y1 <= y0 {
            return 0;
        }
        if self.fills.is_empty() {
            self.fills = vec![[0; 8]; self.cols * self.rows];
        }
        let index = self.fill_index(color, owner);
        let (pw, ph) = self.fill_blitter.cell_pixels();
        let (pw, ph) = (pw as usize, ph as usize);
        let c0 = (x0 / DOTS_X as f64).floor().max(0.0) as usize;
        let c1 = ((x1 / DOTS_X as f64).ceil() as usize).min(self.cols);
        let r0 = (y0 / DOTS_Y as f64).floor().max(0.0) as usize;
        let r1 = ((y1 / DOTS_Y as f64).ceil() as usize).min(self.rows);
        let mut set = 0;
        for row in r0..r1 {
            for col in c0..c1 {
                for py in 0..ph {
                    let y = (row as f64 + (py as f64 + 0.5) / ph as f64) * DOTS_Y as f64;
                    // The box is already cut to the clip.
                    if y < y0 || y >= y1 {
                        continue;
                    }
                    for px in 0..pw {
                        let x = (col as f64 + (px as f64 + 0.5) / pw as f64) * DOTS_X as f64;
                        if x >= x0 && x < x1 && inside(x, y) {
                            self.fills[row * self.cols + col][py * pw + px] = index;
                            set += 1;
                        }
                    }
                }
            }
        }
        set
    }

    /// Fill the annular sector centred on (`cx`, `cy`) between the `inner`
    /// and `outer` radii and the angles `start` to `end` in radians,
    /// clockwise from twelve o'clock. Everything is in dot units, and a dot
    /// is as wide as it is tall on screen, so a full sector is a circle.
    /// Returns the number of samples it set, 0 when the sector covers no
    /// sample's center.
    #[allow(clippy::too_many_arguments)]
    pub fn fill_sector(
        &mut self,
        cx: f64,
        cy: f64,
        inner: f64,
        outer: f64,
        start: f64,
        end: f64,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) -> usize {
        if (end - start).is_nan() || end <= start || outer <= 0.0 {
            return 0;
        }
        self.fill_where(
            (cx - outer, cy - outer, cx + outer, cy + outer),
            color,
            owner,
            |x, y| {
                let (dx, dy) = (x - cx, y - cy);
                let r = dx.hypot(dy);
                if r > outer || r < inner {
                    return false;
                }
                let angle = dx.atan2(-dy).rem_euclid(std::f64::consts::TAU);
                angle >= start && angle < end
            },
        )
    }

    /// Fill the polygon `points` (dot units) by the even-odd rule.
    pub fn fill_polygon(
        &mut self,
        points: &[(f64, f64)],
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        if points.len() < 3 {
            return;
        }
        let bound = |f: fn(f64, f64) -> f64, init: f64, pick: fn(&(f64, f64)) -> f64| {
            points.iter().map(pick).fold(init, f)
        };
        let bounds = (
            bound(f64::min, f64::INFINITY, |p| p.0),
            bound(f64::min, f64::INFINITY, |p| p.1),
            bound(f64::max, f64::NEG_INFINITY, |p| p.0),
            bound(f64::max, f64::NEG_INFINITY, |p| p.1),
        );
        self.fill_where(bounds, color, owner, |x, y| inside_polygon(points, x, y));
    }

    /// Clear the stroke dots, and the markers of cells, whose centers lie
    /// inside the polygon `points` (dot units), so a shape drawn next lies
    /// over the strokes before it.
    pub fn clear_dots_inside(&mut self, points: &[(f64, f64)]) {
        if points.len() < 3 {
            return;
        }
        let (mut x0, mut y0, mut x1, mut y1) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in points {
            x0 = x0.min(p.0);
            y0 = y0.min(p.1);
            x1 = x1.max(p.0);
            y1 = y1.max(p.1);
        }
        if [x0, y0, x1, y1].iter().any(|v| !v.is_finite()) {
            return;
        }
        let x0 = x0.floor().max(0.0) as usize;
        let y0 = y0.floor().max(0.0) as usize;
        let x1 = (x1.ceil().max(0.0) as usize).min(self.cols * DOTS_X);
        let y1 = (y1.ceil().max(0.0) as usize).min(self.rows * DOTS_Y);
        for y in y0..y1 {
            for x in x0..x1 {
                if !inside_polygon(points, x as f64 + 0.5, y as f64 + 0.5) {
                    continue;
                }
                let (col, row) = (x / DOTS_X, y / DOTS_Y);
                let cell_center = (
                    (col as f64 + 0.5) * DOTS_X as f64,
                    (row as f64 + 0.5) * DOTS_Y as f64,
                );
                let marker_covered = inside_polygon(points, cell_center.0, cell_center.1);
                if let Some(cell) = self.cell_mut(col, row) {
                    cell.dots &= !(1u8 << ((y % DOTS_Y) * DOTS_X + x % DOTS_X));
                    if marker_covered {
                        cell.marker = None;
                    }
                    if cell.dots == 0 && cell.marker.is_none() && cell.edge.is_none() {
                        cell.color = None;
                        cell.owner = None;
                    }
                }
            }
        }
    }

    /// The fill samples of the cell at (`col`, `row`), when any is covered.
    fn fill_at(&self, col: usize, row: usize) -> Option<&FillSamples> {
        self.fills
            .get(row * self.cols + col)
            .filter(|samples| samples.iter().any(|index| *index != 0))
    }

    /// The fill palette index covering most samples of `samples`.
    fn main_fill(samples: &FillSamples) -> u16 {
        let mut best = (0u16, 0usize);
        for index in samples.iter().copied().filter(|index| *index != 0) {
            let count = samples.iter().filter(|other| **other == index).count();
            if count > best.1 {
                best = (index, count);
            }
        }
        best.0
    }

    fn fill_entry(&self, index: u16) -> FillEntry {
        usize::from(index)
            .checked_sub(1)
            .and_then(|i| self.fill_palette.get(i))
            .copied()
            .unwrap_or((None, None))
    }

    /// Resolve a fill-only cell: the blitter's glyph with its two-color
    /// split, or with ASCII glyphs the dot rule over the samples.
    fn resolve_fill(&self, samples: &FillSamples, glyphs: GlyphSet) -> Resolved {
        let owner = self.fill_entry(Self::main_fill(samples)).1;
        let (pw, ph) = self.fill_blitter.cell_pixels();
        let count = (pw * ph) as usize;
        if glyphs == GlyphSet::Ascii {
            let mut dots = 0u8;
            if (pw, ph) == (DOTS_X as u32, DOTS_Y as u32) {
                for (i, index) in samples.iter().take(count).enumerate() {
                    if *index != 0 {
                        dots |= 1 << i;
                    }
                }
            } else {
                dots = FULL_DOTS;
            }
            return Resolved {
                glyph: Some(if dots == FULL_DOTS {
                    "#"
                } else {
                    ascii_for(dots)
                }),
                color: self.fill_entry(Self::main_fill(samples)).0,
                background: None,
                owner,
            };
        }
        let mut pixels = [[0u8; 4]; 8];
        for (pixel, index) in pixels.iter_mut().zip(samples.iter()).take(count) {
            if let Some(color) = self.fill_entry(*index).0.filter(|_| *index != 0) {
                *pixel = fill_pixel(color);
            }
        }
        let cell = blit_block(self.fill_blitter, &pixels[..count]);
        // A cell of one color splits with no error whichever set holds its
        // samples. `blit_block` leaves them clear and paints the background;
        // the set pattern draws the same cell as the blitter's full glyph in
        // the foreground, as a fully covered stroke or bar cell is drawn.
        // The ASCII tier's full glyph is a space, so it keeps the background.
        if cell.pattern == 0 && cell.bg.is_some() && self.fill_blitter != Blitter::Ascii {
            let full = if count == 8 {
                u8::MAX
            } else {
                (1u8 << count) - 1
            };
            return Resolved {
                glyph: Some(blit_glyph(self.fill_blitter, full)),
                color: cell.bg.map(pixel_color),
                background: None,
                owner,
            };
        }
        Resolved {
            glyph: Some(blit_glyph(self.fill_blitter, cell.pattern)),
            color: cell.fg.map(pixel_color),
            background: cell.bg.map(pixel_color),
            owner,
        }
    }

    /// Place a marker in the cell containing dot (`x`, `y`).
    pub fn marker(
        &mut self,
        x: f64,
        y: f64,
        marker: Marker,
        color: Option<Rgba>,
        owner: Option<(usize, usize)>,
    ) {
        if !self.inside(x.floor() as i64, y.floor() as i64) {
            return;
        }
        let (col, row) = ((x / DOTS_X as f64) as usize, (y / DOTS_Y as f64) as usize);
        if let Some(cell) = self.cell_mut(col, row) {
            cell.marker = Some(marker);
            cell.color = color.or(cell.color);
            cell.owner = owner.or(cell.owner);
        }
    }

    /// Whether the cell at (`col`, `row`) holds any shape.
    pub fn is_painted(&self, col: usize, row: usize) -> bool {
        col < self.cols
            && (self
                .cells
                .get(row * self.cols + col)
                .is_some_and(|c| c.dots != 0 || c.marker.is_some())
                || self.fill_at(col, row).is_some())
    }

    /// The (series, point) that painted the cell, if any: a stroke's or
    /// marker's owner, else the owner of most of the cell's fill.
    pub fn owner_at(&self, col: usize, row: usize) -> Option<(usize, usize)> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        self.cells[row * self.cols + col].owner.or_else(|| {
            self.fill_at(col, row)
                .and_then(|samples| self.fill_entry(Self::main_fill(samples)).1)
        })
    }

    /// Resolve the cell at (`col`, `row`) to a glyph and color.
    pub fn resolve(&self, col: usize, row: usize, glyphs: GlyphSet) -> Resolved {
        let Some(cell) =
            (col < self.cols && row < self.rows).then(|| &self.cells[row * self.cols + col])
        else {
            return Resolved {
                glyph: None,
                color: None,
                background: None,
                owner: None,
            };
        };
        // A stroke, marker or bar edge draws over fill; a cell with fill
        // alone takes the blitter's split (CHT-025).
        if cell.dots == 0 && cell.edge.is_none() && cell.marker.is_none() {
            if let Some(samples) = self.fill_at(col, row) {
                return self.resolve_fill(samples, glyphs);
            }
        }
        let glyph = if cell.dots == FULL_DOTS {
            Some(match glyphs {
                GlyphSet::Unicode => FULL,
                GlyphSet::Ascii => "#",
            })
        } else if let Some(edge) = cell.edge {
            match glyphs {
                GlyphSet::Unicode => match edge {
                    Edge::Bottom(f) => eighths(f).map(|n| LOWER_EIGHTHS[n]),
                    Edge::Top(f) => eighths(f).map(|n| UPPER_EIGHTHS[n]),
                    Edge::Left(f) => eighths(f).map(|n| LEFT_EIGHTHS[n]),
                    Edge::Right(f) => eighths(f).map(|n| RIGHT_EIGHTHS[n]),
                }
                .or_else(|| (cell.dots != 0).then(|| braille(cell.dots))),
                GlyphSet::Ascii => Some(match edge {
                    Edge::Bottom(_) | Edge::Top(_) => "-",
                    Edge::Left(_) | Edge::Right(_) => "|",
                }),
            }
        } else if let Some(marker) = cell.marker {
            Some(match (glyphs, marker) {
                (GlyphSet::Unicode, Marker::Dot) => "•",
                (GlyphSet::Unicode, Marker::Disc) => "●",
                (GlyphSet::Ascii, _) => "o",
            })
        } else if cell.dots != 0 {
            Some(match glyphs {
                GlyphSet::Unicode => braille(cell.dots),
                GlyphSet::Ascii => ascii_for(cell.dots),
            })
        } else {
            None
        };
        Resolved {
            glyph,
            color: cell.color,
            background: None,
            owner: cell.owner,
        }
    }
}

/// Whether (`x`, `y`) lies inside the polygon `points` by the even-odd rule.
fn inside_polygon(points: &[(f64, f64)], x: f64, y: f64) -> bool {
    let mut inside = false;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        if (a.1 > y) != (b.1 > y) && x < a.0 + (y - a.1) / (b.1 - a.1) * (b.0 - a.0) {
            inside = !inside;
        }
    }
    inside
}

/// ASCII stand-in for a partial dot pattern: a column of dots is `|`, a row
/// of dots is `-`, anything else is `.`.
fn ascii_for(dots: u8) -> &'static str {
    let rows: Vec<u8> = (0..DOTS_Y).map(|r| (dots >> (r * DOTS_X)) & 0b11).collect();
    let filled_rows = rows.iter().filter(|r| **r != 0).count();
    let left = rows.iter().all(|r| *r == 0 || *r == 0b01);
    let right = rows.iter().all(|r| *r == 0 || *r == 0b10);
    if filled_rows >= 3 && (left || right) {
        "|"
    } else if filled_rows == 1 && rows.contains(&0b11) {
        "-"
    } else {
        "."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glyph(canvas: &MaskCanvas, col: usize, row: usize) -> Option<&'static str> {
        canvas.resolve(col, row, GlyphSet::Unicode).glyph
    }

    #[test]
    fn full_coverage_is_a_block_and_partial_is_braille() {
        let mut canvas = MaskCanvas::new(2, 2);
        canvas.rect(0.0, 0.0, 2.0, 4.0, None, None);
        assert_eq!(glyph(&canvas, 0, 0), Some("█"));
        let mut canvas = MaskCanvas::new(2, 2);
        canvas.dot(0, 0, None, None);
        canvas.dot(1, 3, None, None);
        let g = glyph(&canvas, 0, 0).unwrap();
        assert!(
            ('\u{2800}'..='\u{28FF}').contains(&g.chars().next().unwrap()),
            "{g}"
        );
        assert_eq!(glyph(&canvas, 1, 1), None);
    }

    #[test]
    fn rectangle_edges_resolve_to_exact_eighths() {
        // A bar covering the bottom 3/8 of the single cell.
        let mut canvas = MaskCanvas::new(1, 1);
        canvas.rect(0.0, 4.0 - 1.5, 2.0, 4.0, None, None);
        assert_eq!(glyph(&canvas, 0, 0), Some("▃"));
        let mut canvas = MaskCanvas::new(1, 1);
        canvas.rect(0.0, 0.0, 2.0, 2.0, None, None);
        assert_eq!(glyph(&canvas, 0, 0), Some("▀"));
        let mut canvas = MaskCanvas::new(1, 1);
        canvas.rect(0.0, 0.0, 0.5, 4.0, None, None);
        assert_eq!(glyph(&canvas, 0, 0), Some("▎"));
        let mut canvas = MaskCanvas::new(1, 1);
        canvas.rect(1.25, 0.0, 2.0, 4.0, None, None);
        assert_eq!(glyph(&canvas, 0, 0), Some("🮈"));
        // Not a multiple of one eighth: braille carries the coverage.
        let mut canvas = MaskCanvas::new(1, 1);
        canvas.rect(0.0, 4.0 - 1.3, 2.0, 4.0, None, None);
        let g = glyph(&canvas, 0, 0).unwrap();
        assert!(
            g.starts_with('\u{2800}')
                || ('\u{2800}'..='\u{28FF}').contains(&g.chars().next().unwrap())
        );
    }

    #[test]
    fn a_tall_bar_spans_full_cells_and_one_tip_cell() {
        let mut canvas = MaskCanvas::new(1, 4);
        // 8 rows of 4 dots = 16 dots tall canvas; bar top at 3.5 cells → covers 0.5 of row 0.
        canvas.rect(0.0, 16.0 - 14.0, 2.0, 16.0, None, None);
        assert_eq!(glyph(&canvas, 0, 3), Some("█"));
        assert_eq!(glyph(&canvas, 0, 1), Some("█"));
        assert_eq!(glyph(&canvas, 0, 0), Some("▄"));
    }

    #[test]
    fn lines_polygons_sectors_and_markers_cover_dots() {
        let mut canvas = MaskCanvas::new(4, 2);
        canvas.polyline(&[(0.0, 0.0), (7.0, 7.0)], None, None);
        assert!(canvas.is_painted(0, 0) && canvas.is_painted(3, 1));
        let mut canvas = MaskCanvas::new(4, 2);
        canvas.polygon(
            &[(0.0, 0.0), (8.0, 0.0), (8.0, 8.0), (0.0, 8.0)],
            None,
            None,
        );
        assert!(
            (0..4).all(|c| glyph(&canvas, c, 0) == Some("█") && glyph(&canvas, c, 1) == Some("█"))
        );
        let mut canvas = MaskCanvas::new(4, 2);
        canvas.sector(4.0, 4.0, 0.0, 4.0, 0.0, std::f64::consts::TAU, None, None);
        assert!(canvas.is_painted(1, 0) && canvas.is_painted(2, 1));
        let mut canvas = MaskCanvas::new(4, 2);
        canvas.marker(3.0, 5.0, Marker::Dot, None, Some((0, 1)));
        assert_eq!(glyph(&canvas, 1, 1), Some("•"));
        assert_eq!(canvas.owner_at(1, 1), Some((0, 1)));
        canvas.marker(3.0, 1.0, Marker::Disc, None, None);
        assert_eq!(glyph(&canvas, 1, 0), Some("●"));
    }

    #[test]
    fn ascii_set_never_emits_unicode() {
        let mut canvas = MaskCanvas::new(3, 1);
        canvas.rect(0.0, 0.0, 2.0, 4.0, None, None);
        canvas.rect(2.0, 2.5, 4.0, 4.0, None, None);
        canvas.dot(4, 1, None, None);
        for col in 0..3 {
            let g = canvas.resolve(col, 0, GlyphSet::Ascii).glyph.unwrap();
            assert!(g.is_ascii(), "{g}");
        }
        assert_eq!(canvas.resolve(0, 0, GlyphSet::Ascii).glyph, Some("#"));
        assert_eq!(canvas.resolve(1, 0, GlyphSet::Ascii).glyph, Some("-"));
        assert_eq!(canvas.resolve(2, 0, GlyphSet::Ascii).glyph, Some("."));
    }
}
