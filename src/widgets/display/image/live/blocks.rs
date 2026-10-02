//! Image cell fallback through the renderer's blitters: the image scaled to
//! the content box at the blitter's pixels per cell, each block drawn as one
//! glyph over one background (docs/spec/blitters.md).
use super::super::{Image, ImageQuality};
use crate::layout::paint_tree::cells::CellGrid;
use ::suprtui::blit::{self, Blitter};

/// The pixel size that fits an image of `image` pixels into `columns` by
/// `rows` cells at `blitter`'s pixels per cell. A cell is twice as tall as
/// it is wide, so keeping the aspect ratio scales both sides by one length.
fn fitted(
    image: (u32, u32),
    (columns, rows): (u32, u32),
    blitter: Blitter,
    preserve_aspect: bool,
) -> (u32, u32) {
    let (per_column, per_row) = blitter.cell_pixels();
    let (widest, tallest) = (columns * per_column, rows * per_row);
    if !preserve_aspect || image.0 == 0 || image.1 == 0 {
        return (widest, tallest);
    }
    // Lengths in cell widths: the box is `columns` wide and `2 * rows` tall.
    let scale =
        (f64::from(columns) / f64::from(image.0)).min(2.0 * f64::from(rows) / f64::from(image.1));
    let width = (f64::from(image.0) * scale * f64::from(per_column)).round() as u32;
    let height = (f64::from(image.1) * scale / 2.0 * f64::from(per_row)).round() as u32;
    (width.clamp(1, widest), height.clamp(1, tallest))
}

/// The image drawn with `blitter` into at most `columns` by `rows` cells
/// from the top left. A background color is composited under the pixels;
/// without one, a transparent pixel leaves its cell's background
/// transparent.
pub(super) fn grid(
    pixels: &image::RgbaImage,
    image: &Image,
    (columns, rows): (u32, u32),
    blitter: Blitter,
) -> Result<CellGrid, String> {
    if columns == 0
        || rows == 0
        || columns > u32::from(u16::MAX)
        || rows > u32::from(u16::MAX)
        || u64::from(columns) * u64::from(rows) > crate::backend::CellFrame::MAX_CELLS as u64
    {
        return Err("Image layout exceeds the bounded cell grid".into());
    }
    let (width, height) = fitted(
        pixels.dimensions(),
        (columns, rows),
        blitter,
        image.preserve_aspect,
    );
    let filter = match image.quality {
        ImageQuality::Fast => image::imageops::FilterType::Nearest,
        ImageQuality::Balanced => image::imageops::FilterType::Triangle,
        ImageQuality::High => image::imageops::FilterType::Lanczos3,
    };
    let mut scaled = image::imageops::resize(pixels, width, height, filter);
    if image.background_color.is_some() {
        let opaque = super::super::decoded::rgb_pixels(&scaled, image.background_color);
        scaled = image::DynamicImage::ImageRgb8(opaque).into_rgba8();
    }
    let blitted = blit::blit_image(blitter, &scaled);
    let mut grid = CellGrid::new(blitted.columns as u16, blitted.rows as u16);
    let color = |[r, g, b]: [u8; 3]| {
        (
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            1.0,
        )
    };
    let mut glyph = [0u8; 4];
    let stride = blitted.columns.max(1) as usize;
    for (index, cell) in blitted.cells.iter().enumerate() {
        grid.set_with_background(
            (index % stride) as u16,
            (index / stride) as u16,
            cell.glyph.encode_utf8(&mut glyph),
            cell.fg.map(color),
            cell.bg.map(color),
        );
    }
    Ok(grid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Image {
        Image {
            background_color: None,
            ..Image::default()
        }
    }

    #[test]
    fn blt_001_a_square_image_keeps_its_shape_in_cells_twice_as_tall_as_wide() {
        // 100 by 100 pixels into 40 by 10 cells: the box is 40 wide and 20
        // tall in cell widths, so the image is 20 by 20, 20 columns by 10 rows.
        for blitter in Blitter::TIERS {
            let (per_column, per_row) = blitter.cell_pixels();
            let (width, height) = fitted((100, 100), (40, 10), blitter, true);
            assert_eq!(width.div_ceil(per_column), 20, "{blitter:?} columns");
            assert_eq!(height.div_ceil(per_row), 10, "{blitter:?} rows");
        }
        assert_eq!(
            fitted((100, 100), (40, 10), Blitter::Sextant, false),
            (80, 30)
        );
        assert_eq!(
            fitted((1000, 1), (4, 4), Blitter::Octant, true).1,
            1,
            "never zero"
        );
    }

    #[test]
    fn blt_001_blocks_carry_the_image_colors_and_transparency() {
        // Left half red, right half blue: every quadrant cell splits there.
        let pixels = image::RgbaImage::from_fn(8, 8, |x, _| {
            image::Rgba(if x < 4 {
                [255, 0, 0, 255]
            } else {
                [0, 0, 255, 255]
            })
        });
        // Nearest scaling keeps the edge sharp; this test is about colors.
        let sharp = Image {
            quality: ImageQuality::Fast,
            ..config()
        };
        let halves = grid(&pixels, &sharp, (8, 4), Blitter::Quadrant).unwrap();
        assert_eq!((halves.width(), halves.height()), (8, 4));
        let red = Some((1.0, 0.0, 0.0, 1.0));
        let blue = Some((0.0, 0.0, 1.0, 1.0));
        assert_eq!(halves.background(0, 0), red);
        assert_eq!(halves.background(7, 3), blue);
        assert!((0..4).all(|y| halves.background(3, y) == red && halves.background(4, y) == blue));

        let clear = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 255, 0, 0]));
        let transparent = grid(&clear, &config(), (4, 2), Blitter::HalfBlock).unwrap();
        assert!(
            transparent.painted().next().is_none(),
            "nothing paints over what is below"
        );
        let mut backed = config();
        backed.background_color = Some(crate::core::surface::Rgba::new(1.0, 1.0, 1.0, 1.0));
        let white = grid(&clear, &backed, (4, 2), Blitter::HalfBlock).unwrap();
        assert_eq!(white.background(0, 0), Some((1.0, 1.0, 1.0, 1.0)));
    }

    #[test]
    fn blt_001_oversized_layouts_are_refused() {
        let pixels = image::RgbaImage::new(1, 1);
        assert!(grid(&pixels, &config(), (0, 1), Blitter::Sextant).is_err());
        assert!(grid(&pixels, &config(), (70_000, 1), Blitter::Sextant).is_err());
    }

    #[test]
    fn blt_001_grid_backgrounds_reach_the_terminal_through_the_painter() {
        use crate::backend::{Backend, SuprTuiBackend};
        use crate::component::{Element, LayoutType};
        #[derive(Clone, Default)]
        struct Sink(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Sink {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        // Red on the left, a transparent pixel on the right: the right cell
        // keeps the parent's blue background under the red half block.
        let pixels = image::RgbaImage::from_fn(2, 2, |x, y| {
            image::Rgba(if x == 1 && y == 1 {
                [0, 0, 0, 0]
            } else {
                [255, 0, 0, 255]
            })
        });
        let grid = grid(&pixels, &config(), (2, 1), Blitter::HalfBlock).unwrap();
        let element = Element::layout(LayoutType::Flex)
            .with_class("w-full h-full bg-blue-500")
            .with_children(vec![Element::layout(LayoutType::Flex)
                .with_class("w-2 h-1")
                .with_cells(std::sync::Arc::new(grid))]);
        let sink = Sink::default();
        let mut backend = SuprTuiBackend::with_writer(4, 2, sink.clone()).unwrap();
        assert!(backend.render_frame(&element).unwrap());
        backend.present().unwrap();
        backend.sync().unwrap();
        let mut parser = vt100::Parser::new(2, 4, 0);
        parser.process(&sink.0.lock().unwrap());
        let screen = parser.screen();
        assert_eq!(
            screen.cell(0, 0).unwrap().bgcolor(),
            vt100::Color::Rgb(255, 0, 0)
        );
        let right = screen.cell(0, 1).unwrap();
        assert_eq!(
            right.contents(),
            "\u{2580}",
            "the opaque upper pixel is the glyph"
        );
        assert_eq!(right.fgcolor(), vt100::Color::Rgb(255, 0, 0));
        assert_eq!(
            right.bgcolor(),
            screen.cell(0, 3).unwrap().bgcolor(),
            "the transparent pixel keeps the parent's background"
        );
    }
}
