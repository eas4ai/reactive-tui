//! Glyph pixels for text and cell grids, computed once by the canvas's own
//! rasterizer and used by both renderers: the software renderer blends them
//! and the GPU keeps them in its glyph atlas, so text is the same picture
//! on both (GFX-002).
//!
//! Text under a transform that only moves it is drawn from bitmaps placed
//! on whole pixels, one bitmap per quarter-pixel step of the pen; text
//! under any other transform, or taller than [`MAX_BITMAP_SIZE`], is drawn
//! as filled outlines. A cell of a grid is a tile exactly the cell's size:
//! block, quadrant, sextant, octant and braille glyphs are drawn as the
//! shapes they name, so neighbouring cells meet without a seam, box drawing
//! is stretched to the cell, and any other glyph is fitted inside it.

use super::fonts::Font;
use super::geometry::{fill_polygons, Polygon};
use super::raster::rasterize;
use super::scene::{Path, Transform};
use std::collections::HashMap;
use std::sync::Arc;
use suprtui::blit::Blitter;

/// Text taller than this many pixels is drawn as outlines, which keeps
/// large bitmaps out of the glyph atlas.
pub(crate) const MAX_BITMAP_SIZE: f32 = 96.0;

/// Pen positions per pixel that have their own bitmap.
const PHASES: f32 = 4.0;

/// The most text bitmaps and the most tiles kept; past either the cache
/// starts again, and so does the GPU's atlas.
const MAX_ENTRIES: usize = 8192;

/// Coverage of one glyph, row by row, 0 to 255.
#[derive(Debug)]
pub(crate) struct Bitmap {
    /// Names the bitmap in the GPU's atlas.
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub coverage: Vec<u8>,
}

/// A text bitmap and where its top left corner goes, relative to the whole
/// pixel of the pen and the baseline's row.
#[derive(Clone, Debug)]
struct TextGlyph {
    left: i32,
    top: i32,
    bitmap: Arc<Bitmap>,
}

/// A bitmap placed in the picture.
#[derive(Clone, Debug)]
pub(crate) struct Placed {
    pub x: i32,
    pub y: i32,
    pub bitmap: Arc<Bitmap>,
}

/// One line of text, ready for either renderer.
#[derive(Debug)]
pub(crate) enum Text {
    Bitmaps(Vec<Placed>),
    Outlines(Vec<Polygon>),
}

/// A rectangle in fractions of a cell.
#[derive(Clone, Copy)]
struct Part {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

fn part(left: f32, top: f32, right: f32, bottom: f32) -> Part {
    Part {
        left,
        top,
        right,
        bottom,
    }
}

/// How a glyph that names a shape fills its cell.
enum Shape {
    Rects(Vec<Part>),
    /// The whole cell at one coverage.
    Shade(f32),
    /// Braille dots: the set bits of the pattern, dot `n` being bit `n - 1`.
    Dots(u8),
}

/// The tiles of one cell size, by glyph; `None` for a glyph that draws
/// nothing.
type Tiles = HashMap<Box<str>, Option<Arc<Bitmap>>>;

/// The font's glyphs as pixels.
pub(crate) struct Glyphs {
    font: Font,
    /// Glyph and advance in font units of each character seen.
    characters: HashMap<char, (u16, f32)>,
    text: HashMap<(u16, u32, u8), Option<TextGlyph>>,
    /// Tiles by cell width, cell height and width in cells.
    tiles: HashMap<(u16, u16, usize), Tiles>,
    tile_count: usize,
    next_id: u32,
    /// Counts the times the cache started again; the GPU's atlas follows.
    generation: u64,
}

impl Glyphs {
    pub fn new(font: Font) -> Self {
        Self {
            font,
            characters: HashMap::new(),
            text: HashMap::new(),
            tiles: HashMap::new(),
            tile_count: 0,
            next_id: 0,
            generation: 0,
        }
    }

    pub fn font(&self) -> &Font {
        &self.font
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Forget every bitmap when the cache is full. Called between frames, so
    /// a frame's bitmaps stay valid while it is drawn.
    pub fn trim(&mut self) {
        if self.text.len() > MAX_ENTRIES || self.tile_count > MAX_ENTRIES {
            self.text.clear();
            self.tiles.clear();
            self.tile_count = 0;
            self.generation += 1;
        }
    }

    fn character(&mut self, character: char) -> (u16, f32) {
        if let Some(&known) = self.characters.get(&character) {
            return known;
        }
        let glyph = self.font.glyph(character);
        let known = (glyph, self.font.glyph_advance(glyph));
        self.characters.insert(character, known);
        known
    }

    fn bitmap(&mut self, width: u32, height: u32, coverage: Vec<u8>) -> Arc<Bitmap> {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        Arc::new(Bitmap {
            id,
            width,
            height,
            coverage,
        })
    }

    /// `text` on one line from `origin` on its baseline, glyphs `size`
    /// pixels tall, under `transform`.
    pub fn text(
        &mut self,
        origin: (f32, f32),
        size: f32,
        text: &str,
        transform: &Transform,
    ) -> Text {
        if !(size.is_finite() && size > 0.0) {
            return Text::Bitmaps(Vec::new());
        }
        let scale = size / self.font.units_per_em;
        let mut pen = 0.0f32;
        if transform.is_translation() && size <= MAX_BITMAP_SIZE {
            let (x, y) = transform.apply(origin);
            let baseline = y.round() as i32;
            let mut placed = Vec::new();
            for character in text.chars().filter(|c| !c.is_control()) {
                let (glyph, advance) = self.character(character);
                let at = x + pen;
                let column = at.floor();
                let phase = (((at - column) * PHASES) as u8).min(PHASES as u8 - 1);
                if let Some(drawn) = self.text_glyph(glyph, size, phase) {
                    placed.push(Placed {
                        x: column as i32 + drawn.left,
                        y: baseline + drawn.top,
                        bitmap: drawn.bitmap,
                    });
                }
                pen += advance * scale;
            }
            return Text::Bitmaps(placed);
        }
        let mut polygons = Vec::new();
        for character in text.chars().filter(|c| !c.is_control()) {
            let (glyph, advance) = self.character(character);
            if let Some(outline) = self.font.outline(glyph) {
                let place = Transform::scale(scale, scale)
                    .then(Transform::translate(origin.0 + pen, origin.1))
                    .then(*transform);
                polygons.extend(fill_polygons(&outline, &place));
            }
            pen += advance * scale;
        }
        Text::Outlines(polygons)
    }

    fn text_glyph(&mut self, glyph: u16, size: f32, phase: u8) -> Option<TextGlyph> {
        let key = (glyph, size.to_bits(), phase);
        if let Some(known) = self.text.get(&key) {
            return known.clone();
        }
        let drawn = self.draw_text_glyph(glyph, size, phase);
        self.text.insert(key, drawn.clone());
        drawn
    }

    fn draw_text_glyph(&mut self, glyph: u16, size: f32, phase: u8) -> Option<TextGlyph> {
        let outline = self.font.outline(glyph)?;
        let scale = size / self.font.units_per_em;
        let shift = f32::from(phase) / PHASES;
        // The outline with the pen at (shift, 0); its pixels are found by
        // drawing it moved into a window that holds it.
        let place = Transform::scale(scale, scale).then(Transform::translate(shift, 0.0));
        let polygons = fill_polygons(&outline, &place);
        let (mut left, mut top) = (f32::INFINITY, f32::INFINITY);
        let (mut right, mut bottom) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &(x, y) in polygons.iter().flatten() {
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        }
        if !(right > left && bottom > top) {
            return None;
        }
        let (left, top) = (left.floor(), top.floor());
        let width = (right.ceil() - left) as u32;
        let height = (bottom.ceil() - top) as u32;
        if width == 0 || height == 0 || width > 4096 || height > 4096 {
            return None;
        }
        let moved: Vec<Polygon> = polygons
            .into_iter()
            .map(|polygon| {
                polygon
                    .into_iter()
                    .map(|(x, y)| (x - left, y - top))
                    .collect()
            })
            .collect();
        let coverage = window(&moved, (width, height))?;
        Some(TextGlyph {
            left: left as i32,
            top: top as i32,
            bitmap: self.bitmap(width, height, coverage),
        })
    }

    /// The tile of `glyph` for a cell of `cell` pixels, `wide` cells wide;
    /// `None` for a glyph that draws nothing.
    pub fn tile(&mut self, glyph: &str, cell: (u16, u16), wide: usize) -> Option<Arc<Bitmap>> {
        let wide = wide.clamp(1, 2);
        if let Some(known) = self
            .tiles
            .get(&(cell.0, cell.1, wide))
            .and_then(|tiles| tiles.get(glyph))
        {
            return known.clone();
        }
        let width = u32::from(cell.0) * wide as u32;
        let height = u32::from(cell.1);
        let drawn = self
            .draw_tile(glyph, (width, height))
            .map(|coverage| self.bitmap(width, height, coverage));
        self.tiles
            .entry((cell.0, cell.1, wide))
            .or_default()
            .insert(glyph.into(), drawn.clone());
        self.tile_count += 1;
        drawn
    }

    fn draw_tile(&mut self, glyph: &str, (width, height): (u32, u32)) -> Option<Vec<u8>> {
        if width == 0 || height == 0 {
            return None;
        }
        let (w, h) = (width as f32, height as f32);
        let mut characters = glyph.chars();
        let first = characters.next()?;
        if characters.next().is_none() {
            match shape(first) {
                Some(Shape::Shade(coverage)) => {
                    let value = (coverage * 255.0).round() as u8;
                    return Some(vec![value; (width * height) as usize]);
                }
                Some(Shape::Rects(rects)) => {
                    let polygons: Vec<Polygon> = rects
                        .iter()
                        .map(|part| {
                            vec![
                                (part.left * w, part.top * h),
                                (part.right * w, part.top * h),
                                (part.right * w, part.bottom * h),
                                (part.left * w, part.bottom * h),
                            ]
                        })
                        .collect();
                    return window(&polygons, (width, height));
                }
                Some(Shape::Dots(pattern)) => {
                    let radius = (w / 2.0).min(h / 4.0) * 0.3;
                    let mut polygons = Vec::new();
                    for dot in (0..8u8).filter(|dot| pattern & (1 << dot) != 0) {
                        // Dots 1, 2, 3 and 7 go down the left column, and
                        // 4, 5, 6 and 8 down the right.
                        let (column, row): (u8, u8) = match dot {
                            0..=2 => (0, dot),
                            3..=5 => (1, dot - 3),
                            6 => (0, 3),
                            _ => (1, 3),
                        };
                        let centre = (
                            (f32::from(column) + 0.5) * w / 2.0,
                            (f32::from(row) + 0.5) * h / 4.0,
                        );
                        polygons.extend(fill_polygons(
                            &Path::ellipse(centre.0, centre.1, radius, radius),
                            &Transform::identity(),
                        ));
                    }
                    return window(&polygons, (width, height));
                }
                None => {}
            }
        }
        let line = self.font.ascent + self.font.descent;
        let advance = self.font.advance.max(1.0);
        let mut polygons = Vec::new();
        for character in glyph.chars().filter(|c| !c.is_control()) {
            let (id, _) = self.character(character);
            let Some(outline) = self.font.outline(id) else {
                continue;
            };
            let place = if ('\u{2500}'..='\u{257F}').contains(&character) {
                // Box drawing fills the cell, so lines join across cells.
                Transform::scale(w / advance, h / line)
                    .then(Transform::translate(0.0, self.font.ascent * h / line))
            } else {
                let scale = (h / line).min(w / advance);
                let baseline = ((h - line * scale) / 2.0 + self.font.ascent * scale).round();
                Transform::scale(scale, scale)
                    .then(Transform::translate((w - advance * scale) / 2.0, baseline))
            };
            polygons.extend(fill_polygons(&outline, &place));
        }
        window(&polygons, (width, height))
    }
}

/// The coverage of `polygons` over a whole window of `size` pixels as bytes;
/// `None` when they cover none of it.
fn window(polygons: &[Polygon], size: (u32, u32)) -> Option<Vec<u8>> {
    let coverage = rasterize(polygons, size)?;
    let mut bytes = vec![0u8; (size.0 * size.1) as usize];
    let mut any = false;
    for row in 0..coverage.height {
        let from = row * coverage.width;
        let to = (coverage.top as usize + row) * size.0 as usize + coverage.left as usize;
        for (byte, alpha) in bytes[to..to + coverage.width]
            .iter_mut()
            .zip(&coverage.alpha[from..from + coverage.width])
        {
            *byte = (alpha * 255.0).round() as u8;
            any |= *byte != 0;
        }
    }
    any.then_some(bytes)
}

/// The shape `character` names, for the glyphs drawn as shapes.
fn shape(character: char) -> Option<Shape> {
    let code = u32::from(character);
    let eighth = |n: u32| n as f32 / 8.0;
    Some(match code {
        0x2580 => Shape::Rects(vec![part(0.0, 0.0, 1.0, 0.5)]),
        0x2581..=0x2588 => Shape::Rects(vec![part(0.0, 1.0 - eighth(code - 0x2580), 1.0, 1.0)]),
        0x2589..=0x258F => Shape::Rects(vec![part(0.0, 0.0, eighth(0x2590 - code), 1.0)]),
        0x2590 => Shape::Rects(vec![part(0.5, 0.0, 1.0, 1.0)]),
        0x2591 => Shape::Shade(0.25),
        0x2592 => Shape::Shade(0.5),
        0x2593 => Shape::Shade(0.75),
        0x2594 => Shape::Rects(vec![part(0.0, 0.0, 1.0, eighth(1))]),
        0x2595 => Shape::Rects(vec![part(eighth(7), 0.0, 1.0, 1.0)]),
        0x2800..=0x28FF => Shape::Dots((code - 0x2800) as u8),
        _ => {
            let &(columns, rows, pattern) = blocks().get(&character)?;
            let mut rects = Vec::new();
            for bit in (0..columns * rows).filter(|bit| pattern & (1 << bit) != 0) {
                let (column, row) = (bit % columns, bit / columns);
                rects.push(part(
                    f32::from(column) / f32::from(columns),
                    f32::from(row) / f32::from(rows),
                    f32::from(column + 1) / f32::from(columns),
                    f32::from(row + 1) / f32::from(rows),
                ));
            }
            Shape::Rects(rects)
        }
    })
}

/// Every glyph the octant, sextant and quadrant blitters draw, with its
/// grid and the pattern of its set blocks in reading order.
fn blocks() -> &'static HashMap<char, (u8, u8, u8)> {
    static BLOCKS: std::sync::OnceLock<HashMap<char, (u8, u8, u8)>> = std::sync::OnceLock::new();
    BLOCKS.get_or_init(|| {
        let mut blocks = HashMap::new();
        for blitter in [Blitter::Octant, Blitter::Sextant, Blitter::Quadrant] {
            let (columns, rows) = blitter.cell_pixels();
            let patterns = 1u32 << (columns * rows);
            for pattern in 1..patterns {
                blocks.entry(blitter.glyph(pattern as u8)).or_insert((
                    columns as u8,
                    rows as u8,
                    pattern as u8,
                ));
            }
        }
        blocks
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::fonts::FontSource;

    fn glyphs() -> Glyphs {
        Glyphs::new(Font::load(&FontSource::Bundled))
    }

    #[test]
    fn text_that_only_moves_is_bitmaps_on_whole_pixels() {
        let mut glyphs = glyphs();
        let Text::Bitmaps(placed) =
            glyphs.text((10.25, 40.0), 24.0, "ab c", &Transform::translate(3.0, 2.0))
        else {
            panic!("text under a translation is drawn from bitmaps");
        };
        // The space draws nothing.
        assert_eq!(placed.len(), 3);
        assert!(placed.windows(2).all(|pair| pair[0].x < pair[1].x));
        // "a" sits on the baseline at row 42 and is about half the size tall.
        let a = &placed[0];
        assert!(a.y > 42 - 24 && a.y < 42, "top row {}", a.y);
        assert!((a.y + a.bitmap.height as i32 - 42).abs() <= 1);
        assert!(a.bitmap.coverage.contains(&255));
    }

    #[test]
    fn the_same_glyph_at_the_same_phase_is_one_bitmap() {
        let mut glyphs = glyphs();
        let place = |glyphs: &mut Glyphs, x: f32| match glyphs.text(
            (x, 20.0),
            14.0,
            "m",
            &Transform::identity(),
        ) {
            Text::Bitmaps(placed) => placed[0].bitmap.id,
            Text::Outlines(_) => panic!("bitmaps"),
        };
        assert_eq!(place(&mut glyphs, 4.1), place(&mut glyphs, 9.2));
        assert_ne!(place(&mut glyphs, 4.1), place(&mut glyphs, 4.6));
    }

    #[test]
    fn rotated_or_large_text_is_outlines() {
        let mut glyphs = glyphs();
        for (size, transform) in [
            (24.0, Transform::rotate(30.0)),
            (200.0, Transform::identity()),
        ] {
            match glyphs.text((0.0, 0.0), size, "a", &transform) {
                Text::Outlines(polygons) => assert!(!polygons.is_empty()),
                Text::Bitmaps(_) => panic!("outlines at size {size}"),
            }
        }
    }

    #[test]
    fn block_glyphs_fill_exactly_their_part_of_the_cell() {
        let mut glyphs = glyphs();
        let upper = glyphs.tile("▀", (8, 16), 1).expect("the upper half block");
        assert_eq!((upper.width, upper.height), (8, 16));
        for (index, &coverage) in upper.coverage.iter().enumerate() {
            assert_eq!(
                coverage,
                if index < 8 * 8 { 255 } else { 0 },
                "pixel {index}"
            );
        }
        let full = glyphs.tile("█", (8, 16), 1).expect("the full block");
        assert!(full.coverage.iter().all(|&c| c == 255));
        let shade = glyphs.tile("▒", (8, 16), 1).expect("the medium shade");
        assert!(shade.coverage.iter().all(|&c| c == 128));
        // The lower right quadrant, a glyph of the quadrant blitter.
        let quadrant = glyphs.tile("▗", (8, 16), 1).expect("a quadrant");
        assert_eq!(quadrant.coverage[0], 0);
        assert_eq!(quadrant.coverage[15 * 8 + 7], 255);
        assert!(glyphs.tile(" ", (8, 16), 1).is_none());
    }

    #[test]
    fn a_horizontal_box_line_reaches_both_sides_of_its_cell() {
        let mut glyphs = glyphs();
        let line = glyphs.tile("─", (8, 16), 1).expect("a box drawing line");
        let row = (0..16)
            .find(|row| line.coverage[row * 8] > 128)
            .expect("a row the line covers at the left side");
        assert!(line.coverage[row * 8 + 7] > 128);
    }

    #[test]
    fn a_letter_fits_inside_its_cell() {
        let mut glyphs = glyphs();
        let letter = glyphs.tile("a", (8, 16), 1).expect("a letter");
        assert_eq!(letter.coverage.len(), 8 * 16);
        assert!(letter.coverage.iter().any(|&c| c > 200));
        // Nothing in the top row: the letter sits below the ascent.
        assert!(letter.coverage[..8].iter().all(|&c| c == 0));
    }
}
