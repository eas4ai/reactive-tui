//! The software renderer (GFX-002): every draw of the canvas on the CPU,
//! for a host with no usable hardware adapter and after a fault (GFX-007).
//! It blends as the hardware adapter's target does: colors premultiplied
//! in sRGB, one byte per channel, rounded after each draw.

use super::compile::Draw;
use super::glyphs::{Bitmap, Glyphs};
use super::paint::{premultiply, Premul, Shader};
use super::raster::{rasterize, Window};
use crate::layout::CellGrid;

/// A clip in force: how much of each pixel of its window shows, 0 to 255.
struct Mask {
    window: Window,
    alpha: Vec<u8>,
}

impl Mask {
    fn at(&self, x: u32, y: u32) -> f32 {
        let window = &self.window;
        if x < window.left || x >= window.right || y < window.top || y >= window.bottom {
            return 0.0;
        }
        let index = (y - window.top) * window.width() + (x - window.left);
        f32::from(self.alpha[index as usize]) / 255.0
    }
}

/// A color packed as `0xRRGGBBAA`, premultiplied.
pub(crate) fn unpack_premultiplied(packed: u32) -> Premul {
    premultiply([
        ((packed >> 24) & 0xFF) as f32 / 255.0,
        ((packed >> 16) & 0xFF) as f32 / 255.0,
        ((packed >> 8) & 0xFF) as f32 / 255.0,
        (packed & 0xFF) as f32 / 255.0,
    ])
}

/// A picture being drawn: premultiplied RGBA bytes, row by row.
struct Picture {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
    clips: Vec<Mask>,
}

impl Picture {
    fn pixel(&mut self, x: u32, y: u32) -> &mut [u8] {
        let index = (y as usize * self.width as usize + x as usize) * 4;
        &mut self.bytes[index..index + 4]
    }

    /// How much of pixel (`x`, `y`) the clip in force shows.
    fn shows(&self, x: u32, y: u32) -> f32 {
        self.clips.last().map_or(1.0, |mask| mask.at(x, y))
    }

    /// Put `source`, premultiplied, over pixel (`x`, `y`).
    fn blend(&mut self, x: u32, y: u32, source: Premul) {
        let keep = 1.0 - source[3];
        let pixel = self.pixel(x, y);
        for (channel, value) in pixel.iter_mut().zip(source) {
            let blended = value + f32::from(*channel) / 255.0 * keep;
            *channel = (blended.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    }

    /// Paint pixel (`x`, `y`) with `shader` at `coverage`.
    fn paint(&mut self, x: u32, y: u32, coverage: f32, shader: &Shader, solid: Option<Premul>) {
        let coverage = coverage * self.shows(x, y);
        if coverage <= 0.0 {
            return;
        }
        let color = solid.unwrap_or_else(|| shader.at(x, y));
        self.blend(x, y, color.map(|c| c * coverage));
    }

    fn fill(&mut self, polygons: &[super::geometry::Polygon], shader: &Shader) {
        let Some(coverage) = rasterize(polygons, (self.width, self.height)) else {
            return;
        };
        let solid = solid(shader);
        for row in 0..coverage.height {
            let y = coverage.top + row as u32;
            for column in 0..coverage.width {
                let alpha = coverage.alpha[row * coverage.width + column];
                if alpha > 0.0 {
                    self.paint(coverage.left + column as u32, y, alpha, shader, solid);
                }
            }
        }
    }

    fn rect(&mut self, [left, top, right, bottom]: [f32; 4], shader: &Shader) {
        let (width, height) = (self.width as f32, self.height as f32);
        let from = (
            left.floor().clamp(0.0, width),
            top.floor().clamp(0.0, height),
        );
        let to = (
            right.ceil().clamp(0.0, width),
            bottom.ceil().clamp(0.0, height),
        );
        let solid = solid(shader);
        let opaque = shader.opaque().filter(|_| self.clips.is_empty()).map(bytes);
        for y in from.1 as u32..to.1 as u32 {
            let down = ((y + 1) as f32).min(bottom) - (y as f32).max(top);
            if down <= 0.0 {
                continue;
            }
            for x in from.0 as u32..to.0 as u32 {
                let across = ((x + 1) as f32).min(right) - (x as f32).max(left);
                let coverage = (across * down).clamp(0.0, 1.0);
                match opaque {
                    Some(color) if coverage >= 1.0 => self.pixel(x, y).copy_from_slice(&color),
                    _ => self.paint(x, y, coverage, shader, solid),
                }
            }
        }
    }

    /// The part of an area of `size` pixels placed at (`x`, `y`) that lies
    /// in the picture, as (area column, area row, picture x, picture y).
    fn visible(
        &self,
        size: (u32, u32),
        x: i32,
        y: i32,
    ) -> impl Iterator<Item = (u32, u32, u32, u32)> {
        let (width, height) = (self.width as i32, self.height as i32);
        let columns = (-x).max(0)..(size.0 as i32).min(width - x);
        let rows = (-y).max(0)..(size.1 as i32).min(height - y);
        rows.flat_map(move |row| {
            columns.clone().map(move |column| {
                (
                    column as u32,
                    row as u32,
                    (x + column) as u32,
                    (y + row) as u32,
                )
            })
        })
    }

    fn glyph(&mut self, bitmap: &Bitmap, x: i32, y: i32, shader: &Shader) {
        let solid = solid(shader);
        for (column, row, px, py) in self.visible((bitmap.width, bitmap.height), x, y) {
            let coverage = bitmap.coverage[(row * bitmap.width + column) as usize];
            if coverage != 0 {
                self.paint(px, py, f32::from(coverage) / 255.0, shader, solid);
            }
        }
    }

    fn cells(&mut self, x: i32, y: i32, grid: &CellGrid, cell: (u16, u16), glyphs: &mut Glyphs) {
        let (cell_width, cell_height) = (i32::from(cell.0), i32::from(cell.1));
        for (column, row, glyph, wide, fg, bg) in grid.painted() {
            let wide = wide.clamp(1, 2);
            let left = x + i32::from(column) * cell_width;
            let top = y + i32::from(row) * cell_height;
            if left >= self.width as i32
                || top >= self.height as i32
                || left + cell_width * wide as i32 <= 0
                || top + cell_height <= 0
            {
                continue;
            }
            let fg = unpack_premultiplied(fg.unwrap_or(0xFFFF_FFFF));
            let bg = bg.map_or([0.0; 4], unpack_premultiplied);
            let tile = if glyph.is_empty() {
                None
            } else {
                glyphs.tile(glyph, cell, wide)
            };
            let area = (u32::from(cell.0) * wide as u32, u32::from(cell.1));
            for (tx, ty, px, py) in self.visible(area, left, top) {
                let coverage = tile.as_ref().map_or(0.0, |tile| {
                    f32::from(tile.coverage[(ty * tile.width + tx) as usize]) / 255.0
                });
                // The glyph over the cell's background, as one color.
                let keep = 1.0 - fg[3] * coverage;
                let color = [0, 1, 2, 3].map(|i| fg[i] * coverage + bg[i] * keep);
                let shows = self.shows(px, py);
                if color[3] > 0.0 && shows > 0.0 {
                    self.blend(px, py, color.map(|c| c * shows));
                }
            }
        }
    }

    fn push_clip(&mut self, polygons: &[super::geometry::Polygon], window: Window) {
        let mut alpha = vec![0u8; (window.width() * window.height()) as usize];
        if let Some(coverage) = rasterize(polygons, (self.width, self.height)) {
            for row in 0..coverage.height {
                let y = coverage.top + row as u32;
                for column in 0..coverage.width {
                    let x = coverage.left + column as u32;
                    let shows = coverage.alpha[row * coverage.width + column] * self.shows(x, y);
                    let index = (y - window.top) * window.width() + (x - window.left);
                    alpha[index as usize] = (shows.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        }
        self.clips.push(Mask { window, alpha });
    }
}

/// The color of a paint that is the same everywhere.
fn solid(shader: &Shader) -> Option<Premul> {
    matches!(shader.kind, super::paint::ShaderKind::Solid(_)).then(|| shader.at(0, 0))
}

fn bytes(color: Premul) -> [u8; 4] {
    color.map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// `bytes`, premultiplied RGBA, to straight alpha in place: what a
/// [`GraphicsFrame`](super::GraphicsFrame) holds, from either renderer.
pub(crate) fn unpremultiply(bytes: &mut [u8]) {
    for pixel in bytes.chunks_exact_mut(4) {
        let alpha = u32::from(pixel[3]);
        if alpha == 255 || alpha == 0 {
            continue;
        }
        for channel in &mut pixel[..3] {
            *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
        }
    }
}

/// The picture of `draws` at `size`, as straight RGBA bytes row by row.
pub(crate) fn render(draws: &[Draw], size: (u32, u32), glyphs: &mut Glyphs) -> Vec<u8> {
    let mut picture = Picture {
        width: size.0,
        height: size.1,
        bytes: vec![0u8; size.0 as usize * size.1 as usize * 4],
        clips: Vec::new(),
    };
    for draw in draws {
        match draw {
            Draw::Fill {
                polygons, shader, ..
            } => picture.fill(polygons, shader),
            Draw::Rect { rect, shader } => picture.rect(*rect, shader),
            Draw::Glyphs { glyphs, shader } => {
                for glyph in glyphs {
                    picture.glyph(&glyph.bitmap, glyph.x, glyph.y, shader);
                }
            }
            Draw::Cells { x, y, grid, cell } => picture.cells(*x, *y, grid, *cell, glyphs),
            Draw::PushClip { polygons, window } => picture.push_clip(polygons, *window),
            Draw::PopClip => {
                picture.clips.pop();
            }
        }
    }
    unpremultiply(&mut picture.bytes);
    picture.bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::compile::compile;
    use crate::graphics::fonts::{Font, FontSource};
    use crate::graphics::scene::{Color, Paint, Path, Scene, Stroke, Transform};
    use std::sync::Arc;

    fn draw(scene: &Scene, size: (u32, u32)) -> Vec<[u8; 4]> {
        let mut glyphs = Glyphs::new(Font::load(&FontSource::Bundled));
        let draws = compile(scene, &Transform::identity(), size, &mut glyphs);
        render(&draws, size, &mut glyphs)
            .chunks_exact(4)
            .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
            .collect()
    }

    fn solid(r: u8, g: u8, b: u8, a: u8) -> Paint {
        Paint::solid(Color::rgba(r, g, b, a))
    }

    #[test]
    fn an_empty_scene_is_transparent_and_a_fill_covers_its_pixels() {
        assert!(draw(&Scene::new(), (4, 4)).iter().all(|p| *p == [0; 4]));
        let mut scene = Scene::new();
        scene.fill(&Path::rect(1.0, 1.0, 2.0, 2.0), &solid(10, 20, 30, 255));
        let picture = draw(&scene, (4, 4));
        assert_eq!(picture[4 + 1], [10, 20, 30, 255]);
        assert_eq!(picture[0], [0; 4]);
    }

    #[test]
    fn a_half_covered_pixel_over_a_background_is_half_each() {
        let mut scene = Scene::new();
        scene.fill(&Path::rect(0.0, 0.0, 4.0, 4.0), &solid(0, 0, 0, 255));
        scene.fill(&Path::rect(0.5, 0.0, 2.0, 4.0), &solid(255, 255, 255, 255));
        let picture = draw(&scene, (4, 4));
        assert_eq!(picture[0], [128, 128, 128, 255]);
        assert_eq!(picture[1], [255, 255, 255, 255]);
        assert_eq!(picture[2], [128, 128, 128, 255]);
        assert_eq!(picture[3], [0, 0, 0, 255]);
    }

    #[test]
    fn a_translucent_color_over_nothing_keeps_its_color_and_alpha() {
        let mut scene = Scene::new();
        scene.fill(&Path::rect(0.0, 0.0, 2.0, 2.0), &solid(200, 100, 50, 128));
        let picture = draw(&scene, (2, 2));
        let [r, g, b, a] = picture[0];
        assert_eq!(a, 128);
        assert!(r.abs_diff(200) <= 1 && g.abs_diff(100) <= 1 && b.abs_diff(50) <= 1);
    }

    #[test]
    fn a_clip_shows_only_what_is_inside_it_and_a_nested_one_less() {
        let mut scene = Scene::new();
        scene.push_clip(&Path::rect(0.0, 0.0, 4.0, 8.0));
        scene.push_clip(&Path::rect(2.0, 0.0, 6.0, 8.0));
        scene.fill(&Path::ellipse(4.0, 4.0, 20.0, 20.0), &solid(255, 0, 0, 255));
        scene.pop_clip();
        scene.fill(&Path::ellipse(4.0, 4.0, 20.0, 20.0), &solid(0, 0, 255, 128));
        scene.pop_clip();
        let picture = draw(&scene, (8, 8));
        assert_eq!(picture[8 * 4 + 1][3], 128, "the outer clip only");
        assert_eq!(
            picture[8 * 4 + 3][0],
            127,
            "inside both clips: blue over red"
        );
        assert_eq!(picture[8 * 4 + 5], [0; 4], "outside the outer clip");
    }

    #[test]
    fn a_stroke_is_as_wide_as_it_says() {
        let mut scene = Scene::new();
        let line = crate::graphics::scene::PathBuilder::new()
            .move_to(0.0, 4.0)
            .line_to(8.0, 4.0)
            .build();
        scene.stroke(&line, &Stroke::new(2.0), &solid(255, 255, 255, 255));
        let picture = draw(&scene, (8, 8));
        let column: Vec<u8> = (0..8).map(|y| picture[y * 8 + 4][3]).collect();
        assert_eq!(column, vec![0, 0, 0, 255, 255, 0, 0, 0]);
    }

    #[test]
    fn a_cell_grid_paints_backgrounds_and_glyphs() {
        let mut grid = CellGrid::new(2, 1);
        grid.set_with_background(
            0,
            0,
            "▀",
            Some((1.0, 0.0, 0.0, 1.0)),
            Some((0.0, 0.0, 1.0, 1.0)),
        );
        grid.set(1, 0, "█", None);
        let mut scene = Scene::new();
        scene.cells((0.0, 0.0), Arc::new(grid), (2, 4));
        let picture = draw(&scene, (4, 4));
        assert_eq!(picture[0], [255, 0, 0, 255], "the glyph's upper half");
        assert_eq!(picture[3 * 4], [0, 0, 255, 255], "the background below it");
        assert_eq!(picture[2], [255, 255, 255, 255], "no color given is white");
    }

    #[test]
    fn text_puts_ink_on_its_line() {
        let mut scene = Scene::new();
        scene.text((2.0, 20.0), 20.0, "Hi", &solid(255, 255, 255, 255));
        let picture = draw(&scene, (40, 24));
        let ink = |rows: std::ops::Range<usize>| {
            rows.flat_map(|y| (0..40).map(move |x| (x, y)))
                .filter(|&(x, y)| picture[y * 40 + x][3] > 128)
                .count()
        };
        assert!(ink(4..21) > 30, "ink above the baseline: {}", ink(4..21));
        assert_eq!(
            ink(22..24),
            0,
            "no ink below the baseline for these letters"
        );
    }
}
