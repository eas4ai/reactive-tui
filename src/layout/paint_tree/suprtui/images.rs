//! Image placement follows the painter's cell transforms and coverage.
use super::{inside_masks, Affine, NodePaint, PaintNode, Rect};
use crate::{
    error::{ReactiveError, Result},
    widgets::display::image::paint::{ImagePaint, ImageProtocol},
};
use ::suprtui::ansi;
use std::sync::Arc;

/// Index only the image planes beneath each cell. Coverage work scales with
/// actual overlap, rather than the total image count for every painted glyph.
pub(super) struct Layers {
    planes: Vec<Plane>,
    cells: Option<Vec<Vec<usize>>>,
    width: usize,
    height: usize,
    entries: usize,
}
impl Layers {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            planes: Vec::new(),
            cells: None,
            width,
            height,
            entries: 0,
        }
    }
    pub fn push(&mut self, plane: Plane) -> Result<()> {
        self.entries += plane.cover.len();
        if self.entries > 4_194_304 {
            return Err(ReactiveError::resource(
                "image coverage exceeds frame limit",
            ));
        }
        let cells = self
            .cells
            .get_or_insert_with(|| vec![Vec::new(); self.width * self.height]);
        let index = self.planes.len();
        for y in plane.bounds.top..plane.bounds.bottom {
            for x in plane.bounds.left..plane.bounds.right {
                cells[y as usize * self.width + x as usize].push(index);
            }
        }
        self.planes.push(plane);
        Ok(())
    }
    /// Whether the frame has an image plane for painted cells to cover.
    pub fn has_planes(&self) -> bool {
        self.cells.is_some()
    }
    pub fn cover(&mut self, x: i32, y: i32, source: ansi::Rgba) {
        let Some(cells) = &self.cells else {
            return;
        };
        for &index in &cells[y as usize * self.width + x as usize] {
            self.planes[index].cover(x, y, source);
        }
    }
    /// `cover` for the cells `left..right` of row `y`; returns at once when
    /// the frame has no image plane.
    pub fn cover_span(&mut self, y: i32, left: i32, right: i32, source: ansi::Rgba) {
        if self.cells.is_none() {
            return;
        }
        for x in left..right {
            self.cover(x, y, source);
        }
    }
    pub fn into_planes(self) -> Vec<Plane> {
        self.planes
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Cover {
    visible: bool,
    tint: [u8; 4],
    background: [u8; 4],
}

#[derive(Clone, PartialEq)]
pub(crate) struct Plane {
    image: Arc<ImagePaint>,
    bounds: Rect,
    content: Rect,
    transform: Affine,
    opacity: f32,
    cover: Vec<Cover>,
    protocol: ImageProtocol,
}
impl Plane {
    pub(super) fn new(
        image: Arc<ImagePaint>,
        paint: &NodePaint,
        node: &PaintNode,
        target: &::suprtui::buffer::OptimizedBuffer<'_>,
        protocol: ImageProtocol,
    ) -> Result<Option<Self>> {
        let bounds = node.bounds.intersect(node.clip);
        let content = Rect {
            left: paint.pad.left as i32,
            top: paint.pad.top as i32,
            right: node.local.right - paint.pad.right as i32,
            bottom: node.local.bottom - paint.pad._bottom as i32,
        };
        let opacity = paint.opacity * node.parent_opacity;
        if bounds.right <= bounds.left
            || bounds.bottom <= bounds.top
            || content.right <= content.left
            || content.bottom <= content.top
            || opacity <= 0.0
            || node.transform.inverse(0.0, 0.0).is_none()
        {
            return Ok(None);
        }
        let count = (bounds.right - bounds.left) as usize * (bounds.bottom - bounds.top) as usize;
        if count > 262_144 {
            return Err(ReactiveError::resource(
                "image placement exceeds frame cell limit",
            ));
        }
        let mut cover = Vec::with_capacity(count);
        for y in bounds.top..bounds.bottom {
            for x in bounds.left..bounds.right {
                let bg = target
                    .get(x as u32, y as u32)
                    .expect("clipped image cell")
                    .bg;
                cover.push(Cover {
                    visible: inside_masks(&node.mask, x, y),
                    tint: [0; 4],
                    background: [ansi::red(bg), ansi::green(bg), ansi::blue(bg), 255],
                });
            }
        }
        Ok(Some(Self {
            image,
            bounds,
            content,
            transform: node.transform,
            opacity,
            cover,
            protocol,
        }))
    }
    pub fn id(&self) -> u32 {
        self.image.id
    }
    pub fn protocol(&self) -> ImageProtocol {
        self.protocol
    }
    pub fn quality(&self) -> crate::widgets::ImageQuality {
        self.image.quality
    }
    pub fn background(&self, x: u32, y: u32, cell: (u16, u16)) -> image::Rgba<u8> {
        let width = (self.bounds.right - self.bounds.left) as usize;
        image::Rgba(
            self.cover[(y / u32::from(cell.1)) as usize * width + (x / u32::from(cell.0)) as usize]
                .background,
        )
    }
    pub fn position(&self) -> (u32, u32) {
        (self.bounds.left as u32, self.bounds.top as u32)
    }
    pub(super) fn cover(&mut self, x: i32, y: i32, source: ansi::Rgba) {
        if x < self.bounds.left
            || x >= self.bounds.right
            || y < self.bounds.top
            || y >= self.bounds.bottom
        {
            return;
        }
        let index = (y - self.bounds.top) as usize
            * (self.bounds.right - self.bounds.left) as usize
            + (x - self.bounds.left) as usize;
        let cover = &mut self.cover[index];
        if ansi::alpha(source) == 255 {
            cover.visible = false;
            return;
        }
        let old = cover.tint;
        let bg = cover.background;
        let bg = ::suprtui::buffer::draw::blend_colors(
            source,
            ansi::rgb_color(bg[0], bg[1], bg[2], bg[3]),
            None,
        );
        cover.background = [ansi::red(bg), ansi::green(bg), ansi::blue(bg), 255];
        let blended = ::suprtui::buffer::draw::blend_colors(
            source,
            ansi::rgb_color(old[0], old[1], old[2], old[3]),
            None,
        );
        cover.tint = [
            ansi::red(blended),
            ansi::green(blended),
            ansi::blue(blended),
            ansi::alpha(blended),
        ];
    }
    pub fn raster(&self, cell: (u16, u16)) -> Result<image::RgbaImage> {
        let cw = u32::from(cell.0);
        let ch = u32::from(cell.1);
        let width = (self.bounds.right - self.bounds.left) as u32 * cw;
        let height = (self.bounds.bottom - self.bounds.top) as u32 * ch;
        check_pixels(width, height)?;
        let available = (
            (self.content.right - self.content.left) as u32 * cw,
            (self.content.bottom - self.content.top) as u32 * ch,
        );
        let max = self.image.max_size.unwrap_or(available);
        let max = (max.0.min(available.0), max.1.min(available.1));
        let mut output = image::RgbaImage::new(width, height);
        if max.0 == 0 || max.1 == 0 {
            return Ok(output);
        }
        let size = if self.image.preserve_aspect {
            let ratio = (max.0 as f64 / self.image.pixels.width() as f64)
                .min(max.1 as f64 / self.image.pixels.height() as f64)
                .min(1.0);
            (
                (self.image.pixels.width() as f64 * ratio).max(1.0) as u32,
                (self.image.pixels.height() as f64 * ratio).max(1.0) as u32,
            )
        } else {
            max
        };
        check_pixels(size.0, size.1)?;
        let filter = match self.image.quality {
            crate::widgets::ImageQuality::Fast => image::imageops::FilterType::Nearest,
            crate::widgets::ImageQuality::Balanced => image::imageops::FilterType::Triangle,
            crate::widgets::ImageQuality::High => image::imageops::FilterType::Lanczos3,
        };
        let source = image::imageops::resize(self.image.pixels.as_ref(), size.0, size.1, filter);
        for (x, y, pixel) in output.enumerate_pixels_mut() {
            let index = (y / ch) as usize * (width / cw) as usize + (x / cw) as usize;
            let cover = self.cover[index];
            if !cover.visible {
                continue;
            }
            let world_x = self.bounds.left as f32 + (x as f32 + 0.5) / cw as f32 - 0.5;
            let world_y = self.bounds.top as f32 + (y as f32 + 0.5) / ch as f32 - 0.5;
            let Some((lx, ly)) = self.transform.inverse(world_x, world_y) else {
                continue;
            };
            let sx = (lx - self.content.left as f32 + 0.5) * cw as f32;
            let sy = (ly - self.content.top as f32 + 0.5) * ch as f32;
            if sx < 0.0 || sy < 0.0 || sx >= size.0 as f32 || sy >= size.1 as f32 {
                continue;
            }
            *pixel = *source.get_pixel(sx as u32, sy as u32);
            if let Some(bg) = self.image.background {
                let mut below = image::Rgba([
                    (bg.r.clamp(0.0, 1.0) * 255.0).round() as u8,
                    (bg.g.clamp(0.0, 1.0) * 255.0).round() as u8,
                    (bg.b.clamp(0.0, 1.0) * 255.0).round() as u8,
                    (bg.a.clamp(0.0, 1.0) * 255.0).round() as u8,
                ]);
                crate::widgets::display::image::decoded::blend_pixel(&mut below, pixel);
                *pixel = below;
            }
            for (channel, tint) in pixel.0[..3].iter_mut().zip(&cover.tint[..3]) {
                *channel = ((u32::from(*channel) * (255 - u32::from(cover.tint[3]))
                    + u32::from(*tint) * u32::from(cover.tint[3])
                    + 127)
                    / 255) as u8;
            }
            pixel.0[3] = (f32::from(pixel.0[3]) * self.opacity.clamp(0.0, 1.0)).round() as u8;
        }
        Ok(output)
    }
}
fn check_pixels(width: u32, height: u32) -> Result<()> {
    if width.checked_mul(height).is_none_or(|n| n > 16_777_216) {
        return Err(ReactiveError::resource(
            "projected image exceeds 64 MiB RGBA limit",
        ));
    }
    Ok(())
}
