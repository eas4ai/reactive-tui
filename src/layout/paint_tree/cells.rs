//! A prepared grid of glyphs and colors that one element paints in a single
//! step. Widgets that rasterize off the main thread (charts, canvases) fill
//! a [`CellGrid`](crate::layout::CellGrid) on their worker and attach it to one element with
//! [`crate::component::Element::with_cells`]; the frame painter then blits
//! the grid at the element's position instead of laying out one text node
//! per colored run.

use std::collections::HashMap;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

/// A color as red, green, blue and alpha in `0.0..=1.0`.
pub type Rgba = (f32, f32, f32, f32);

/// One cell of a [`CellGrid`]: an interned glyph, a packed foreground and
/// a packed background.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GridCell {
    /// Index into the glyph table; 0 is the blank cell.
    glyph: u16,
    /// Foreground as `0xRRGGBBAA`; alpha 0 inherits the element's color.
    fg: u32,
    /// Background as `0xRRGGBBAA`; alpha 0 keeps what is below the cell.
    bg: u32,
}

/// A width by height grid of glyphs with per-cell foreground colors and
/// optional per-cell backgrounds.
#[derive(Clone, Debug)]
pub struct CellGrid {
    width: u16,
    height: u16,
    glyphs: Vec<Arc<str>>,
    /// The cell width of each entry of `glyphs`, at least 1, measured once
    /// when the glyph is interned rather than for every cell painted.
    widths: Vec<usize>,
    intern: HashMap<Arc<str>, u16>,
    cells: Vec<GridCell>,
}

impl PartialEq for CellGrid {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width
            && self.height == other.height
            && self.cells == other.cells
            && self.glyphs == other.glyphs
    }
}

fn pack(color: Rgba) -> u32 {
    let channel = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u32;
    (channel(color.0) << 24) | (channel(color.1) << 16) | (channel(color.2) << 8) | channel(color.3)
}

fn unpack(packed: u32) -> Rgba {
    let channel = |shift: u32| ((packed >> shift) & 0xFF) as f32 / 255.0;
    (channel(24), channel(16), channel(8), channel(0))
}

impl CellGrid {
    /// An empty grid of `width` by `height` cells.
    pub fn new(width: u16, height: u16) -> Self {
        let blank: Arc<str> = Arc::from("");
        Self {
            width,
            height,
            glyphs: vec![blank.clone()],
            widths: vec![1],
            intern: HashMap::from([(blank, 0)]),
            cells: vec![GridCell::default(); usize::from(width) * usize::from(height)],
        }
    }

    /// Width in cells.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Height in cells.
    pub fn height(&self) -> u16 {
        self.height
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        (x < self.width && y < self.height)
            .then(|| usize::from(y) * usize::from(self.width) + usize::from(x))
    }

    fn intern(&mut self, glyph: &str) -> u16 {
        if let Some(&index) = self.intern.get(glyph) {
            return index;
        }
        let index = u16::try_from(self.glyphs.len()).unwrap_or(0);
        if index == 0 {
            return 0;
        }
        let glyph: Arc<str> = Arc::from(glyph);
        self.widths.push(UnicodeWidthStr::width(&*glyph).max(1));
        self.glyphs.push(glyph.clone());
        self.intern.insert(glyph, index);
        index
    }

    /// Put `glyph` at (`x`, `y`) with an optional foreground; an empty or
    /// whitespace-only glyph clears the cell, `None` inherits the element's
    /// color. A glyph wider than one cell covers the cells to its right.
    pub fn set(&mut self, x: u16, y: u16, glyph: &str, fg: Option<Rgba>) {
        let Some(index) = self.index(x, y) else {
            return;
        };
        let glyph = if glyph.trim().is_empty() || glyph.chars().any(char::is_control) {
            ""
        } else {
            glyph
        };
        let id = self.intern(glyph);
        self.cells[index] = GridCell {
            glyph: id,
            fg: fg.map_or(0, pack),
            bg: 0,
        };
    }

    /// Put `glyph` at (`x`, `y`) as [`CellGrid::set`] does, over `bg`: a
    /// background paints the cell even under a blank glyph, and `None`
    /// keeps what is below it.
    pub fn set_with_background(
        &mut self,
        x: u16,
        y: u16,
        glyph: &str,
        fg: Option<Rgba>,
        bg: Option<Rgba>,
    ) {
        self.set(x, y, glyph, fg);
        if let Some(index) = self.index(x, y) {
            self.cells[index].bg = bg.map_or(0, pack);
        }
    }

    /// The background at (`x`, `y`); `None` outside the grid and where the
    /// cell keeps what is below it.
    pub fn background(&self, x: u16, y: u16) -> Option<Rgba> {
        let cell = self.cells[self.index(x, y)?];
        (cell.bg & 0xFF != 0).then(|| unpack(cell.bg))
    }

    /// Clear the cell at (`x`, `y`).
    pub fn clear(&mut self, x: u16, y: u16) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = GridCell::default();
        }
    }

    /// The glyph and foreground at (`x`, `y`); `None` outside the grid, and
    /// an empty glyph for a blank cell.
    pub fn get(&self, x: u16, y: u16) -> Option<(&str, Option<Rgba>)> {
        let cell = self.cells[self.index(x, y)?];
        Some((
            &self.glyphs[usize::from(cell.glyph)],
            (cell.fg & 0xFF != 0).then(|| unpack(cell.fg)),
        ))
    }

    /// Whether the cell at (`x`, `y`) holds a glyph.
    pub fn is_set(&self, x: u16, y: u16) -> bool {
        self.index(x, y).is_some_and(|i| self.cells[i].glyph != 0)
    }

    /// Every set cell as (`x`, `y`, glyph, cell width, foreground), in row
    /// order.
    pub fn iter(&self) -> impl Iterator<Item = (u16, u16, &str, usize, Option<Rgba>)> + '_ {
        self.cells.iter().enumerate().filter_map(move |(i, cell)| {
            if cell.glyph == 0 {
                return None;
            }
            let x = (i % usize::from(self.width)) as u16;
            let y = (i / usize::from(self.width)) as u16;
            let glyph: &str = &self.glyphs[usize::from(cell.glyph)];
            Some((
                x,
                y,
                glyph,
                self.widths[usize::from(cell.glyph)],
                (cell.fg & 0xFF != 0).then(|| unpack(cell.fg)),
            ))
        })
    }

    /// Every cell that paints, with a glyph, a background or both, as
    /// (`x`, `y`, glyph, cell width, foreground, background) in row order;
    /// a blank glyph is empty. Colors stay packed as `0xRRGGBBAA`, the form
    /// the painter writes, so a frame converts no color through floats.
    #[allow(clippy::type_complexity)]
    pub(crate) fn painted(
        &self,
    ) -> impl Iterator<Item = (u16, u16, &str, usize, Option<u32>, Option<u32>)> + '_ {
        self.cells.iter().enumerate().filter_map(move |(i, cell)| {
            if cell.glyph == 0 && cell.bg & 0xFF == 0 {
                return None;
            }
            Some((
                (i % usize::from(self.width)) as u16,
                (i / usize::from(self.width)) as u16,
                &*self.glyphs[usize::from(cell.glyph)],
                self.widths[usize::from(cell.glyph)],
                (cell.fg & 0xFF != 0).then_some(cell.fg),
                (cell.bg & 0xFF != 0).then_some(cell.bg),
            ))
        })
    }

    /// The grid as text, one line per row, for tests and diagnostics.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let glyph = self.get(x, y).map_or("", |(g, _)| g);
                out.push_str(if glyph.is_empty() { " " } else { glyph });
            }
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_keep_glyphs_and_colors_and_clear_on_blank() {
        let mut grid = CellGrid::new(3, 2);
        grid.set(1, 0, "█", Some((1.0, 0.0, 0.0, 1.0)));
        grid.set(2, 1, "•", None);
        grid.set(9, 9, "x", None);
        assert_eq!(grid.get(1, 0), Some(("█", Some((1.0, 0.0, 0.0, 1.0)))));
        assert_eq!(grid.get(2, 1), Some(("•", None)));
        assert_eq!(grid.get(0, 0), Some(("", None)));
        assert_eq!(grid.get(3, 0), None);
        assert!(grid.is_set(1, 0) && !grid.is_set(0, 1));
        grid.set(1, 0, " ", None);
        assert!(!grid.is_set(1, 0));
        assert_eq!(grid.to_text(), "   \n  •\n");
        let set: Vec<_> = grid
            .iter()
            .map(|(x, y, g, w, _)| (x, y, g.to_string(), w))
            .collect();
        assert_eq!(set, vec![(2, 1, "•".to_string(), 1)]);
    }

    #[test]
    fn blt_001_backgrounds_paint_blank_cells_and_none_keeps_what_is_below() {
        let red = (1.0, 0.0, 0.0, 1.0);
        let blue = (0.0, 0.0, 1.0, 1.0);
        let mut grid = CellGrid::new(3, 1);
        grid.set_with_background(0, 0, " ", None, Some(red));
        grid.set_with_background(1, 0, "▐", Some(blue), Some(red));
        grid.set_with_background(2, 0, "▀", Some(blue), None);
        assert_eq!(grid.background(0, 0), Some(red));
        assert_eq!(grid.background(2, 0), None);
        assert_eq!(grid.background(3, 0), None);
        assert!(!grid.is_set(0, 0), "a blank glyph stays blank");
        let painted: Vec<_> = grid
            .painted()
            .map(|(x, _, glyph, _, fg, bg)| (x, glyph.to_string(), fg, bg))
            .collect();
        let (red, blue) = (pack(red), pack(blue));
        assert_eq!(
            painted,
            vec![
                (0, String::new(), None, Some(red)),
                (1, "▐".into(), Some(blue), Some(red)),
                (2, "▀".into(), Some(blue), None),
            ]
        );
        assert_eq!(grid.iter().count(), 2, "iter lists glyph cells only");
        grid.set(1, 0, "x", None);
        assert_eq!(grid.background(1, 0), None, "set replaces the background");
    }

    #[test]
    fn interning_shares_glyphs_and_equality_compares_contents() {
        let mut a = CellGrid::new(2, 1);
        a.set(0, 0, "█", Some((0.0, 0.5, 1.0, 1.0)));
        a.set(1, 0, "█", Some((0.0, 0.5, 1.0, 1.0)));
        assert_eq!(a.glyphs.len(), 2, "one interned glyph beside the blank");
        let mut b = CellGrid::new(2, 1);
        b.set(0, 0, "█", Some((0.0, 0.5, 1.0, 1.0)));
        b.set(1, 0, "█", Some((0.0, 0.5, 1.0, 1.0)));
        assert_eq!(a, b);
        b.clear(1, 0);
        assert_ne!(a, b);
        let packed = pack((0.2, 0.4, 0.6, 1.0));
        let back = unpack(packed);
        assert!((back.0 - 0.2).abs() < 0.01 && (back.2 - 0.6).abs() < 0.01);
    }
}
