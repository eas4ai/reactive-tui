mod coverage;
use crate::{
    error::{ReactiveError, Result},
    layout::paint_tree::suprtui::images::Plane,
    widgets::display::image::{
        decoded::blend_pixel, paint::ImageProtocol, ProtocolRenderer, SixelRenderer,
    },
};
use std::collections::{BTreeSet, HashMap};

/// Images are acknowledged only after the frame's checked write and flush.
pub(crate) struct Graphics<P = Plane> {
    last: Vec<P>,
    possible_ids: BTreeSet<u32>,
    possible_legacy: bool,
    coverage: Vec<coverage::Coverage>,
    candidate_coverage: Option<Vec<coverage::Coverage>>,
}
impl<P> Default for Graphics<P> {
    fn default() -> Self {
        Self {
            last: Vec::new(),
            possible_ids: BTreeSet::new(),
            possible_legacy: false,
            coverage: Vec::new(),
            candidate_coverage: None,
        }
    }
}

pub(crate) trait RasterPlane: PartialEq {
    fn id(&self) -> u32;
    fn protocol(&self) -> ImageProtocol;
    fn quality(&self) -> crate::widgets::ImageQuality;
    fn position(&self) -> (u32, u32);
    fn raster(&self, cell: (u16, u16)) -> Result<image::RgbaImage>;
    fn background(&self, x: u32, y: u32, cell: (u16, u16)) -> image::Rgba<u8>;
    fn z_index(&self, order: usize) -> i32 {
        order as i32
    }
}

impl RasterPlane for Plane {
    fn id(&self) -> u32 {
        self.id()
    }
    fn protocol(&self) -> ImageProtocol {
        self.protocol()
    }
    fn quality(&self) -> crate::widgets::ImageQuality {
        self.quality()
    }
    fn position(&self) -> (u32, u32) {
        self.position()
    }
    fn raster(&self, cell: (u16, u16)) -> Result<image::RgbaImage> {
        self.raster(cell)
    }
    fn background(&self, x: u32, y: u32, cell: (u16, u16)) -> image::Rgba<u8> {
        self.background(x, y, cell)
    }
}

impl<P: RasterPlane> Graphics<P> {
    pub fn prepare(
        &mut self,
        next: &[P],
        cell: (u16, u16),
        force: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        self.candidate_coverage = None;
        if !force && next == self.last {
            return Ok(None);
        }
        let before = self.cleanup();
        let mut after = Vec::new();
        let blend_legacy = next.iter().any(|p| p.protocol() != ImageProtocol::Kitty);
        let mut below = PixelLayers::new(cell);
        let mut raster_bytes = 0usize;
        let mut coverage = Vec::with_capacity(next.len());
        if !next.is_empty() {
            after.extend_from_slice(b"\x1b7");
        }
        for (z, plane) in next.iter().enumerate() {
            let pixels = plane.raster(cell)?;
            raster_bytes = raster_bytes.saturating_add(pixels.as_raw().len());
            if raster_bytes > 64 * 1024 * 1024 {
                return Err(ReactiveError::resource(
                    "image frame exceeds 64 MiB raster limit",
                ));
            }
            let (x, y) = plane.position();
            coverage.push(coverage::Coverage::new((x, y), &pixels, cell));
            let output = match plane.protocol() {
                ImageProtocol::Kitty => {
                    ProtocolRenderer::kitty_pixels(&pixels, plane.id(), plane.z_index(z), true)
                }
                ImageProtocol::Inline => {
                    ProtocolRenderer::iterm_pixels(&below.flatten(plane, &pixels, false), false)?
                }
                ImageProtocol::Sixel => {
                    let pixels = below.flatten(plane, &pixels, true);
                    let pixels = below.absolute(plane, &pixels)?;
                    raster_bytes = raster_bytes.saturating_add(pixels.as_raw().len());
                    if raster_bytes > 64 * 1024 * 1024 {
                        return Err(ReactiveError::resource(
                            "image frame exceeds 64 MiB raster limit",
                        ));
                    }
                    SixelRenderer::encode_pixels(&pixels, plane.quality())?
                }
            };
            if blend_legacy {
                below.add(plane, &pixels);
            }
            if after.len().saturating_add(output.len()) > 64 * 1024 * 1024 {
                return Err(ReactiveError::resource(
                    "image frame exceeds 64 MiB output limit",
                ));
            }
            after.extend_from_slice(format!("\x1b[{};{}H", y + 1, x + 1).as_bytes());
            if plane.protocol() == ImageProtocol::Sixel {
                after.extend_from_slice(b"\x1b[?80s\x1b[?80h");
            }
            after.extend_from_slice(output.as_bytes());
            if plane.protocol() == ImageProtocol::Sixel {
                after.extend_from_slice(b"\x1b[?80r");
            }
        }
        if !next.is_empty() {
            after.extend_from_slice(b"\x1b8");
        }
        self.possible_ids.extend(
            next.iter()
                .filter(|p| p.protocol() == ImageProtocol::Kitty)
                .map(RasterPlane::id),
        );
        self.possible_legacy |= next.iter().any(|p| p.protocol() != ImageProtocol::Kitty);
        self.candidate_coverage = Some(coverage);
        Ok(Some((before, after)))
    }
    pub fn covers_cell(&self, x: u32, y: u32) -> bool {
        self.candidate_coverage
            .as_ref()
            .unwrap_or(&self.coverage)
            .iter()
            .any(|mask| mask.contains(x, y))
    }
    pub fn acknowledge(&mut self, next: Vec<P>) {
        if let Some(coverage) = self.candidate_coverage.take() {
            self.coverage = coverage;
        }
        self.possible_ids = next
            .iter()
            .filter(|p| p.protocol() == ImageProtocol::Kitty)
            .map(RasterPlane::id)
            .collect();
        self.possible_legacy = next.iter().any(|p| p.protocol() != ImageProtocol::Kitty);
        self.last = next;
    }
    pub fn cleanup(&self) -> Vec<u8> {
        let mut bytes = if self.possible_legacy {
            b"\x1b[2J".to_vec()
        } else {
            Vec::new()
        };
        bytes.extend(
            self.possible_ids
                .iter()
                .flat_map(|id| format!("\x1b_Ga=d,d=I,i={id},q=2;\x1b\\").into_bytes())
                .collect::<Vec<_>>(),
        );
        bytes
    }
}

/// Sparse cell tiles keep alpha composition proportional to painted pixels,
/// rather than walking every preceding image for each translucent pixel.
struct PixelLayers {
    cell: (u32, u32),
    tiles: HashMap<(u32, u32), Vec<image::Rgba<u8>>>,
}
impl PixelLayers {
    /// Display-mode Sixel starts at the screen origin and never scrolls text.
    /// Retain preceding image pixels in the padded region because some hosts
    /// replace whole image cells even when the new pixels are transparent.
    fn absolute(
        &self,
        plane: &impl RasterPlane,
        pixels: &image::RgbaImage,
    ) -> Result<image::RgbaImage> {
        let (left, top) = plane.position();
        let left = left.checked_mul(self.cell.0);
        let top = top.checked_mul(self.cell.1);
        let (Some(left), Some(top)) = (left, top) else {
            return Err(ReactiveError::resource("Sixel position overflow"));
        };
        let width = left.checked_add(pixels.width());
        let height = top.checked_add(pixels.height());
        let (Some(width), Some(height)) = (width, height) else {
            return Err(ReactiveError::resource("Sixel dimensions overflow"));
        };
        if u64::from(width) * u64::from(height) > 16 * 1024 * 1024 {
            return Err(ReactiveError::resource(
                "Sixel placement exceeds 64 MiB raster limit",
            ));
        }
        Ok(image::RgbaImage::from_fn(width, height, |x, y| {
            if x >= left && y >= top {
                *pixels.get_pixel(x - left, y - top)
            } else {
                let key = (x / self.cell.0, y / self.cell.1);
                let offset = ((y % self.cell.1) * self.cell.0 + x % self.cell.0) as usize;
                self.tiles
                    .get(&key)
                    .map_or(image::Rgba([0; 4]), |tile| tile[offset])
            }
        }))
    }
    fn new(cell: (u16, u16)) -> Self {
        Self {
            cell: (u32::from(cell.0), u32::from(cell.1)),
            tiles: HashMap::new(),
        }
    }
    fn address(&self, plane: &impl RasterPlane, x: u32, y: u32) -> ((u32, u32), usize) {
        let (left, top) = plane.position();
        (
            (left + x / self.cell.0, top + y / self.cell.1),
            ((y % self.cell.1) * self.cell.0 + x % self.cell.0) as usize,
        )
    }
    fn add(&mut self, plane: &impl RasterPlane, pixels: &image::RgbaImage) {
        for (x, y, pixel) in pixels.enumerate_pixels().filter(|(_, _, p)| p[3] != 0) {
            let (key, offset) = self.address(plane, x, y);
            let tile = self
                .tiles
                .entry(key)
                .or_insert_with(|| vec![image::Rgba([0; 4]); (self.cell.0 * self.cell.1) as usize]);
            blend_pixel(&mut tile[offset], pixel);
            if tile[offset][3] != 255 {
                let mut background =
                    plane.background(x, y, (self.cell.0 as u16, self.cell.1 as u16));
                blend_pixel(&mut background, &tile[offset]);
                tile[offset] = background;
            }
        }
    }
    fn flatten(
        &self,
        plane: &impl RasterPlane,
        pixels: &image::RgbaImage,
        opaque: bool,
    ) -> image::RgbaImage {
        image::RgbaImage::from_fn(pixels.width(), pixels.height(), |x, y| {
            let pixel = pixels.get_pixel(x, y);
            let (key, offset) = self.address(plane, x, y);
            let mut composite = self
                .tiles
                .get(&key)
                .map_or(image::Rgba([0; 4]), |tile| tile[offset]);
            blend_pixel(&mut composite, pixel);
            if !opaque || composite[3] == 0 || composite[3] == 255 {
                return composite;
            }
            let mut color = plane.background(x, y, (self.cell.0 as u16, self.cell.1 as u16));
            blend_pixel(&mut color, &composite);
            color
        })
    }
}

#[cfg(test)]
mod tests;
