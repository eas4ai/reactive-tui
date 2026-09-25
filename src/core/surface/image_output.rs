//! Surface image projection; protocol state stays in the shared output owner.
use super::{Attr, Cell, ImageCellPlacement, ImageData, Rgba, Surface};
use crate::{
    backend::{suprtui::graphics::RasterPlane, ImageOutputOptions},
    error::{ReactiveError, Result},
    widgets::{
        display::image::{decoded::blend_pixel, paint::ImageProtocol},
        ImageDisplayMode, ImageQuality,
    },
};

#[derive(Clone, PartialEq)]
pub(super) struct Raster {
    id: u32,
    protocol: ImageProtocol,
    pixels: image::RgbaImage,
    backgrounds: Vec<image::Rgba<u8>>,
    columns: u32,
    behind: bool,
}
impl RasterPlane for Raster {
    fn id(&self) -> u32 {
        self.id
    }
    fn protocol(&self) -> ImageProtocol {
        self.protocol
    }
    fn quality(&self) -> ImageQuality {
        ImageQuality::Balanced
    }
    fn z_index(&self, _: usize) -> i32 {
        if self.behind {
            -1
        } else {
            1
        }
    }
    fn position(&self) -> (u32, u32) {
        (0, 0)
    }
    fn raster(&self, _: (u16, u16)) -> Result<image::RgbaImage> {
        Ok(self.pixels.clone())
    }
    fn background(&self, x: u32, y: u32, cell: (u16, u16)) -> image::Rgba<u8> {
        self.backgrounds[((y / u32::from(cell.1)) * self.columns + x / u32::from(cell.0)) as usize]
    }
}

fn rgba(color: Rgba) -> image::Rgba<u8> {
    image::Rgba(
        [color.r, color.g, color.b, color.a].map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8),
    )
}
fn color(pixel: image::Rgba<u8>) -> Rgba {
    Rgba::new(
        f32::from(pixel[0]) / 255.0,
        f32::from(pixel[1]) / 255.0,
        f32::from(pixel[2]) / 255.0,
        f32::from(pixel[3]) / 255.0,
    )
}
fn placement(surface: &Surface, cell: Cell) -> Result<Option<(&ImageData, ImageCellPlacement)>> {
    let (Some(id), Some(placement)) = (cell.image_id, cell.image_placement) else {
        if cell.image_id.is_some() || cell.image_placement.is_some() {
            return Err(ReactiveError::invalid_parameter(
                "Incomplete Surface image placement",
            ));
        }
        return Ok(None);
    };
    let image = surface
        .get_image(id)
        .ok_or_else(|| ReactiveError::invalid_parameter("Unknown Surface image ID"))?;
    if !placement.opacity.is_finite()
        || !(0.0..=1.0).contains(&placement.opacity)
        || u32::from(placement.source_x) >= image.width
        || u32::from(placement.source_y) >= image.height
    {
        return Err(ReactiveError::invalid_parameter(
            "Invalid Surface image coordinates or opacity",
        ));
    }
    Ok(Some((image, placement)))
}

/// Zero source extent means the remaining image; oversized extents clip at its edge.
fn pixel(
    image: &ImageData,
    p: ImageCellPlacement,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> image::Rgba<u8> {
    let sx = u32::from(p.source_x);
    let sy = u32::from(p.source_y);
    let sw = if p.source_width == 0 {
        image.width - sx
    } else {
        u32::from(p.source_width).min(image.width - sx)
    };
    let sh = if p.source_height == 0 {
        image.height - sy
    } else {
        u32::from(p.source_height).min(image.height - sy)
    };
    let px = sx + ((u64::from(x) * u64::from(sw)) / u64::from(width)) as u32;
    let py = sy + ((u64::from(y) * u64::from(sh)) / u64::from(height)) as u32;
    let offset = (u64::from(py) * u64::from(image.width) + u64::from(px)) as usize * 4;
    let mut pixel = image::Rgba(
        image.pixels[offset..offset + 4]
            .try_into()
            .expect("validated RGBA"),
    );
    pixel[3] = (f32::from(pixel[3]) * p.opacity).round() as u8;
    pixel
}

pub(super) fn fallback(surface: &Surface, index: usize) -> Result<Cell> {
    let mut cell = surface.buf[index];
    let Some((image, p)) = placement(surface, cell)? else {
        if replaces_grapheme(surface, index) {
            cell.ch = ' ';
        }
        return Ok(cell);
    };
    if p.opacity == 0.0 {
        return Ok(cell);
    }
    let mut upper = rgba(cell.bg);
    let mut lower = upper;
    // Two vertical samples per cell preserve decoded color in a terminal glyph.
    let top = pixel(image, p, 1, 1, 2, 4);
    let bottom = pixel(image, p, 1, 3, 2, 4);
    if top[3] == 0 && bottom[3] == 0 {
        return Ok(cell);
    }
    blend_pixel(&mut upper, &top);
    blend_pixel(&mut lower, &bottom);
    if p.z_index < 0 && !matches!(cell.ch, ' ' | '\0') {
        cell.bg = color(upper);
    } else {
        cell.ch = '▀';
        cell.fg = color(upper);
        cell.bg = color(lower);
        cell.attr = Attr::empty();
    }
    Ok(cell)
}

/// Sixel and inline protocols have no text z-index. Keep the glyph and use a
/// decoded background sample in covered text cells; other cells retain pixels.
pub(super) fn legacy_text(surface: &Surface, index: usize) -> Result<Cell> {
    let mut cell = surface.buf[index];
    if let Some((image, p)) = placement(surface, cell)? {
        if p.z_index < 0 {
            let mut background = rgba(cell.bg);
            blend_pixel(&mut background, &pixel(image, p, 1, 1, 2, 2));
            cell.bg = color(background);
        }
    }
    Ok(cell)
}

/// A terminal cannot display half of a wide glyph beside a cell image. Clear
/// its complete footprint before emitting foreground fallback cells.
pub(super) fn replaces_grapheme(surface: &Surface, index: usize) -> bool {
    let Some((&start, (_, width))) = surface.graphemes.range(..=index).next_back() else {
        return false;
    };
    if index >= start + width {
        return false;
    }
    surface.buf[start..start + width].iter().any(|cell| {
        let Ok(Some((image, p))) = placement(surface, *cell) else {
            return false;
        };
        p.z_index >= 0 && (pixel(image, p, 1, 1, 2, 4)[3] > 0 || pixel(image, p, 1, 3, 2, 4)[3] > 0)
    })
}

pub(super) fn project(
    surface: &Surface,
    options: ImageOutputOptions,
    ids: [u32; 2],
) -> Result<Vec<Raster>> {
    if !(1..=256).contains(&options.cell_pixels.0) || !(1..=256).contains(&options.cell_pixels.1) {
        return Err(ReactiveError::invalid_parameter(
            "Image cell dimensions must be in 1..=256",
        ));
    }
    let protocol = options.protocol(ImageDisplayMode::Auto);
    let mut used = [false; 2];
    for cell in &surface.buf {
        if let Some((_, p)) = placement(surface, *cell)? {
            used[usize::from(p.z_index >= 0)] |= p.opacity > 0.0;
        }
    }
    let Some(protocol) = protocol else {
        return Ok(Vec::new());
    };
    if !used.iter().any(|used| *used) {
        return Ok(Vec::new());
    }
    let width = surface.w.checked_mul(usize::from(options.cell_pixels.0));
    let height = surface.h.checked_mul(usize::from(options.cell_pixels.1));
    let (Some(width), Some(height)) = (width, height) else {
        return Err(ReactiveError::resource("Surface image dimensions overflow"));
    };
    if width
        .checked_mul(height)
        .and_then(|n| n.checked_mul(used.iter().filter(|used| **used).count()))
        .is_none_or(|n| n > 8 * 1024 * 1024)
    {
        return Err(ReactiveError::resource(
            "Surface image projection exceeds 32 MiB limit",
        ));
    }
    let cw = u32::from(options.cell_pixels.0);
    let ch = u32::from(options.cell_pixels.1);
    let mut result = Vec::new();
    for layer in 0..2 {
        if !used[layer] {
            continue;
        }
        let mut pixels = image::RgbaImage::new(width as u32, height as u32);
        for (index, cell) in surface.buf.iter().enumerate() {
            let Some((image, p)) = placement(surface, *cell)? else {
                continue;
            };
            if usize::from(p.z_index >= 0) != layer {
                continue;
            }
            if layer == 0
                && protocol != ImageProtocol::Kitty
                && (!matches!(cell.ch, ' ' | '\0') || surface.grapheme_continuation(index))
            {
                continue;
            }
            let left = (index % surface.w) as u32 * cw;
            let top = (index / surface.w) as u32 * ch;
            for y in 0..ch {
                for x in 0..cw {
                    pixels.put_pixel(left + x, top + y, pixel(image, p, x, y, cw, ch));
                }
            }
        }
        result.push(Raster {
            id: ids[layer],
            protocol,
            pixels,
            columns: surface.w as u32,
            behind: layer == 0,
            backgrounds: surface
                .buf
                .iter()
                .map(|c| {
                    let mut bg = rgba(c.bg);
                    bg[3] = 255;
                    bg
                })
                .collect(),
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::surface::DiffWriter;

    fn source() -> Surface {
        let mut surface = Surface::new(2, 1);
        surface.clear(Rgba::black());
        let id = surface.create_image_from_rgba(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 0, 255, 255, 0, 255, 0, 255, 255, 255, 0, 255,
            ],
        );
        surface.place_image_region(0, 0, 2, 1, id, 2, 2, 1, 1.0);
        surface
    }

    #[test]
    fn api_surface_image_fallback_uses_source_regions_and_removes_pixels() {
        let mut next = source();
        let left = fallback(&next, 0).unwrap();
        let right = fallback(&next, 1).unwrap();
        assert_eq!(left.ch, '▀');
        assert_eq!(rgba(left.fg).0, [255, 0, 0, 255]);
        assert_eq!(rgba(left.bg).0, [0, 255, 0, 255]);
        assert_eq!(rgba(right.fg).0, [0, 0, 255, 255]);
        assert_eq!(rgba(right.bg).0, [255, 255, 0, 255]);
        let mut writer = DiffWriter::new();
        writer.try_diff(&Surface::new(2, 1), &next, false).unwrap();
        let output = String::from_utf8_lossy(writer.output());
        assert!(output.contains("38;2;255;0;0m"));
        assert!(output.contains("48;2;0;255;0m"));
        assert_eq!(output.matches('▀').count(), 2);
        let old = next.clone_into_new();
        next.clear_images();
        writer.try_diff(&old, &next, false).unwrap();
        assert!(!String::from_utf8_lossy(writer.output()).contains('▀'));
        assert!(writer.output().contains(&b' '));
    }

    #[test]
    fn api_surface_image_projection_layer_alpha_and_checked_errors() {
        let mut next = source();
        let mut cell = next.get(0, 0);
        cell.ch = 'A';
        cell.image_placement.as_mut().unwrap().z_index = -1;
        next.set(0, 0, cell);
        let options = ImageOutputOptions {
            kitty_graphics: true,
            cell_pixels: (1, 2),
            ..Default::default()
        };
        let planes = project(&next, options, [19, 20]).unwrap();
        assert_eq!(planes.len(), 2);
        assert_eq!(planes[0].pixels.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(planes[0].z_index(0), -1);
        let legacy = project(
            &next,
            ImageOutputOptions {
                sixel: true,
                kitty_graphics: false,
                ..options
            },
            [19, 20],
        )
        .unwrap();
        assert!(legacy[0].pixels.pixels().all(|p| p[3] == 0));
        assert_eq!(legacy_text(&next, 0).unwrap().ch, 'A');
        assert_eq!(rgba(legacy_text(&next, 0).unwrap().bg).0, [0, 255, 0, 255]);
        assert_eq!(planes[1].pixels.get_pixel(1, 0).0, [0, 0, 255, 255]);
        assert_eq!(planes[1].pixels.get_pixel(1, 1).0, [255, 255, 0, 255]);
        assert_eq!(fallback(&next, 0).unwrap().ch, 'A');
        cell.image_placement.as_mut().unwrap().opacity = 0.5;
        next.set(0, 0, cell);
        assert_eq!(rgba(fallback(&next, 0).unwrap().bg).0, [128, 0, 0, 255]);
        cell.image_placement.as_mut().unwrap().source_x = 99;
        next.set(0, 0, cell);
        let mut writer = DiffWriter::new();
        assert!(writer.try_diff(&Surface::new(2, 1), &next, false).is_err());
        assert!(writer.output().is_empty());
        writer.diff(&Surface::new(2, 1), &next, false);
        assert!(writer.image_error().is_some());
        assert!(writer.output().is_empty());
    }

    #[test]
    fn api_surface_image_foreground_fallback_clears_a_whole_wide_grapheme() {
        let mut surface = Surface::new(2, 1);
        surface.set_grapheme(
            0,
            0,
            "界",
            Cell {
                fg: Rgba::white(),
                bg: Rgba::black(),
                ..Default::default()
            },
        );
        let id = surface.create_test_image(1, 1, 255, 0, 0);
        surface.place_image_foreground(1, 0, id);
        let mut writer = DiffWriter::new();
        writer
            .try_diff(&Surface::new(2, 1), &surface, true)
            .unwrap();
        let bytes = String::from_utf8_lossy(writer.output());
        assert!(!bytes.contains('界'));
        assert!(bytes.contains('▀'));
        assert_eq!(fallback(&surface, 0).unwrap().ch, ' ');
        assert_eq!(fallback(&surface, 1).unwrap().ch, '▀');
    }

    #[test]
    fn api_surface_image_protocol_retry_and_removal_are_owned() {
        for protocol in 0..3 {
            let mut next = source();
            let empty = Surface::new(2, 1);
            let options = ImageOutputOptions {
                kitty_graphics: protocol == 0,
                sixel: protocol == 1,
                iterm2_inline: protocol == 2,
                cell_pixels: (1, 2),
            };
            let mut writer = DiffWriter::new();
            writer.set_image_options(options);
            writer.try_diff(&empty, &next, false).unwrap();
            let marker = ["\x1b_G", "\x1bP", "\x1b]1337;File="][protocol];
            assert!(String::from_utf8_lossy(writer.output()).contains(marker));
            assert!(!writer.image_cleanup().is_empty());
            // A partial write was not acknowledged: cleanup and payload recur.
            writer.try_diff(&empty, &next, false).unwrap();
            assert!(String::from_utf8_lossy(writer.output()).contains(marker));
            writer.acknowledge_output();
            writer.try_diff(&next, &next, false).unwrap();
            assert!(!String::from_utf8_lossy(writer.output()).contains(marker));
            let old = next.clone_into_new();
            next.clear_images();
            writer.try_diff(&old, &next, false).unwrap();
            assert!(!writer.image_cleanup().is_empty());
            let bytes = String::from_utf8_lossy(writer.output());
            assert!(if protocol == 0 {
                bytes.contains("a=d,d=I")
            } else {
                bytes.contains("\x1b[2J")
            });
            writer.acknowledge_output();
            assert!(writer.image_cleanup().is_empty());
        }
    }
}
