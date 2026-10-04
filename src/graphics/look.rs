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
use super::{Color, GraphicsFrame, GraphicsWorker, Paint, Path, Scene, Stroke, Transform};
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
/// pixels at a cell height of 16, in a theme role.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Pixels at a cell height of 16.
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

/// A shape a control draws inside its look. Positions are in cells of the
/// look's rectangle (fractions allowed), sizes in pixels at a cell height
/// of 16, scaled in proportion otherwise.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// A rounded box centered at `center`, `size` pixels square.
    Box {
        /// The box's center, in cells.
        center: (f32, f32),
        /// Its side, in pixels at a cell height of 16.
        size: f32,
        /// Its corner radius, in the same pixels.
        radius: f32,
        /// The role it is filled with, if any.
        fill: Option<String>,
        /// Its border, if any.
        border: Option<Line>,
    },
    /// A circle centered at `center` of `diameter` pixels.
    Circle {
        /// The circle's center, in cells.
        center: (f32, f32),
        /// Its diameter, in pixels at a cell height of 16.
        diameter: f32,
        /// The role it is filled with, if any.
        fill: Option<String>,
        /// Its border, if any.
        border: Option<Line>,
    },
    /// A bar from `from` to `to` cells across, `height` pixels tall, centered
    /// on row `row`, with rounded ends of `radius` pixels.
    Bar {
        /// Where the bar starts, in cells across.
        from: f32,
        /// Where it ends, in cells across.
        to: f32,
        /// The row its middle lies on, in cells down.
        row: f32,
        /// Its height, in pixels at a cell height of 16.
        height: f32,
        /// The radius of its ends, in the same pixels.
        radius: f32,
        /// The role it is filled with.
        fill: String,
    },
    /// A check mark centered at `center`, `size` pixels wide, `width`
    /// pixels thick.
    Check {
        /// The mark's center, in cells.
        center: (f32, f32),
        /// Its width, in pixels at a cell height of 16.
        size: f32,
        /// Its stroke, in the same pixels.
        width: f32,
        /// The role it is drawn in.
        role: String,
    },
    /// A dash centered at `center`, `size` pixels wide, `width` thick.
    Dash {
        /// The dash's center, in cells.
        center: (f32, f32),
        /// Its width, in pixels at a cell height of 16.
        size: f32,
        /// Its stroke, in the same pixels.
        width: f32,
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
        let scale = Self::scale(cell);
        let radius = self.radius_px(width, height, cell);
        let mut scene = Scene::new();
        if let Some(fill) = &self.fill {
            scene.fill(
                &Path::rounded_rect(0.0, 0.0, width, height, radius),
                &Paint::solid(Color::token(fill.as_str())).opacity(self.fill_opacity),
            );
        }
        let mut inset = 0.0;
        for line in [&self.ring, &self.border].into_iter().flatten() {
            let line_width = (line.width * scale).max(1.0).round();
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
                &Paint::solid(Color::token(line.role.as_str())),
            );
            inset += line_width;
        }
        for shape in &self.shapes {
            shape.draw(&mut scene, cell, scale);
        }
        scene
    }

    /// The cells of a box of `columns` by `rows` whose outer corner pixel
    /// the rounded corners leave transparent: the painter leaves their cell
    /// background unpainted, so what is under the element shows there
    /// (PIX-003), and gives text in them the fill's color.
    pub(crate) fn corner_cells(
        &self,
        columns: u32,
        rows: u32,
        cell: (u16, u16),
    ) -> Vec<(u32, u32)> {
        let (cw, ch) = (f32::from(cell.0.max(1)), f32::from(cell.1.max(1)));
        let (width, height) = (columns as f32 * cw, rows as f32 * ch);
        let radius = self.radius_px(width, height, cell);
        if radius <= 0.0 || columns == 0 || rows == 0 {
            return Vec::new();
        }
        let mut cells = Vec::new();
        // Whether the pixel at (dx, dy) from a corner, inside the box, is
        // outside the arc of that corner.
        let outside = |dx: f32, dy: f32| {
            dx < radius
                && dy < radius
                && (radius - dx).powi(2) + (radius - dy).powi(2) > radius * radius
        };
        let span_x = (radius / cw).ceil() as u32;
        let span_y = (radius / ch).ceil() as u32;
        for row in 0..span_y.min(rows) {
            for column in 0..span_x.min(columns) {
                if !outside(column as f32 * cw, row as f32 * ch) {
                    continue;
                }
                for (x, y) in [
                    (column, row),
                    (columns - 1 - column, row),
                    (column, rows - 1 - row),
                    (columns - 1 - column, rows - 1 - row),
                ] {
                    if !cells.contains(&(x, y)) {
                        cells.push((x, y));
                    }
                }
            }
        }
        cells
    }
}

impl Shape {
    fn draw(&self, scene: &mut Scene, cell: (u16, u16), scale: f32) {
        let (cw, ch) = (f32::from(cell.0.max(1)), f32::from(cell.1.max(1)));
        let at = |(x, y): (f32, f32)| (x * cw, y * ch);
        let px = |size: f32| (size * scale).max(1.0);
        match self {
            Shape::Box {
                center,
                size,
                radius,
                fill,
                border,
            } => {
                let (cx, cy) = at(*center);
                let side = px(*size).round();
                let path = Path::rounded_rect(
                    cx - side / 2.0,
                    cy - side / 2.0,
                    side,
                    side,
                    px(*radius).min(side / 2.0),
                );
                if let Some(fill) = fill {
                    scene.fill(&path, &Paint::solid(Color::token(fill.as_str())));
                }
                if let Some(line) = border {
                    let width = px(line.width).round();
                    let inner = Path::rounded_rect(
                        cx - side / 2.0 + width / 2.0,
                        cy - side / 2.0 + width / 2.0,
                        side - width,
                        side - width,
                        (px(*radius) - width / 2.0).max(0.0),
                    );
                    scene.stroke(
                        &inner,
                        &Stroke::new(width),
                        &Paint::solid(Color::token(line.role.as_str())),
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
                let r = px(*diameter) / 2.0;
                if let Some(fill) = fill {
                    scene.fill(
                        &Path::ellipse(cx, cy, r, r),
                        &Paint::solid(Color::token(fill.as_str())),
                    );
                }
                if let Some(line) = border {
                    let width = px(line.width).round();
                    scene.stroke(
                        &Path::ellipse(cx, cy, r - width / 2.0, r - width / 2.0),
                        &Stroke::new(width),
                        &Paint::solid(Color::token(line.role.as_str())),
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
                let h = px(*height).round();
                let y = row * ch - h / 2.0;
                if x1 > x0 {
                    scene.fill(
                        &Path::rounded_rect(x0, y, x1 - x0, h, px(*radius).min(h / 2.0)),
                        &Paint::solid(Color::token(fill.as_str())),
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
                let s = px(*size);
                let path = super::PathBuilder::new()
                    .move_to(cx - s * 0.45, cy)
                    .line_to(cx - s * 0.1, cy + s * 0.35)
                    .line_to(cx + s * 0.5, cy - s * 0.35)
                    .build();
                scene.stroke(
                    &path,
                    &Stroke::new(px(*width).round())
                        .cap(super::LineCap::Round)
                        .join(super::LineJoin::Round),
                    &Paint::solid(Color::token(role.as_str())),
                );
            }
            Shape::Dash {
                center,
                size,
                width,
                role,
            } => {
                let (cx, cy) = at(*center);
                let s = px(*size);
                let path = super::PathBuilder::new()
                    .move_to(cx - s * 0.4, cy)
                    .line_to(cx + s * 0.4, cy)
                    .build();
                scene.stroke(
                    &path,
                    &Stroke::new(px(*width).round()).cap(super::LineCap::Round),
                    &Paint::solid(Color::token(role.as_str())),
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

    /// The image id and the finished picture of `look` for the element
    /// `identity` at `size` pixels over cells of `cell` pixels, when the
    /// drawing thread has made one for exactly that; a scene is submitted
    /// when what drew the last picture differs. The App is woken when the
    /// picture is ready, so the next frame shows it.
    pub fn picture(
        &mut self,
        identity: u64,
        look: &Look,
        size: (u32, u32),
        cell: (u16, u16),
    ) -> Option<(u32, Option<Arc<GraphicsFrame>>)> {
        let worker = self.worker()?;
        let frame = self.frame;
        let slot = self.slots.entry(identity).or_insert_with(|| LookSlot {
            picture: worker.picture(),
            image_id: crate::widgets::display::image::ProtocolRenderer::new().generate_image_id(),
            submitted: None,
            shown: None,
            seen: frame,
        });
        slot.seen = frame;
        let wanted = Drawn {
            look: look.key(),
            size,
            cell,
            theme: crate::theme::Theme::generation(),
        };
        slot.picture.observe();
        if slot.submitted != Some(wanted) {
            slot.picture.submit_job(Job {
                scene: Arc::new(look.scene(size, cell)),
                size,
                want: Want::Pixels,
                base: Transform::identity(),
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
        let shown = slot
            .shown
            .as_ref()
            .filter(|(drawn, _)| *drawn == wanted)
            .map(|(_, frame)| frame.clone());
        Some((slot.image_id, shown))
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

    #[test]
    fn the_corner_cells_are_the_ones_the_arcs_leave_transparent() {
        let button = Look::from_classes("bg-primary rounded").unwrap();
        let mut cells = button.corner_cells(8, 1, (8, 16));
        cells.sort();
        assert_eq!(cells, vec![(0, 0), (7, 0)]);
        let box_2xl = Look::from_classes("bg-primary rounded-2xl").unwrap();
        let mut cells = box_2xl.corner_cells(20, 4, (8, 16));
        cells.sort();
        assert_eq!(
            cells,
            vec![
                (0, 0),
                (0, 3),
                (1, 0),
                (1, 3),
                (18, 0),
                (18, 3),
                (19, 0),
                (19, 3)
            ]
        );
        let square = Look {
            fill: Some("primary".into()),
            ..Default::default()
        };
        assert!(square.corner_cells(8, 1, (8, 16)).is_empty());
    }
}
