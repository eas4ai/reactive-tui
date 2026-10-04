//! Pixel looks (docs/spec/pixel-looks.md): a widget's or an element's
//! appearance drawn as one picture over the rectangle it paints where the
//! terminal takes pixels, around its text, with its cell look kept as the
//! fallback.
//!
//! A [`Look`] names what the picture holds: a fill in a theme role with
//! rounded corners, a border and a ring along the edge, and the shapes a
//! control adds (a check mark, a dot, a track, a thumb). The painter builds
//! the scene for the rectangle's size in pixels (PIX-001), keeps one drawing
//! slot and one image per look across frames in [`Looks`], and draws a look
//! again only when what drew it changed (PIX-002). An element gets a look
//! from its `rounded-*`, `border` and `ring-*` classes with a background
//! role (PIX-003), or from the control that paints it (PIX-004, PIX-005).

use super::worker::{Job, Picture, Want};
use super::{
    CanvasOutput, Color, GraphicsFrame, GraphicsWorker, HostReport, Paint, Path, Scene, Stroke,
    Transform,
};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// The pixels a size names at a cell height of 16; other cell heights
/// scale it in proportion (PIX-003).
const REFERENCE_CELL_HEIGHT: f32 = 16.0;

/// The radius of a look's corners.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Radius {
    /// Square corners.
    #[default]
    None,
    /// That many pixels at a cell height of 16.
    Px(f32),
    /// Half the box's shorter side: a pill or a circle.
    Full,
}

/// A line along a look's edge, inside it: a border or a ring, `width`
/// pixels wide at every cell size, in a theme role (PIX-003, PIX-004).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Its width in pixels.
    pub width: f32,
    /// The theme role it is drawn in.
    pub role: String,
}

impl Line {
    /// A line `width` pixels wide in `role`.
    pub fn new(width: f32, role: impl Into<String>) -> Self {
        Self {
            width,
            role: role.into(),
        }
    }
}

/// A size a control's shape takes, in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Extent {
    /// That many pixels at every cell size: a check mark's stroke, a
    /// slider's track (PIX-004, PIX-005).
    Px(f32),
    /// `times` the cell height less `less` pixels: a checkbox's box, a
    /// radio's circle and dot, a slider's thumb (PIX-004, PIX-005).
    CellLess {
        /// The pixels taken off the cell height.
        less: f32,
        /// What the rest is multiplied by.
        times: f32,
    },
    /// That many pixels at a cell height of 16, in proportion otherwise: a
    /// progress bar over its rows.
    Scaled(f32),
}

impl Extent {
    /// The cell height less two pixels.
    pub const CELL_LESS_TWO: Self = Self::CellLess {
        less: 2.0,
        times: 1.0,
    };

    /// The size in pixels at a cell height of `cell_height` pixels; never
    /// less than one.
    fn px(self, cell_height: f32) -> f32 {
        match self {
            Self::Px(px) => px,
            Self::CellLess { less, times } => (cell_height - less) * times,
            Self::Scaled(px) => px * cell_height / REFERENCE_CELL_HEIGHT,
        }
        .max(1.0)
    }
}

/// A shape a control draws inside its look. Positions are in cells of the
/// look's rectangle (fractions allowed), sizes as each [`Extent`] says.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// A rounded box centered at `center`, `size` square.
    Box {
        /// The box's center, in cells.
        center: (f32, f32),
        /// Its side.
        size: Extent,
        /// Its corner radius.
        radius: Extent,
        /// The role it is filled with, if any.
        fill: Option<String>,
        /// Its border, if any.
        border: Option<Line>,
    },
    /// A circle centered at `center` of `diameter`.
    Circle {
        /// The circle's center, in cells.
        center: (f32, f32),
        /// Its diameter.
        diameter: Extent,
        /// The role it is filled with, if any.
        fill: Option<String>,
        /// Its border, if any.
        border: Option<Line>,
    },
    /// A bar from `from` to `to` cells across, `height` tall, centered on
    /// row `row`, with rounded ends of `radius`.
    Bar {
        /// Where the bar starts, in cells across.
        from: f32,
        /// Where it ends, in cells across.
        to: f32,
        /// The row its middle lies on, in cells down.
        row: f32,
        /// Its height.
        height: Extent,
        /// The radius of its ends.
        radius: Extent,
        /// The role it is filled with.
        fill: String,
    },
    /// A check mark centered at `center`, `size` wide, `width` thick.
    Check {
        /// The mark's center, in cells.
        center: (f32, f32),
        /// Its width.
        size: Extent,
        /// Its stroke.
        width: Extent,
        /// The role it is drawn in.
        role: String,
    },
    /// A dash centered at `center`, `size` wide, `width` thick.
    Dash {
        /// The dash's center, in cells.
        center: (f32, f32),
        /// Its width.
        size: Extent,
        /// Its stroke.
        width: Extent,
        /// The role it is drawn in.
        role: String,
    },
}

/// What a look's picture holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Look {
    /// The theme role the box is filled with, if any.
    pub fill: Option<String>,
    /// The fill's opacity over what is under the element: 1 is opaque.
    pub fill_opacity: f32,
    /// The whole look's opacity over what is under the element: a disabled
    /// control is drawn at half (PIX-004, PIX-005).
    pub opacity: f32,
    /// The corners' radius.
    pub radius: Radius,
    /// A line along the edge, inside the ring when both are drawn.
    pub border: Option<Line>,
    /// A line at the edge: the focus ring.
    pub ring: Option<Line>,
    /// What a control draws inside the box.
    pub shapes: Vec<Shape>,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            fill: None,
            fill_opacity: 1.0,
            opacity: 1.0,
            radius: Radius::None,
            border: None,
            ring: None,
            shapes: Vec::new(),
        }
    }
}

/// The radius a `rounded*` class names, in pixels at a cell height of 16
/// (Tailwind's numbers), or `Radius::Full`.
fn radius_of(token: &str) -> Option<Radius> {
    Some(match token {
        "rounded" | "rounded-md" => Radius::Px(if token == "rounded" { 4.0 } else { 6.0 }),
        "rounded-sm" => Radius::Px(2.0),
        "rounded-lg" => Radius::Px(8.0),
        "rounded-xl" => Radius::Px(12.0),
        "rounded-2xl" => Radius::Px(16.0),
        "rounded-3xl" => Radius::Px(24.0),
        "rounded-full" => Radius::Full,
        "rounded-none" => Radius::None,
        _ => return None,
    })
}

impl Look {
    /// The look an element's classes describe, when they describe one: a
    /// `bg-R` role with a `rounded*` class; `border` and `border-R` add a
    /// one-pixel border (or `border-2`, two), `ring-1` and `ring-2` with
    /// `ring-R` a ring, and `bg-opacity-N` the fill's opacity. State
    /// variants (`focus:`, `hover:`, `disabled:`) are already stripped from
    /// the classes of an element in that state when the painter reads them.
    pub fn from_classes(classes: &str) -> Option<Self> {
        let mut look = Look::default();
        let mut rounded = false;
        let mut border_width: Option<f32> = None;
        let mut border_role = None;
        let mut ring_width: Option<f32> = None;
        let mut ring_role = None;
        for token in classes.split_whitespace() {
            if let Some(radius) = radius_of(token) {
                look.radius = radius;
                rounded = true;
            } else if let Some(percent) = token.strip_prefix("bg-opacity-") {
                if let Ok(percent) = percent.parse::<f32>() {
                    look.fill_opacity = (percent / 100.0).clamp(0.0, 1.0);
                }
            } else if let Some(role) = token.strip_prefix("bg-") {
                if !role.is_empty() {
                    look.fill = Some(role.to_owned());
                }
            } else if token == "border" {
                border_width.get_or_insert(1.0);
            } else if let Some(rest) = token.strip_prefix("border-") {
                match rest.parse::<f32>() {
                    Ok(width) => border_width = Some(width),
                    Err(_) if rest.is_empty() => {}
                    Err(_) => border_role = Some(rest.to_owned()),
                }
            } else if let Some(rest) = token.strip_prefix("ring-") {
                match rest.parse::<f32>() {
                    Ok(width) => ring_width = Some(width),
                    Err(_) => ring_role = Some(rest.to_owned()),
                }
            } else if token == "ring" {
                ring_width.get_or_insert(2.0);
            }
        }
        if look.fill.is_none() || !rounded {
            return None;
        }
        if let Some(width) = border_width.filter(|width| *width > 0.0) {
            look.border = Some(Line::new(
                width,
                border_role.unwrap_or_else(|| "border".to_owned()),
            ));
        }
        if let Some(width) = ring_width.filter(|width| *width > 0.0) {
            look.ring = Some(Line::new(
                width,
                ring_role.unwrap_or_else(|| "ring".to_owned()),
            ));
        }
        Some(look)
    }

    /// A number that tells one look from another.
    pub(crate) fn key(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        format!("{self:?}").hash(&mut hasher);
        hasher.finish()
    }

    /// The scale of a pixel size: cell height over 16.
    fn scale(cell: (u16, u16)) -> f32 {
        f32::from(cell.1.max(1)) / REFERENCE_CELL_HEIGHT
    }

    /// The corner radius in pixels of a box of `width` by `height` pixels.
    fn radius_px(&self, width: f32, height: f32, cell: (u16, u16)) -> f32 {
        let half = (width.min(height) / 2.0).max(0.0);
        match self.radius {
            Radius::None => 0.0,
            Radius::Px(px) => (px * Self::scale(cell)).min(half),
            Radius::Full => half,
        }
    }

    /// The scene of this look for a picture of `size` pixels over cells of
    /// `cell` pixels (PIX-001): the fill, the ring at the edge, the border
    /// inside it, then the control's shapes. Colors are theme tokens the
    /// renderer resolves when it draws (GFX-001).
    pub(crate) fn scene(&self, size: (u32, u32), cell: (u16, u16)) -> Scene {
        let (width, height) = (size.0 as f32, size.1 as f32);
        let radius = self.radius_px(width, height, cell);
        let mut scene = Scene::new();
        if let Some(fill) = &self.fill {
            scene.fill(
                &Path::rounded_rect(0.0, 0.0, width, height, radius),
                &Paint::solid(Color::token(fill.as_str()))
                    .opacity(self.fill_opacity * self.opacity),
            );
        }
        let mut inset = 0.0;
        for line in [&self.ring, &self.border].into_iter().flatten() {
            // A border is one pixel and a ring one or two at every cell size
            // (PIX-003); only the radius follows the cell height.
            let line_width = line.width.max(1.0).round();
            let half = inset + line_width / 2.0;
            if width - 2.0 * half <= 0.0 || height - 2.0 * half <= 0.0 {
                break;
            }
            scene.stroke(
                &Path::rounded_rect(
                    half,
                    half,
                    width - 2.0 * half,
                    height - 2.0 * half,
                    (radius - half).max(0.0),
                ),
                &Stroke::new(line_width),
                &Paint::solid(Color::token(line.role.as_str())).opacity(self.opacity),
            );
            inset += line_width;
        }
        for shape in &self.shapes {
            shape.draw(&mut scene, cell, self.opacity);
        }
        scene
    }
}

impl Shape {
    fn draw(&self, scene: &mut Scene, cell: (u16, u16), opacity: f32) {
        let (cw, ch) = (f32::from(cell.0.max(1)), f32::from(cell.1.max(1)));
        let at = |(x, y): (f32, f32)| (x * cw, y * ch);
        let px = |size: &Extent| size.px(ch);
        // A border is one pixel wide at every cell size (PIX-004).
        let line_px = |line: &Line| line.width.max(1.0).round();
        match self {
            Shape::Box {
                center,
                size,
                radius,
                fill,
                border,
            } => {
                let (cx, cy) = at(*center);
                let side = px(size).round();
                let path = Path::rounded_rect(
                    cx - side / 2.0,
                    cy - side / 2.0,
                    side,
                    side,
                    px(radius).min(side / 2.0),
                );
                if let Some(fill) = fill {
                    scene.fill(
                        &path,
                        &Paint::solid(Color::token(fill.as_str())).opacity(opacity),
                    );
                }
                if let Some(line) = border {
                    let width = line_px(line);
                    let inner = Path::rounded_rect(
                        cx - side / 2.0 + width / 2.0,
                        cy - side / 2.0 + width / 2.0,
                        side - width,
                        side - width,
                        (px(radius) - width / 2.0).max(0.0),
                    );
                    scene.stroke(
                        &inner,
                        &Stroke::new(width),
                        &Paint::solid(Color::token(line.role.as_str())).opacity(opacity),
                    );
                }
            }
            Shape::Circle {
                center,
                diameter,
                fill,
                border,
            } => {
                let (cx, cy) = at(*center);
                let r = px(diameter) / 2.0;
                if let Some(fill) = fill {
                    scene.fill(
                        &Path::ellipse(cx, cy, r, r),
                        &Paint::solid(Color::token(fill.as_str())).opacity(opacity),
                    );
                }
                if let Some(line) = border {
                    let width = line_px(line);
                    scene.stroke(
                        &Path::ellipse(cx, cy, r - width / 2.0, r - width / 2.0),
                        &Stroke::new(width),
                        &Paint::solid(Color::token(line.role.as_str())).opacity(opacity),
                    );
                }
            }
            Shape::Bar {
                from,
                to,
                row,
                height,
                radius,
                fill,
            } => {
                let (x0, x1) = (from * cw, to * cw);
                let h = px(height).round();
                let y = row * ch - h / 2.0;
                if x1 > x0 {
                    scene.fill(
                        &Path::rounded_rect(
                            x0,
                            y,
                            x1 - x0,
                            h,
                            px(radius).min(h / 2.0).min((x1 - x0) / 2.0),
                        ),
                        &Paint::solid(Color::token(fill.as_str())).opacity(opacity),
                    );
                }
            }
            Shape::Check {
                center,
                size,
                width,
                role,
            } => {
                let (cx, cy) = at(*center);
                let s = px(size);
                let path = super::PathBuilder::new()
                    .move_to(cx - s * 0.45, cy)
                    .line_to(cx - s * 0.1, cy + s * 0.35)
                    .line_to(cx + s * 0.5, cy - s * 0.35)
                    .build();
                scene.stroke(
                    &path,
                    &Stroke::new(px(width).round())
                        .cap(super::LineCap::Round)
                        .join(super::LineJoin::Round),
                    &Paint::solid(Color::token(role.as_str())).opacity(opacity),
                );
            }
            Shape::Dash {
                center,
                size,
                width,
                role,
            } => {
                let (cx, cy) = at(*center);
                let s = px(size);
                let path = super::PathBuilder::new()
                    .move_to(cx - s * 0.4, cy)
                    .line_to(cx + s * 0.4, cy)
                    .build();
                scene.stroke(
                    &path,
                    &Stroke::new(px(width).round()).cap(super::LineCap::Round),
                    &Paint::solid(Color::token(role.as_str())).opacity(opacity),
                );
            }
        }
    }
}

/// What a look's slot last drew for.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Drawn {
    look: u64,
    size: (u32, u32),
    cell: (u16, u16),
    theme: u64,
}

/// One look's picture on the drawing thread, kept across frames.
struct LookSlot {
    picture: Picture,
    image_id: u32,
    submitted: Option<Drawn>,
    /// The newest finished picture that matches what was submitted.
    shown: Option<(Drawn, Arc<GraphicsFrame>)>,
    seen: u64,
    /// Why a frame last left the look's picture out, told by the backend.
    refused: Arc<std::sync::Mutex<Option<String>>>,
    /// The budget the output taught the look when a frame refused its
    /// picture for the room its Sixel text or its command takes (GFX-010).
    room: Option<super::widget::Room>,
}

/// A look's picture ready to show: its image id, the picture, its pixels
/// per cell, and what tells the look why a frame leaves its picture out.
pub(crate) struct LookPicture {
    pub image_id: u32,
    pub frame: Arc<GraphicsFrame>,
    pub pixels: (u16, u16),
    pub refused: Arc<dyn Fn(&str) + Send + Sync>,
}

/// The painter's looks: one drawing slot and one image per look identity,
/// kept across frames so a look is drawn again only when what drew it
/// changed (PIX-002), and dropped, with its picture, when its element is
/// gone from a frame.
#[derive(Default)]
pub(crate) struct Looks {
    worker: Option<Arc<GraphicsWorker>>,
    /// Why there is no drawing thread; tried once.
    failed: bool,
    slots: HashMap<u64, LookSlot>,
    frame: u64,
}

impl Looks {
    /// A frame begins: what this frame does not touch is dropped at its end.
    pub fn begin_frame(&mut self) {
        self.frame += 1;
    }

    /// A frame ends: looks no element of the frame carried free their
    /// pictures, so the terminal deletes their placements (PIX-002).
    pub fn end_frame(&mut self) {
        let frame = self.frame;
        self.slots.retain(|_, slot| slot.seen == frame);
    }

    fn worker(&mut self) -> Option<Arc<GraphicsWorker>> {
        if self.worker.is_none() && !self.failed {
            let options = crate::widgets::display::charts::graphics_options();
            match GraphicsWorker::shared(&options) {
                Ok(worker) => self.worker = Some(worker),
                Err(error) => {
                    log::warn!("pixel looks: no drawing thread: {error}");
                    self.failed = true;
                }
            }
        }
        self.worker.clone()
    }

    /// The finished picture of `look` for the element `identity` over
    /// `cells` columns and rows of `cell` pixels, shown as `output`, when the
    /// drawing thread has made one for exactly that; a scene is submitted
    /// when what drew the last picture differs. The App is woken when the
    /// picture is ready, so the next frame shows it. The picture has the
    /// terminal's pixels per cell, or, when that picture exceeds the
    /// renderer's limits or the room its output has (a Kitty picture sent
    /// in the command, `shared` false, takes 12 million pixels; one through
    /// shared memory or as Sixel the frame's 64 MiB), the largest whole
    /// pixels per cell that fit; a picture a frame refused for its output's
    /// room is drawn next with a quarter of its pixels (GFX-010).
    pub fn picture(
        &mut self,
        identity: u64,
        look: &Look,
        cells: (u32, u32),
        cell: (u16, u16),
        output: CanvasOutput,
        shared: bool,
    ) -> Option<LookPicture> {
        use super::widget::{pixels_per_cell, refused_for_room, Room, DIRECT_PIXELS, FRAME_PIXELS};
        let worker = self.worker()?;
        let frame = self.frame;
        let slot = self.slots.entry(identity).or_insert_with(|| LookSlot {
            picture: worker.picture(),
            image_id: crate::widgets::display::image::ProtocolRenderer::new().generate_image_id(),
            submitted: None,
            shown: None,
            seen: frame,
            refused: Arc::new(std::sync::Mutex::new(None)),
            room: None,
        });
        slot.seen = frame;
        let refusal = slot
            .refused
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(reason) = refusal {
            log::debug!("a pixel look's picture was left out of the frame: {reason}");
            if let Some(submitted) = slot.submitted.filter(|_| refused_for_room(&reason)) {
                slot.room = Some(Room::after(output, cells, submitted.size));
            }
        }
        let budget = if output == CanvasOutput::Kitty && !shared {
            DIRECT_PIXELS
        } else {
            FRAME_PIXELS
        };
        let budget = Room::within(slot.room, output, cells, budget);
        // Before the drawing thread has made its renderer, whose limits
        // decide the picture, the look asks for the terminal's pixels per
        // cell; a refused picture is asked for again once they are known.
        let pixels = match worker.limits() {
            Some(limits) => pixels_per_cell(cells.0, cells.1, cell, limits, budget)?,
            None => cell,
        };
        let size = (cells.0 * u32::from(pixels.0), cells.1 * u32::from(pixels.1));
        let wanted = Drawn {
            look: look.key(),
            size,
            cell,
            theme: crate::theme::Theme::generation(),
        };
        slot.picture.observe();
        if slot.submitted != Some(wanted) {
            // The scene is laid out in the terminal's pixels; a picture
            // with fewer pixels per cell is drawn smaller by that ratio.
            let base = if pixels == cell {
                Transform::identity()
            } else {
                Transform::scale(
                    f32::from(pixels.0) / f32::from(cell.0),
                    f32::from(pixels.1) / f32::from(cell.1),
                )
            };
            let full = (cells.0 * u32::from(cell.0), cells.1 * u32::from(cell.1));
            slot.picture.submit_job(Job {
                scene: Arc::new(look.scene(full, cell)),
                size,
                want: Want::Pixels,
                base,
            });
            slot.submitted = Some(wanted);
        }
        if let Some(finished) = slot.picture.latest() {
            if let Some(frame) = finished.frame.as_ref() {
                if finished.size == size && finished.theme == wanted.theme {
                    slot.shown = Some((wanted, frame.clone()));
                }
            }
        }
        // While the drawing thread makes the next picture of a look that
        // changed (the focus, the pointer, a value), the last one stays
        // placed: it fits the box, so the frame keeps its pixels and sends
        // nothing until the new picture is ready (PIX-002). A picture of
        // another size or cell size is never stretched, and one in an older
        // theme's colors is never shown: the next frame presented after a
        // change of theme is in the new colors, the look's flat cells until
        // its new picture is ready (THM-003, PIX-001).
        let shown = slot
            .shown
            .as_ref()
            .filter(|(drawn, _)| {
                drawn.size == wanted.size
                    && drawn.cell == wanted.cell
                    && drawn.theme == wanted.theme
            })
            .map(|(_, frame)| frame.clone())?;
        let told = slot.refused.clone();
        Some(LookPicture {
            image_id: slot.image_id,
            frame: shown,
            pixels,
            refused: Arc::new(move |reason: &str| {
                *told
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(reason.to_owned());
            }),
        })
    }
}

// ---------------------------------------------------------------------------
// What the controls read to choose their look, and the looks they draw.

thread_local! {
    /// What the terminal of the App rendering on this thread takes, set
    /// around a frame's render so a control can choose its pixel look
    /// (PIX-001).
    static HOST: std::cell::Cell<Option<HostReport>> = const { std::cell::Cell::new(None) };
}

/// The host report a backend's image output options describe.
pub(crate) fn host_report(options: &crate::backend::ImageOutputOptions) -> HostReport {
    HostReport {
        kitty: options.kitty_graphics,
        kitty_shared_memory: options.kitty_shared_memory,
        sixel: options.sixel,
        cell: options.cell_pixels,
    }
}

/// How a look's picture reaches `host`: as the canvas output is chosen,
/// from the host report, the process-wide graphics options and
/// `REACTIVE_TUI_CANVAS` (GFX-005, CHT-037); blocks means cells (PIX-001).
pub(crate) fn output_for(host: HostReport) -> CanvasOutput {
    CanvasOutput::choose(
        Some(host),
        crate::widgets::display::charts::graphics_options().output,
        CanvasOutput::from_environment(),
    )
}

/// The host of the frame being rendered, until the guard drops: the App
/// enters it before its root renders, with what its backend reports.
pub(crate) struct HostScope(Option<HostReport>);

impl Drop for HostScope {
    fn drop(&mut self) {
        HOST.with(|host| host.set(self.0));
    }
}

/// Enter the host `options` describe for the frame being rendered on this
/// thread; none when the backend reports no terminal.
pub(crate) fn enter_host(options: Option<&crate::backend::ImageOutputOptions>) -> HostScope {
    HostScope(HOST.with(|host| host.replace(options.map(host_report))))
}

/// Whether a control rendering now draws its pixel look: the frame's host
/// takes pixels and no switch turns the looks back to cells (PIX-001).
pub fn pixels_on() -> bool {
    HOST.with(|host| host.get())
        .is_some_and(|host| output_for(host) != CanvasOutput::Blocks)
}

/// A line one pixel wide in `border`, or in `ring` while the control holds
/// the focus.
fn frame_line(focused: bool) -> Line {
    Line::new(1.0, if focused { "ring" } else { "border" })
}

impl Look {
    /// A checkbox's box over its three cells: a square of the cell height
    /// less two pixels with a radius of two, bordered in `border` (`ring`
    /// with the focus), filled `primary` with a check mark in
    /// `primary-foreground` when checked and a dash when mixed; a disabled
    /// box at half (PIX-004).
    pub fn checkbox(checked: bool, mixed: bool, focused: bool, disabled: bool) -> Self {
        let center = (1.5, 0.5);
        let mut shapes = vec![Shape::Box {
            center,
            size: Extent::CELL_LESS_TWO,
            radius: Extent::Px(2.0),
            fill: (checked || mixed).then(|| "primary".to_owned()),
            border: Some(frame_line(focused)),
        }];
        if mixed {
            // The mark is sized with the box; its stroke is two pixels.
            shapes.push(Shape::Dash {
                center,
                size: Extent::CellLess {
                    less: 2.0,
                    times: 8.0 / 14.0,
                },
                width: Extent::Px(2.0),
                role: "primary-foreground".to_owned(),
            });
        } else if checked {
            shapes.push(Shape::Check {
                center,
                size: Extent::CellLess {
                    less: 2.0,
                    times: 10.0 / 14.0,
                },
                width: Extent::Px(2.0),
                role: "primary-foreground".to_owned(),
            });
        }
        Self {
            shapes,
            opacity: if disabled { 0.5 } else { 1.0 },
            ..Self::default()
        }
    }

    /// A radio button's circle over its three cells: a circle of the cell
    /// height less two pixels bordered in `border` (`ring` with the focus)
    /// and, when chosen, a dot of half that diameter in `primary`; a
    /// disabled circle at half (PIX-004).
    pub fn radio(chosen: bool, focused: bool, disabled: bool) -> Self {
        let center = (1.5, 0.5);
        let mut shapes = vec![Shape::Circle {
            center,
            diameter: Extent::CELL_LESS_TWO,
            fill: None,
            border: Some(frame_line(focused)),
        }];
        if chosen {
            shapes.push(Shape::Circle {
                center,
                diameter: Extent::CellLess {
                    less: 2.0,
                    times: 0.5,
                },
                fill: Some("primary".to_owned()),
                border: None,
            });
        }
        Self {
            shapes,
            opacity: if disabled { 0.5 } else { 1.0 },
            ..Self::default()
        }
    }

    /// A text input's field: `input` with a radius of a quarter of the cell
    /// height, bordered one pixel in `border`, two in `ring` with the focus
    /// and two in `error` while the value is invalid; a disabled field at
    /// half (PIX-004).
    pub fn field(focused: bool, invalid: bool, disabled: bool) -> Self {
        let (border, ring) = if invalid {
            (None, Some(Line::new(2.0, "error")))
        } else if focused {
            (None, Some(Line::new(2.0, "ring")))
        } else {
            (Some(Line::new(1.0, "border")), None)
        };
        Self {
            fill: Some("input".to_owned()),
            radius: Radius::Px(4.0),
            border,
            ring,
            opacity: if disabled { 0.5 } else { 1.0 },
            ..Self::default()
        }
    }

    /// A horizontal slider over `cells` cells: a track four pixels tall with
    /// rounded ends in `border` from the second cell to the last but one,
    /// its part up to `fraction` of the way in `primary`, and a thumb of
    /// the cell height less two pixels in `foreground` (`ring` with the
    /// focus) centered on that pixel; a disabled slider at half (PIX-005).
    pub fn slider(cells: usize, fraction: f64, focused: bool, disabled: bool) -> Self {
        let cells = cells.max(2) as f32;
        let (from, to) = (1.0, cells - 1.0);
        let at = from + (to - from) * fraction.clamp(0.0, 1.0) as f32;
        Self {
            shapes: vec![
                Shape::Bar {
                    from,
                    to,
                    row: 0.5,
                    height: Extent::Px(4.0),
                    radius: Extent::Px(2.0),
                    fill: "border".to_owned(),
                },
                Shape::Bar {
                    from,
                    to: at,
                    row: 0.5,
                    height: Extent::Px(4.0),
                    radius: Extent::Px(2.0),
                    fill: "primary".to_owned(),
                },
                Shape::Circle {
                    center: (at, 0.5),
                    diameter: Extent::CELL_LESS_TWO,
                    fill: Some(if focused { "ring" } else { "foreground" }.to_owned()),
                    border: None,
                },
            ],
            opacity: if disabled { 0.5 } else { 1.0 },
            ..Self::default()
        }
    }

    /// A horizontal progress bar over `cells` by `rows` cells: a track with
    /// a radius of half its height in `border` and its part up to
    /// `fraction` of the way in `primary` with the same radius (PIX-005).
    pub fn progress(cells: usize, rows: usize, fraction: f64) -> Self {
        let (cells, rows) = (cells as f32, rows.max(1) as f32);
        Self::bars(
            cells,
            rows,
            vec![(0.0, cells * fraction.clamp(0.0, 1.0) as f32)],
        )
    }

    /// An indeterminate progress bar over `cells` by `rows` cells: its track
    /// and a segment a quarter of it long whose start lies `along` of the
    /// way from the track's start to where the segment ends at its end
    /// (PIX-005).
    pub fn indeterminate_progress(cells: usize, rows: usize, along: f64) -> Self {
        let (cells, rows) = (cells as f32, rows.max(1) as f32);
        let span = (cells / 4.0).max(1.0);
        let start = (cells - span) * along.clamp(0.0, 1.0) as f32;
        Self::bars(cells, rows, vec![(start, start + span)])
    }

    fn bars(cells: f32, rows: f32, filled: Vec<(f32, f32)>) -> Self {
        let height = rows * REFERENCE_CELL_HEIGHT;
        let bar = |from: f32, to: f32, fill: &str| Shape::Bar {
            from,
            to,
            row: rows / 2.0,
            height: Extent::Scaled(height),
            radius: Extent::Scaled(height / 2.0),
            fill: fill.to_owned(),
        };
        let mut shapes = vec![bar(0.0, cells, "border")];
        shapes.extend(
            filled
                .into_iter()
                .map(|(from, to)| bar(from, to, "primary")),
        );
        Self {
            shapes,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_describe_a_look() {
        let look = Look::from_classes(
            "h-1 px-1 bg-primary text-primary-foreground rounded ring-2 ring-ring",
        )
        .unwrap();
        assert_eq!(look.fill.as_deref(), Some("primary"));
        assert_eq!(look.radius, Radius::Px(4.0));
        assert_eq!(look.ring, Some(Line::new(2.0, "ring")));
        assert!(look.border.is_none());
        let card = Look::from_classes("bg-surface border border-border rounded-lg p-2").unwrap();
        assert_eq!(card.border, Some(Line::new(1.0, "border")));
        assert_eq!(card.radius, Radius::Px(8.0));
        assert!(
            Look::from_classes("bg-primary p-2").is_none(),
            "no rounded class, no look"
        );
        assert!(
            Look::from_classes("rounded-lg p-2").is_none(),
            "no fill, no look"
        );
        let faded = Look::from_classes("bg-secondary rounded bg-opacity-50").unwrap();
        assert_eq!(faded.fill_opacity, 0.5);
    }

    /// GFX-010: a look whose picture a frame refused for the room its
    /// output has (its Sixel text, or its command) is drawn next with a
    /// quarter of the pixels, at whole pixels per cell; a refusal for the
    /// frame's budget across pictures changes nothing.
    #[test]
    fn a_look_refused_for_its_outputs_room_is_drawn_with_fewer_pixels_per_cell() {
        use std::time::{Duration, Instant};
        let worker = GraphicsWorker::spawn(crate::graphics::GraphicsOptions {
            force_cpu: true,
            font: crate::graphics::fonts::FontSource::Bundled,
            ..Default::default()
        })
        .expect("a worker");
        assert!(worker.wait_ready(Duration::from_secs(30)).is_some());
        let mut looks = Looks {
            worker: Some(Arc::new(worker)),
            ..Default::default()
        };
        let look = Look::from_classes("bg-primary rounded").unwrap();
        let (cells, cell) = ((40u32, 10u32), (8u16, 16u16));
        let shown = |looks: &mut Looks| {
            let deadline = Instant::now() + Duration::from_secs(30);
            loop {
                looks.begin_frame();
                if let Some(picture) =
                    looks.picture(1, &look, cells, cell, CanvasOutput::Sixel, false)
                {
                    return picture;
                }
                assert!(Instant::now() < deadline, "the look got no picture");
                std::thread::sleep(Duration::from_millis(1));
            }
        };
        let first = shown(&mut looks);
        assert_eq!(
            first.pixels, cell,
            "the first picture has the terminal's pixels per cell"
        );
        (first.refused)("image frame exceeds 64 MiB raster limit");
        let same = shown(&mut looks);
        assert_eq!(
            same.pixels, cell,
            "a refusal for the frame's budget across pictures draws no smaller picture"
        );
        (same.refused)("image frame exceeds 64 MiB output limit");
        let smaller = shown(&mut looks);
        let (across, down) = (u32::from(smaller.pixels.0), u32::from(smaller.pixels.1));
        assert!(
            across * down * cells.0 * cells.1 <= 40 * 8 * 10 * 16 / 4 && across >= 1 && down >= 1,
            "after a refusal for its output's room the picture has {:?} pixels per cell, more than a quarter of {cell:?}",
            smaller.pixels
        );
        assert_eq!(
            smaller.frame.image().dimensions(),
            (cells.0 * across, cells.1 * down),
            "the smaller picture has whole pixels per cell"
        );
    }
}
