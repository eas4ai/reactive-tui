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
//! With the ASCII glyph set (CHT-028) the same coverage resolves to `#`, `|`,
//! `-` and `.` instead, with the same geometry.

use super::plot::Rgba;

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
    /// The (series, point) that painted the cell.
    pub owner: Option<(usize, usize)>,
}

/// The mask canvas over a `cols` by `rows` cell grid.
#[derive(Debug, Clone)]
pub struct MaskCanvas {
    cols: usize,
    rows: usize,
    cells: Vec<Cell>,
    /// Dot-space clip: (left, top, right, bottom), exclusive on the far side.
    clip: (i64, i64, i64, i64),
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
        }
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

    /// Fill the annular sector centred on (`cx`, `cy`) in dot units between
    /// `inner` and `outer` radii (in dot rows) and the angles `start` to
    /// `end` in radians measured clockwise from twelve o'clock. Dots are
    /// tested against the sector in a space where one cell is square, so a
    /// full circle comes out as wide as it is tall on screen (CHT-015).
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
        let aspect = DOTS_Y as f64 / DOTS_X as f64;
        let reach_x = outer / aspect + 1.0;
        let x0 = ((cx - reach_x).floor().max(0.0)) as i64;
        let x1 = ((cx + reach_x).ceil().min(self.width())) as i64;
        let y0 = ((cy - outer).floor().max(0.0)) as i64;
        let y1 = ((cy + outer).ceil().min(self.height())) as i64;
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = (x as f64 + 0.5 - cx) * aspect;
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
        self.cells
            .get(row * self.cols + col)
            .is_some_and(|c| c.dots != 0 || c.marker.is_some())
    }

    /// The (series, point) that painted the cell, if any.
    pub fn owner_at(&self, col: usize, row: usize) -> Option<(usize, usize)> {
        if col >= self.cols || row >= self.rows {
            return None;
        }
        self.cells[row * self.cols + col].owner
    }

    /// Resolve the cell at (`col`, `row`) to a glyph and color.
    pub fn resolve(&self, col: usize, row: usize, glyphs: GlyphSet) -> Resolved {
        let Some(cell) =
            (col < self.cols && row < self.rows).then(|| &self.cells[row * self.cols + col])
        else {
            return Resolved {
                glyph: None,
                color: None,
                owner: None,
            };
        };
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
            owner: cell.owner,
        }
    }
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
