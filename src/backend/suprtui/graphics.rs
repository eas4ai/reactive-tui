mod coverage;
mod maker;
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
    /// Whether the last `prepare` changed canvas pictures only and cleared
    /// nothing, so no cell needs writing for it but the `stale` ones
    /// (GFX-005).
    in_place: bool,
    /// Cells, as (column, row, columns, rows), that show a Sixel picture
    /// the last `prepare` retired. Sixel has no way to remove a picture but
    /// to write the cells under it.
    stale: Vec<(u32, u32, u32, u32)>,
    shared: shared::Pictures,
    /// Whether canvas pictures are made ready on the picture thread, which
    /// the App does not wait for (GFX-009).
    apart: bool,
    /// The picture thread, once a canvas's picture has been made ready.
    maker: Option<maker::Maker<P>>,
    /// The thread that made each canvas picture written since the last
    /// `take_made_on` ready, one name per picture (GFX-009).
    made_on: Vec<String>,
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
            stale: Vec::new(),
            shared: shared::Pictures::default(),
            apart: false,
            maker: None,
            made_on: Vec::new(),
        }
    }
}

/// How a plane of the next frame continues one of the last, which it names
/// by its place in the last frame's planes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    New,
    /// The same plane.
    Kept(usize),
    /// A canvas's next picture, in the same place with the same cells
    /// showing.
    InPlace(usize),
    Changed(usize),
}

impl Step {
    fn previous(self) -> Option<usize> {
        match self {
            Self::New => None,
            Self::Kept(index) | Self::InPlace(index) | Self::Changed(index) => Some(index),
        }
    }
}

pub(crate) trait RasterPlane: PartialEq + Clone + Send + 'static {
    fn id(&self) -> u32;
    fn protocol(&self) -> ImageProtocol;
    fn quality(&self) -> crate::widgets::ImageQuality;
    fn position(&self) -> (u32, u32);
    fn raster(&self, cell: (u16, u16)) -> Result<image::RgbaImage>;
    fn background(&self, x: u32, y: u32, cell: (u16, u16)) -> image::Rgba<u8>;
    /// Whether the cell of the pixel at `x`, `y` shows the picture: no
    /// element above it covers the cell.
    fn shows(&self, _x: u32, _y: u32, _cell: (u16, u16)) -> bool {
        true
    }
    /// The plane's columns and rows.
    fn cells(&self) -> (u32, u32) {
        (0, 0)
    }
    /// Tell the canvas whose picture this is why the frame cannot show it.
    fn refuse(&self, _reason: &str) {}
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
    fn shows(&self, x: u32, y: u32, cell: (u16, u16)) -> bool {
        self.shows(x, y, cell)
    }
    fn cells(&self) -> (u32, u32) {
        self.cells()
    }
    fn refuse(&self, reason: &str) {
        self.refuse(reason);
    }
}

impl<P: RasterPlane> Graphics<P> {
    /// Make canvas pictures ready on the picture thread from now on, which
    /// the App does not wait for (GFX-009).
    pub fn make_pictures_apart(&mut self) {
        self.apart = true;
    }
    /// The picture thread, started when the first canvas picture is made
    /// ready on it; `None` when pictures are made where `prepare` runs.
    fn maker(&mut self) -> Option<&maker::Maker<P>> {
        if !self.apart {
            return None;
        }
        if self.maker.is_none() {
            match maker::Maker::start() {
                Ok(maker) => self.maker = Some(maker),
                Err(error) => {
                    log::warn!("canvas pictures are made ready before present returns: {error}");
                    self.apart = false;
                    return None;
                }
            }
        }
        self.maker.as_ref()
    }
    pub fn prepare(
        &mut self,
        next: &[P],
        cell: (u16, u16),
        force: bool,
    ) -> Result<Option<(Vec<u8>, Vec<u8>)>> {
        self.candidate_coverage = None;
        self.in_place = false;
        self.stale.clear();
        if !force && next == self.last {
            return Ok(None);
        }
        // When only canvases differ from the last frame, nothing is
        // cleared: a canvas's next picture replaces the last where it is,
        // and a canvas that moved, went, or has other cells over it is
        // retired alone (GFX-005).
        let steps = self.steps(next);
        let in_place = !force && self.only_canvases_differ(next, &steps);
        let before = if in_place {
            self.retire(next, &steps, cell)
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
            let kept = match steps[z] {
                Step::Kept(index) if in_place => Some(index),
                _ => None,
            };
            if let Some(known) = kept
                .and_then(|index| self.coverage.get(index))
                .filter(|_| !blend_legacy)
            {
                coverage.push(known.clone());
                continue;
            }
            // A canvas's picture is made ready on the picture thread and
            // written when it is ready (GFX-009). Until then a canvas that
            // stays in its place keeps its last picture; one that is new,
            // moved or covered otherwise, or that this frame cleared,
            // shows none. Planes above it are blended without its pixels.
            if plane.canvas().is_some() && self.apart {
                let shown = match steps[z] {
                    Step::Kept(index) | Step::InPlace(index) if in_place => {
                        self.coverage.get(index).cloned()
                    }
                    _ => None,
                };
                if kept.is_none() {
                    let job = maker::Job {
                        plane: plane.clone(),
                        z,
                        cell,
                        blend_legacy,
                        below: below.clone(),
                    };
                    if let Some(maker) = self.maker() {
                        maker.submit(job);
                    }
                }
                coverage.push(shown.unwrap_or_else(|| coverage::Coverage::empty(plane.position())));
                continue;
            }
            let room = (64usize * 1024 * 1024).saturating_sub(after.len());
            let drawn = self.draw(
                plane,
                z,
                kept.is_some(),
                cell,
                blend_legacy,
                &mut below,
                &mut raster_bytes,
                room,
            );
            match drawn {
                Ok((covered, bytes)) => {
                    coverage.push(covered);
                    after.extend_from_slice(&bytes);
                }
                // A canvas's picture that the frame cannot hold is left
                // out and the canvas is told why (GFX-007).
                Err(error) if plane.canvas().is_some() => {
                    plane.refuse(&error.to_string());
                    let nothing = image::RgbaImage::new(0, 0);
                    coverage.push(coverage::Coverage::new(plane.position(), &nothing, cell));
                }
                Err(error) => return Err(error),
            }
        }
        if !next.is_empty() {
            after.extend_from_slice(b"\x1b8");
        }
        let canvases: Vec<u32> = next
            .iter()
            .filter(|plane| plane.canvas().is_some())
            .map(RasterPlane::id)
            .collect();
        self.shared.keep(&canvases);
        if let Some(maker) = &self.maker {
            maker.keep(canvases);
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
    /// The cells `plane` covers and the bytes that show it, of at most
    /// `room` bytes; no bytes for a plane that is `kept` as it is.
    #[allow(clippy::too_many_arguments)]
    fn draw(
        &mut self,
        plane: &P,
        z: usize,
        kept: bool,
        cell: (u16, u16),
        blend_legacy: bool,
        below: &mut PixelLayers,
        raster_bytes: &mut usize,
        room: usize,
    ) -> Result<(coverage::Coverage, Vec<u8>)> {
        let drawn = draw_plane(
            plane,
            z,
            kept,
            cell,
            blend_legacy,
            below,
            raster_bytes,
            room,
            &mut self.shared,
        )?;
        if plane.canvas().is_some() && !kept {
            let thread = std::thread::current();
            self.made_on
                .push(thread.name().unwrap_or_default().to_owned());
        }
        Ok(drawn)
    }
    /// Whether a canvas picture waits or is being made ready on the picture
    /// thread, so the worker looks for it soon (GFX-009).
    pub fn making(&self) -> bool {
        self.maker.as_ref().is_some_and(maker::Maker::making)
    }
    /// What shows each canvas picture the picture thread has made ready
    /// since the last call, for a picture-only update; empty when there is
    /// none. A picture made for cells its canvas no longer shows, because
    /// the canvas moved, went, or has other cells over it, is not written,
    /// and when no newer picture of the canvas waits, the canvas's picture
    /// is made ready again for the cells as they are (GFX-009). The flag
    /// says whether a picture was handed over again.
    pub fn take_made(&mut self, cell: (u16, u16)) -> (Vec<u8>, bool) {
        let Some(made) = self.maker.as_ref().map(maker::Maker::made) else {
            return (Vec::new(), false);
        };
        let mut again = false;
        let mut bytes = b"\x1b7".to_vec();
        for made in made {
            let Some(index) = self
                .last
                .iter()
                .position(|plane| plane.id() == made.plane.id())
            else {
                continue;
            };
            let current = &self.last[index];
            if *current != made.plane && !current.replaces(&made.plane) {
                if made.latest {
                    let job = maker::Job {
                        plane: current.clone(),
                        z: index,
                        cell,
                        blend_legacy: self.possible_legacy,
                        below: PixelLayers::new(cell),
                    };
                    if let Some(maker) = &self.maker {
                        maker.submit(job);
                        again = true;
                    }
                }
                continue;
            }
            if let Some(covered) = self.coverage.get_mut(index) {
                *covered = made.covered;
            }
            self.made_on.push(made.thread);
            bytes.extend_from_slice(&made.bytes);
        }
        if bytes.len() == 2 {
            return (Vec::new(), again);
        }
        bytes.extend_from_slice(b"\x1b8");
        (bytes, again)
    }
    /// The thread that made each canvas picture written since the last
    /// call ready, one name per picture (GFX-009).
    pub fn take_made_on(&mut self) -> Vec<String> {
        std::mem::take(&mut self.made_on)
    }
    /// Whether the last `prepare` changed canvas pictures only and cleared
    /// nothing: the cells stay as they are written, but the `stale` ones.
    pub fn in_place(&self) -> bool {
        self.in_place
    }
    /// The cells, as (column, row, columns, rows), that the renderer must
    /// write again because a Sixel picture the last `prepare` retired is
    /// drawn over them.
    pub fn stale(&self) -> &[(u32, u32, u32, u32)] {
        &self.stale
    }
    /// How each plane of `next` continues a plane of the last frame, which
    /// it is matched with by its id.
    fn steps(&self, next: &[P]) -> Vec<Step> {
        next.iter()
            .map(
                |plane| match self.last.iter().position(|last| last.id() == plane.id()) {
                    None => Step::New,
                    Some(index) if *plane == self.last[index] => Step::Kept(index),
                    Some(index) if plane.replaces(&self.last[index]) => Step::InPlace(index),
                    Some(index) => Step::Changed(index),
                },
            )
            .collect()
    }
    /// Whether every plane that comes, goes or changes is a canvas's, the
    /// planes that stay keep their order, and every picture drawn over
    /// cells (Sixel or inline) is a canvas's: then the frame needs no
    /// clearing.
    fn only_canvases_differ(&self, next: &[P], steps: &[Step]) -> bool {
        let canvas = |plane: &P| plane.canvas().is_some();
        let over_cells = |plane: &P| plane.protocol() != ImageProtocol::Kitty;
        let continued: Vec<usize> = steps.iter().filter_map(|step| step.previous()).collect();
        continued.windows(2).all(|pair| pair[0] < pair[1])
            && next.iter().zip(steps).all(|(plane, step)| {
                (canvas(plane) || !over_cells(plane))
                    && (matches!(step, Step::Kept(_))
                        || canvas(plane)
                            && step
                                .previous()
                                .is_none_or(|index| canvas(&self.last[index])))
            })
            && self.last.iter().enumerate().all(|(index, last)| {
                (canvas(last) || !over_cells(last)) && (continued.contains(&index) || canvas(last))
            })
    }
    /// Take the canvas pictures off the screen that the next frame does not
    /// replace where they are: the ones that are gone, and the ones that
    /// moved or have other cells over them. A Kitty picture is deleted,
    /// unless the next picture of its canvas replaces it by its id; the
    /// cells under a Sixel picture are listed in `stale`.
    fn retire(&mut self, next: &[P], steps: &[Step], cell: (u16, u16)) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (index, old) in self.last.iter().enumerate() {
            let follows = steps
                .iter()
                .position(|step| step.previous() == Some(index))
                .map(|z| (&next[z], steps[z]));
            if matches!(follows, Some((_, Step::Kept(_) | Step::InPlace(_)))) {
                continue;
            }
            if old.protocol() == ImageProtocol::Kitty {
                if follows.is_none_or(|(plane, _)| plane.protocol() != ImageProtocol::Kitty) {
                    bytes.extend_from_slice(
                        format!("\x1b_Ga=d,d=I,i={},q=2;\x1b\\", old.id()).as_bytes(),
                    );
                }
                continue;
            }
            let (left, top) = old.position();
            let (columns, rows) = old.cells();
            let (across, down) = (u32::from(cell.0), u32::from(cell.1));
            for row in 0..rows {
                let mut column = 0;
                while column < columns {
                    let shown = |column: u32| old.shows(column * across, row * down, cell);
                    if !shown(column) {
                        column += 1;
                        continue;
                    }
                    let first = column;
                    while column < columns && shown(column) {
                        column += 1;
                    }
                    self.stale
                        .push((left + first, top + row, column - first, 1));
                }
            }
        }
        bytes
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
        if let Some(maker) = &self.maker {
            maker.forget();
        }
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

/// The cells `plane` covers and the bytes that show it, of at most `room`
/// bytes; no bytes for a plane that is `kept` as it is. A canvas's Kitty
/// picture goes through the shared-memory objects of `shared` where the
/// host reads them.
#[allow(clippy::too_many_arguments)]
fn draw_plane<P: RasterPlane>(
    plane: &P,
    z: usize,
    kept: bool,
    cell: (u16, u16),
    blend_legacy: bool,
    below: &mut PixelLayers,
    raster_bytes: &mut usize,
    room: usize,
    shared: &mut shared::Pictures,
) -> Result<(coverage::Coverage, Vec<u8>)> {
    let pixels = plane.raster(cell)?;
    let rastered = raster_bytes.saturating_add(pixels.as_raw().len());
    if rastered > 64 * 1024 * 1024 {
        return Err(ReactiveError::resource(
            "image frame exceeds 64 MiB raster limit",
        ));
    }
    *raster_bytes = rastered;
    let (x, y) = plane.position();
    let covered = coverage::Coverage::new((x, y), &pixels, cell);
    if kept {
        below.add(plane, &pixels);
        return Ok((covered, Vec::new()));
    }
    let output = match plane.protocol() {
        ImageProtocol::Kitty => plane
            .canvas()
            .filter(|picture| picture.shared_memory)
            .and_then(|_| shared.kitty(&pixels, plane.id(), plane.z_index(z)))
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
            let mut pixels = below.flatten(plane, &pixels, true);
            // Sixel leaves a pixel it is not given as the screen
            // shows it, which after the first picture is the last
            // picture. So every pixel of a cell that shows the
            // canvas is drawn: where the picture is transparent,
            // in the cell's background.
            for (x, y, pixel) in pixels.enumerate_pixels_mut() {
                if pixel[3] == 0 && plane.shows(x, y, cell) {
                    *pixel = plane.background(x, y, cell);
                }
            }
            let height = pixels.height() - pixels.height() % 6;
            if height == 0 {
                String::new()
            } else {
                let pixels =
                    image::imageops::crop_imm(&pixels, 0, 0, pixels.width(), height).to_image();
                SixelRenderer::encode_picture(&pixels)?
            }
        }
        ImageProtocol::Sixel => {
            let pixels = below.flatten(plane, &pixels, true);
            let pixels = below.absolute(plane, &pixels)?;
            let rastered = raster_bytes.saturating_add(pixels.as_raw().len());
            if rastered > 64 * 1024 * 1024 {
                return Err(ReactiveError::resource(
                    "image frame exceeds 64 MiB raster limit",
                ));
            }
            *raster_bytes = rastered;
            SixelRenderer::encode_pixels(&pixels, plane.quality())?
        }
    };
    if output.len() > room {
        return Err(ReactiveError::resource(
            "image frame exceeds 64 MiB output limit",
        ));
    }
    if blend_legacy {
        below.add(plane, &pixels);
    }
    let mut bytes = format!("\x1b[{};{}H", y + 1, x + 1).into_bytes();
    // Sixel display mode (80) draws from the screen's corner; a
    // canvas's picture is drawn from the cursor instead, which
    // stays beside the picture (8452) so the screen never scrolls.
    let sixel_modes: (&[u8], &[u8]) = match (plane.protocol(), plane.canvas()) {
        (ImageProtocol::Sixel, None) => (b"\x1b[?80s\x1b[?80h", b"\x1b[?80r"),
        (ImageProtocol::Sixel, Some(_)) => {
            (b"\x1b[?80;8452s\x1b[?80l\x1b[?8452h", b"\x1b[?80;8452r")
        }
        _ => (b"", b""),
    };
    bytes.extend_from_slice(sixel_modes.0);
    bytes.extend_from_slice(output.as_bytes());
    bytes.extend_from_slice(sixel_modes.1);
    Ok((covered, bytes))
}

/// Sparse cell tiles keep alpha composition proportional to painted pixels,
/// rather than walking every preceding image for each translucent pixel.
#[derive(Clone)]
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
