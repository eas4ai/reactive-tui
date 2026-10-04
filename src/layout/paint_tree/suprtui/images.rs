//! Image placement follows the painter's cell transforms and coverage.
use super::{inside_masks, Affine, NodePaint, PaintNode, Rect};
use crate::error::{ReactiveError, Result};
#[cfg(feature = "wgpu-graphics")]
pub(super) use crate::widgets::display::image::paint::Refusal;
pub(super) use crate::widgets::display::image::paint::{ImagePaint, ImageProtocol};
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
    /// The cells a pixel look's picture lies over, with the look's fill
    /// there: text painted later in one of them takes that fill as its
    /// background, so the text cell and the picture beside it are one color
    /// (PIX-001, PIX-003). A later element's own background takes the cell
    /// back.
    look_fills: std::collections::HashMap<(i32, i32), ansi::Rgba>,
    /// Whether the node being painted has its look's picture in the frame:
    /// its background fill then covers no plane, neither its own picture
    /// nor the pictures under it, while its text covers them all (PIX-001).
    look_painting: bool,
}
impl Layers {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            planes: Vec::new(),
            cells: None,
            width,
            height,
            entries: 0,
            look_fills: std::collections::HashMap::new(),
            look_painting: false,
        }
    }
    /// Remember that text in cell `(x, y)` takes `fill` as its background.
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub fn set_look_fill(&mut self, x: i32, y: i32, fill: ansi::Rgba) {
        self.look_fills.insert((x, y), fill);
    }
    /// The background text in cell `(x, y)` takes, when a look's picture
    /// lies over the cell.
    pub fn look_fill(&self, x: i32, y: i32) -> Option<ansi::Rgba> {
        self.look_fills.get(&(x, y)).copied()
    }
    /// Whether the node being painted has its look's picture in the frame.
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub fn set_look_painting(&mut self, painting: bool) {
        self.look_painting = painting;
    }
    /// Whether plane `index` still shows cell `(x, y)`: nothing painted
    /// over it covered the cell.
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub fn plane_shows(&self, index: usize, x: i32, y: i32) -> bool {
        self.planes[index].shows_cell(x, y)
    }
    /// Cover, at cell `(x, y)`, the pictures of the looks under plane
    /// `plane`: a look inside another look's box cuts the outer picture out
    /// under its whole box, once, so text typed into it changes its own
    /// picture alone (PIX-002, PIX-006).
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub fn cover_under(&mut self, plane: usize, x: i32, y: i32) {
        let Some(cells) = &self.cells else {
            return;
        };
        for &index in &cells[y as usize * self.width + x as usize] {
            if index < plane && self.planes[index].look {
                self.planes[index].cover(x, y, ansi::rgb_color(0, 0, 0, 255));
            }
        }
    }
    /// Place `plane` over the planes before it; its place in the frame.
    pub fn push(&mut self, plane: Plane) -> Result<usize> {
        // A plane that is refused leaves the count as it was, so the
        // planes after it are judged without it.
        let entries = self.entries + plane.cover.len();
        if entries > 4_194_304 {
            return Err(ReactiveError::resource(
                "image coverage exceeds frame limit",
            ));
        }
        self.entries = entries;
        let cells = self
            .cells
            .get_or_insert_with(|| vec![Vec::new(); self.width * self.height]);
        let index = self.planes.len();
        for y in plane.bounds.top..plane.bounds.bottom {
            for x in plane.bounds.left..plane.bounds.right {
                let under = &mut cells[y as usize * self.width + x as usize];
                // A canvas drawn over cells (Sixel, iTerm2) leaves the cells
                // a plane above it covers to that plane: its picture is made
                // ready apart and written when it is ready, which may be
                // after the plane above it is (GFX-009).
                for &lower in under.iter() {
                    let lower = &mut self.planes[lower];
                    if lower.image.canvas.is_some() && lower.protocol != ImageProtocol::Kitty {
                        lower.hide(x, y);
                    }
                }
                under.push(index);
            }
        }
        self.planes.push(plane);
        Ok(index)
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
    /// `cover` for a node's background fill of cell `(x, y)`: it takes the
    /// cell back from a look whose picture lies over it, and covers nothing
    /// while the node painted has its own look's picture in the frame
    /// (PIX-001).
    pub fn cover_background(&mut self, x: i32, y: i32, source: ansi::Rgba) {
        if self.look_painting {
            return;
        }
        if !self.look_fills.is_empty() {
            self.look_fills.remove(&(x, y));
        }
        self.cover(x, y, source);
    }
    /// `cover_background` for the cells `left..right` of row `y`.
    pub fn cover_span_background(&mut self, y: i32, left: i32, right: i32, source: ansi::Rgba) {
        if self.look_painting || (self.cells.is_none() && self.look_fills.is_empty()) {
            return;
        }
        for x in left..right {
            self.cover_background(x, y, source);
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
    /// Whether the plane is a pixel look's picture over its element's
    /// whole box (PIX-001).
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    look: bool,
}
impl Plane {
    pub(super) fn new(
        image: Arc<ImagePaint>,
        paint: &NodePaint,
        node: &PaintNode,
        target: &::suprtui::buffer::OptimizedBuffer<'_>,
        protocol: ImageProtocol,
    ) -> Result<Option<Self>> {
        Self::build(image, paint, node, target, protocol, false)
    }
    /// A plane over the node's whole box, padding included: a pixel look's
    /// picture (PIX-001).
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub(super) fn covering(
        image: Arc<ImagePaint>,
        paint: &NodePaint,
        node: &PaintNode,
        target: &::suprtui::buffer::OptimizedBuffer<'_>,
        protocol: ImageProtocol,
    ) -> Result<Option<Self>> {
        Self::build(image, paint, node, target, protocol, true)
    }
    fn build(
        image: Arc<ImagePaint>,
        paint: &NodePaint,
        node: &PaintNode,
        target: &::suprtui::buffer::OptimizedBuffer<'_>,
        protocol: ImageProtocol,
        whole_box: bool,
    ) -> Result<Option<Self>> {
        let bounds = node.bounds.intersect(node.clip);
        let content = if whole_box {
            node.local
        } else {
            Rect {
                left: paint.pad.left as i32,
                top: paint.pad.top as i32,
                right: node.local.right - paint.pad.right as i32,
                bottom: node.local.bottom - paint.pad._bottom as i32,
            }
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
            look: whole_box,
        }))
    }
    pub fn id(&self) -> u32 {
        self.image.id
    }
    /// How the picture is sent, when it is a canvas's.
    pub fn canvas(&self) -> Option<crate::widgets::display::image::paint::CanvasPicture> {
        self.image.canvas
    }
    /// Whether this is the next picture of the canvas `previous` shows, in
    /// the same place with the same cells showing, so it can replace that
    /// picture where it is (GFX-005).
    pub fn replaces(&self, previous: &Self) -> bool {
        self.image.canvas.is_some()
            && self.image.canvas == previous.image.canvas
            && self.image.id == previous.image.id
            && self.image.pixels.dimensions() == previous.image.pixels.dimensions()
            && self.bounds == previous.bounds
            && self.content == previous.content
            && self.transform == previous.transform
            && self.opacity == previous.opacity
            && self.protocol == previous.protocol
            && self.cover == previous.cover
    }
    /// Whether this plane shows the picture `previous` showed, cell for
    /// cell, at other cells: the same canvas picture over a box of the same
    /// size, placed the same way inside it, with the same cells cut out,
    /// only moved. A Kitty host then places the image it holds again
    /// instead of receiving its pixels again (PIX-002).
    pub fn moved_from(&self, previous: &Self) -> bool {
        let size = |rect: &Rect| (rect.right - rect.left, rect.bottom - rect.top);
        // Where the picture's top left pixel lies, in cells from the plane's.
        let inset = |plane: &Self| {
            let (x, y) = plane.transform.offset();
            (
                x.round() as i32 + plane.content.left - plane.bounds.left,
                y.round() as i32 + plane.content.top - plane.bounds.top,
            )
        };
        self.image.canvas.is_some()
            && self.protocol == ImageProtocol::Kitty
            && previous.protocol == ImageProtocol::Kitty
            && self.image == previous.image
            && self.bounds != previous.bounds
            && size(&self.bounds) == size(&previous.bounds)
            && size(&self.content) == size(&previous.content)
            && self.transform.is_translation()
            && previous.transform.is_translation()
            && inset(self) == inset(previous)
            && self.opacity == previous.opacity
            && self.cover.len() == previous.cover.len()
            && self
                .cover
                .iter()
                .zip(&previous.cover)
                .all(|(now, then)| now.visible == then.visible && now.tint == then.tint)
    }
    /// The pixels one cell of the plane's raster holds: a canvas picture's
    /// own pixels per cell, which may be fewer than the terminal's
    /// (GFX-010); the terminal's `cell` for any other plane.
    pub fn raster_cell(&self, cell: (u16, u16)) -> (u16, u16) {
        self.image.canvas.map_or(cell, |picture| picture.cell)
    }

    /// A canvas's picture cut to the plane's cells, at the picture's own
    /// pixels per cell, when the plane shows it as it is: only moved,
    /// opaque, and its picture the size of its content. Cells another
    /// element covers are cleared or tinted as `raster` does, whole cells
    /// of the picture, so the holes stay on cell edges (GFX-010).
    fn canvas_raster(&self, whole: bool) -> Option<image::RgbaImage> {
        let picture = self.image.canvas?;
        let (cw, ch) = (u32::from(picture.cell.0), u32::from(picture.cell.1));
        let content = (
            (self.content.right - self.content.left) as u32 * cw,
            (self.content.bottom - self.content.top) as u32 * ch,
        );
        if !self.transform.is_translation()
            || self.opacity < 1.0
            || self.image.pixels.dimensions() != content
        {
            return None;
        }
        let (offset_x, offset_y) = self.transform.offset();
        // The picture's top left pixel, in pixels from the plane's.
        let left = (offset_x.round() as i32 + self.content.left - self.bounds.left) * cw as i32;
        let top = (offset_y.round() as i32 + self.content.top - self.bounds.top) * ch as i32;
        let columns = (self.bounds.right - self.bounds.left) as u32;
        let rows = (self.bounds.bottom - self.bounds.top) as u32;
        let (width, height) = (columns * cw, rows * ch);
        check_pixels(width, height).ok()?;
        let source = self.image.pixels.as_ref();
        let mut output = image::RgbaImage::new(width, height);
        let row_bytes = width as usize * 4;
        for (y, row) in output.chunks_exact_mut(row_bytes).enumerate() {
            let from_y = y as i32 - top;
            if from_y < 0 || from_y >= source.height() as i32 {
                continue;
            }
            // The columns of this row that the picture covers.
            let first = left.max(0);
            let last = (left + source.width() as i32).min(width as i32);
            if last <= first {
                continue;
            }
            let from = ((from_y as u32 * source.width()) as usize + (first - left) as usize) * 4;
            let count = (last - first) as usize * 4;
            row[first as usize * 4..first as usize * 4 + count]
                .copy_from_slice(&source.as_raw()[from..from + count]);
        }
        for (index, cover) in self.cover.iter().enumerate() {
            let visible = cover.visible || whole;
            if visible && cover.tint[3] == 0 {
                continue;
            }
            let (column, line) = (index as u32 % columns, index as u32 / columns);
            for y in line * ch..(line + 1) * ch {
                for x in column * cw..(column + 1) * cw {
                    let pixel = output.get_pixel_mut(x, y);
                    if !visible {
                        *pixel = image::Rgba([0; 4]);
                        continue;
                    }
                    for (channel, tint) in pixel.0[..3].iter_mut().zip(&cover.tint[..3]) {
                        *channel = ((u32::from(*channel) * (255 - u32::from(cover.tint[3]))
                            + u32::from(*tint) * u32::from(cover.tint[3])
                            + 127)
                            / 255) as u8;
                    }
                }
            }
        }
        Some(output)
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
    /// Whether the cell of the pixel at `x`, `y` shows the picture: no
    /// element above the plane covers it.
    pub fn shows(&self, x: u32, y: u32, cell: (u16, u16)) -> bool {
        let width = (self.bounds.right - self.bounds.left) as usize;
        self.cover[(y / u32::from(cell.1)) as usize * width + (x / u32::from(cell.0)) as usize]
            .visible
    }
    pub fn position(&self) -> (u32, u32) {
        (self.bounds.left as u32, self.bounds.top as u32)
    }
    /// Tell the canvas whose picture this is why a frame cannot show it.
    pub fn refuse(&self, reason: &str) {
        if let Some(refused) = &self.image.refused {
            refused(reason);
        }
    }
    /// The plane's columns and rows.
    pub fn cells(&self) -> (u32, u32) {
        (
            (self.bounds.right - self.bounds.left) as u32,
            (self.bounds.bottom - self.bounds.top) as u32,
        )
    }
    /// The cell at `x`, `y` shows another plane above this one, which takes
    /// it whole.
    fn hide(&mut self, x: i32, y: i32) {
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
        self.cover[index].visible = false;
    }
    /// Whether the plane still shows cell `(x, y)`: the cell is one of its
    /// and nothing painted over it covered the cell.
    #[cfg_attr(not(feature = "wgpu-graphics"), allow(dead_code))]
    pub(super) fn shows_cell(&self, x: i32, y: i32) -> bool {
        if x < self.bounds.left
            || x >= self.bounds.right
            || y < self.bounds.top
            || y >= self.bounds.bottom
        {
            return false;
        }
        let index = (y - self.bounds.top) as usize
            * (self.bounds.right - self.bounds.left) as usize
            + (x - self.bounds.left) as usize;
        self.cover[index].visible
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
        self.raster_with(cell, false)
    }

    /// The plane's pixels with the cells another plane above it takes
    /// kept, not cleared: what shows through a translucent picture made over
    /// this one on the picture thread (GFX-009).
    pub fn raster_under(&self, cell: (u16, u16)) -> Result<image::RgbaImage> {
        self.raster_with(cell, true)
    }

    fn raster_with(&self, cell: (u16, u16), whole: bool) -> Result<image::RgbaImage> {
        if let Some(picture) = self.canvas_raster(whole) {
            return Ok(picture);
        }
        // A canvas picture under a transform or an opacity its fast path
        // does not take is still rastered at its own pixels per cell, the
        // size its placement and its coverage are computed for (GFX-010).
        let (cw, ch) = {
            let cell = self.raster_cell(cell);
            (u32::from(cell.0), u32::from(cell.1))
        };
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
            if !cover.visible && !whole {
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
