mod coverage;
mod shared;
use crate::{
    error::{ReactiveError, Result},
    layout::paint_tree::suprtui::images::Plane,
    widgets::display::image::{
        decoded::blend_pixel,
        paint::{CanvasPicture, ImageProtocol},
        ProtocolRenderer, SixelRenderer,
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
    /// Whether the last `prepare` only replaced canvas pictures where they
    /// are, so no cell needs writing for it (GFX-005).
    in_place: bool,
    shared: shared::Pictures,
}
impl<P> Default for Graphics<P> {
    fn default() -> Self {
        Self {
            last: Vec::new(),
            possible_ids: BTreeSet::new(),
            possible_legacy: false,
            coverage: Vec::new(),
            candidate_coverage: None,
            in_place: false,
            shared: shared::Pictures::default(),
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
    /// How the picture is sent, when it is a canvas's (GFX-005).
    fn canvas(&self) -> Option<CanvasPicture> {
        None
    }
    /// Whether this is the next picture of the canvas `previous` shows, in
    /// the same place.
    fn replaces(&self, _previous: &Self) -> bool {
        false
    }
}

impl RasterPlane for Plane {
    fn canvas(&self) -> Option<CanvasPicture> {
        self.canvas()
    }
    fn replaces(&self, previous: &Self) -> bool {
        self.replaces(previous)
    }
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
        self.in_place = false;
        if !force && next == self.last {
            return Ok(None);
        }
        // Planes that differ from the last frame's only by a canvas's next
        // picture are replaced where they are: nothing is deleted, the
        // screen is not cleared and no cell is written (GFX-005).
        let in_place = !force
            && next.len() == self.last.len()
            && next
                .iter()
                .zip(&self.last)
                .all(|(plane, last)| plane == last || plane.replaces(last));
        let before = if in_place {
            Vec::new()
        } else {
            self.cleanup()
        };
        let mut after = Vec::new();
        let blend_legacy = next.iter().any(|p| p.protocol() != ImageProtocol::Kitty);
        let mut below = PixelLayers::new(cell);
        let mut raster_bytes = 0usize;
        let mut coverage = Vec::with_capacity(next.len());
        if !next.is_empty() {
            after.extend_from_slice(b"\x1b7");
        }
        for (z, plane) in next.iter().enumerate() {
            // A plane that stays as it is needs its pixels again only
            // where a plane above it is blended with them.
            let kept = in_place && *plane == self.last[z];
            if let Some(known) = self.coverage.get(z).filter(|_| kept && !blend_legacy) {
                coverage.push(known.clone());
                continue;
            }
            let pixels = plane.raster(cell)?;
            raster_bytes = raster_bytes.saturating_add(pixels.as_raw().len());
            if raster_bytes > 64 * 1024 * 1024 {
                return Err(ReactiveError::resource(
                    "image frame exceeds 64 MiB raster limit",
                ));
            }
            let (x, y) = plane.position();
            coverage.push(coverage::Coverage::new((x, y), &pixels, cell));
            if kept {
                below.add(plane, &pixels);
                continue;
            }
            let output = match plane.protocol() {
                ImageProtocol::Kitty => plane
                    .canvas()
                    .filter(|picture| picture.shared_memory)
                    .and_then(|_| self.shared.kitty(&pixels, plane.id(), plane.z_index(z)))
                    .unwrap_or_else(|| {
                        ProtocolRenderer::kitty_pixels(&pixels, plane.id(), plane.z_index(z), true)
                    }),
                ImageProtocol::Inline => {
                    ProtocolRenderer::iterm_pixels(&below.flatten(plane, &pixels, false), false)?
                }
                ImageProtocol::Sixel if plane.canvas().is_some() => {
                    // A canvas's picture is drawn from the cursor, so it
                    // touches no cell outside the canvas, and only in whole
                    // bands of six rows, so none reaches below it.
                    let pixels = below.flatten(plane, &pixels, true);
                    let height = pixels.height() - pixels.height() % 6;
                    if height == 0 {
                        String::new()
                    } else {
                        let pixels =
                            image::imageops::crop_imm(&pixels, 0, 0, pixels.width(), height)
                                .to_image();
                        SixelRenderer::encode_pixels(&pixels, plane.quality())?
                    }
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
            // Sixel display mode (80) draws from the screen's corner; a
            // canvas's picture is drawn from the cursor instead, which
            // stays beside the picture (8452) so the screen never scrolls.
            let sixel_modes: (&[u8], &[u8]) = match (plane.protocol(), plane.canvas()) {
                (ImageProtocol::Sixel, None) => (b"\x1b[?80s\x1b[?80h", b"\x1b[?80r"),
                (ImageProtocol::Sixel, Some(_)) => (
                    b"\x1b[?80;8452s\x1b[?80l\x1b[?8452h",
                    b"\x1b[?80;8452r",
                ),
                _ => (b"", b""),
            };
            after.extend_from_slice(sixel_modes.0);
            after.extend_from_slice(output.as_bytes());
            after.extend_from_slice(sixel_modes.1);
        }
        if !next.is_empty() {
            after.extend_from_slice(b"\x1b8");
        }
        self.in_place = in_place;
        self.possible_ids.extend(
            next.iter()
                .filter(|p| p.protocol() == ImageProtocol::Kitty)
                .map(RasterPlane::id),
        );
        self.possible_legacy |= next.iter().any(|p| p.protocol() != ImageProtocol::Kitty);
        self.candidate_coverage = Some(coverage);
        Ok(Some((before, after)))
    }
    /// Whether the last `prepare` only replaced canvas pictures where they
    /// are: the cells stay as they are written.
    pub fn in_place(&self) -> bool {
        self.in_place
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
        self.shared.forget();
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
