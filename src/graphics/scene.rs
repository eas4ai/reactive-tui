//! What an application asks the canvas to draw (GFX-001): paths filled or
//! stroked with solid or gradient paint, images, text and cell grids, under
//! nested transforms and clips. A [`Scene`] is a list of drawing commands in
//! painting order; both renderers read the same list.

use super::GraphicsError;
use crate::layout::CellGrid;
use std::sync::Arc;

/// A color: exact red, green, blue and alpha, or a token the layout's
/// color resolver reads (a theme variable such as `primary`, a palette name
/// such as `blue-500`, or hex), resolved against the active theme when the
/// scene is drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    /// Exact sRGB channels and alpha.
    Rgba([u8; 4]),
    /// A token for [`crate::theme::Theme::resolve_color`].
    Token(String),
}

impl Color {
    /// An exact color.
    pub fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color::Rgba([red, green, blue, alpha])
    }

    /// A color the active theme resolves.
    pub fn token(token: impl Into<String>) -> Self {
        Color::Token(token.into())
    }

    /// Straight (not premultiplied) sRGB channels and alpha in `0.0..=1.0`;
    /// a token no resolver knows is transparent.
    pub(crate) fn resolve(&self) -> [f32; 4] {
        match self {
            Color::Rgba(channels) => channels.map(|c| f32::from(c) / 255.0),
            Color::Token(token) => crate::theme::Theme::active()
                .resolve_color(token)
                .map_or([0.0; 4], |(r, g, b, a)| [r, g, b, a]),
        }
    }
}

/// A color at an offset along a gradient, from 0.0 at its start to 1.0 at
/// its end.
#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    /// Where the color sits, clamped to `0.0..=1.0`.
    pub offset: f32,
    /// The color there.
    pub color: Color,
}

impl GradientStop {
    /// A stop of `color` at `offset`.
    pub fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

/// How a paint colors each pixel.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PaintKind {
    Solid(Color),
    Linear {
        start: (f32, f32),
        end: (f32, f32),
        stops: Vec<GradientStop>,
    },
    Radial {
        center: (f32, f32),
        radius: f32,
        stops: Vec<GradientStop>,
    },
}

/// What fills a shape or a stroke: a solid color or a linear or radial
/// gradient in scene coordinates, with an opacity.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    pub(crate) kind: PaintKind,
    pub(crate) opacity: f32,
}

impl Paint {
    /// One color.
    pub fn solid(color: Color) -> Self {
        Self {
            kind: PaintKind::Solid(color),
            opacity: 1.0,
        }
    }

    /// A gradient along the line from `start` to `end`, flat beyond them.
    pub fn linear(start: (f32, f32), end: (f32, f32), stops: Vec<GradientStop>) -> Self {
        Self {
            kind: PaintKind::Linear { start, end, stops },
            opacity: 1.0,
        }
    }

    /// A gradient from `center` out to `radius`, flat beyond it.
    pub fn radial(center: (f32, f32), radius: f32, stops: Vec<GradientStop>) -> Self {
        Self {
            kind: PaintKind::Radial {
                center,
                radius,
                stops,
            },
            opacity: 1.0,
        }
    }

    /// The paint at `opacity`, clamped to `0.0..=1.0`.
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }
}

/// How a stroke's segments meet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineJoin {
    /// Extended to a point, or beveled past the miter limit.
    #[default]
    Miter,
    /// Rounded.
    Round,
    /// Cut straight across.
    Bevel,
}

/// How an open stroke ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LineCap {
    /// Flat at the end point.
    #[default]
    Butt,
    /// A half circle past the end point.
    Round,
    /// Half the width past the end point, square.
    Square,
}

/// How a path's outline is drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub(crate) width: f32,
    pub(crate) join: LineJoin,
    pub(crate) cap: LineCap,
    pub(crate) miter_limit: f32,
    pub(crate) dash: Vec<f32>,
}

impl Stroke {
    /// A stroke `width` wide with miter joins and butt caps.
    pub fn new(width: f32) -> Self {
        Self {
            width: width.max(0.0),
            join: LineJoin::default(),
            cap: LineCap::default(),
            miter_limit: 4.0,
            dash: Vec::new(),
        }
    }

    /// The stroke with `join`.
    pub fn join(mut self, join: LineJoin) -> Self {
        self.join = join;
        self
    }

    /// The stroke with `cap`.
    pub fn cap(mut self, cap: LineCap) -> Self {
        self.cap = cap;
        self
    }

    /// The stroke dashed: lengths of dash and gap in turn, repeated; an
    /// empty or all-zero pattern draws a solid line.
    pub fn dash(mut self, pattern: &[f32]) -> Self {
        self.dash = pattern.iter().map(|length| length.max(0.0)).collect();
        self
    }
}

/// An affine transform: a point (x, y) goes to (a x + c y + e, b x + d y + f).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub(crate) a: f32,
    pub(crate) b: f32,
    pub(crate) c: f32,
    pub(crate) d: f32,
    pub(crate) e: f32,
    pub(crate) f: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    /// No change.
    pub fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    /// A move by (`x`, `y`).
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            e: x,
            f: y,
            ..Self::identity()
        }
    }

    /// A rotation by `degrees`, clockwise on screen (y grows downwards).
    pub fn rotate(degrees: f32) -> Self {
        let (sin, cos) = degrees.to_radians().sin_cos();
        Self {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// A scale by `x` across and `y` down.
    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            a: x,
            d: y,
            ..Self::identity()
        }
    }

    /// This transform, then `next`.
    pub fn then(self, next: Transform) -> Self {
        Self {
            a: next.a * self.a + next.c * self.b,
            b: next.b * self.a + next.d * self.b,
            c: next.a * self.c + next.c * self.d,
            d: next.b * self.c + next.d * self.d,
            e: next.a * self.e + next.c * self.f + next.e,
            f: next.b * self.e + next.d * self.f + next.f,
        }
    }

    /// Where the point (`x`, `y`) goes.
    pub fn apply(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    /// The transform that undoes this one, or `None` when it collapses the
    /// plane.
    pub fn inverse(&self) -> Option<Transform> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-12 {
            return None;
        }
        let a = self.d / det;
        let b = -self.b / det;
        let c = -self.c / det;
        let d = self.a / det;
        Some(Self {
            a,
            b,
            c,
            d,
            e: -(a * self.e + c * self.f),
            f: -(b * self.e + d * self.f),
        })
    }

    /// The transform that fits a picture of `view` into one of `into`: as
    /// large as fits with its shape kept, and centred.
    pub fn fit(view: (f32, f32), into: (f32, f32)) -> Self {
        if !(view.0 > 0.0 && view.1 > 0.0) {
            return Self::identity();
        }
        let scale = (into.0 / view.0).min(into.1 / view.1);
        Self::scale(scale, scale).then(Self::translate(
            (into.0 - view.0 * scale) / 2.0,
            (into.1 - view.1 * scale) / 2.0,
        ))
    }

    /// Whether the transform only moves points.
    pub(crate) fn is_translation(&self) -> bool {
        self.a == 1.0 && self.b == 0.0 && self.c == 0.0 && self.d == 1.0
    }

    /// The largest factor by which it stretches a length.
    pub(crate) fn max_scale(&self) -> f32 {
        let x = (self.a * self.a + self.b * self.b).sqrt();
        let y = (self.c * self.c + self.d * self.d).sqrt();
        x.max(y)
    }
}

/// One step of a path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Segment {
    MoveTo((f32, f32)),
    LineTo((f32, f32)),
    QuadTo((f32, f32), (f32, f32)),
    CubicTo((f32, f32), (f32, f32), (f32, f32)),
    /// An elliptical arc to a point, as SVG's `A` command describes it.
    ArcTo {
        radii: (f32, f32),
        rotation: f32,
        large: bool,
        sweep: bool,
        to: (f32, f32),
    },
    Close,
}

/// A shape: subpaths of lines, curves and arcs, filled by the non-zero
/// winding rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Path {
    pub(crate) segments: Vec<Segment>,
}

impl Path {
    /// A rectangle with its top left corner at (`x`, `y`).
    pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Self {
        PathBuilder::new()
            .move_to(x, y)
            .line_to(x + width, y)
            .line_to(x + width, y + height)
            .line_to(x, y + height)
            .close()
            .build()
    }

    /// A rectangle whose corners are quarter circles of `radius`, at most
    /// half its shorter side.
    pub fn rounded_rect(x: f32, y: f32, width: f32, height: f32, radius: f32) -> Self {
        let r = radius.clamp(0.0, width.min(height) / 2.0);
        if r == 0.0 {
            return Self::rect(x, y, width, height);
        }
        PathBuilder::new()
            .move_to(x + r, y)
            .line_to(x + width - r, y)
            .arc_to(r, r, 0.0, false, true, x + width, y + r)
            .line_to(x + width, y + height - r)
            .arc_to(r, r, 0.0, false, true, x + width - r, y + height)
            .line_to(x + r, y + height)
            .arc_to(r, r, 0.0, false, true, x, y + height - r)
            .line_to(x, y + r)
            .arc_to(r, r, 0.0, false, true, x + r, y)
            .close()
            .build()
    }

    /// An ellipse centred on (`cx`, `cy`) with radii `rx` and `ry`.
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        PathBuilder::new()
            .move_to(cx + rx, cy)
            .arc_to(rx, ry, 0.0, false, true, cx - rx, cy)
            .arc_to(rx, ry, 0.0, false, true, cx + rx, cy)
            .close()
            .build()
    }
}

/// Builds a [`Path`] one step at a time.
#[derive(Clone, Debug, Default)]
pub struct PathBuilder {
    path: Path,
}

impl PathBuilder {
    /// An empty path.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new subpath at (`x`, `y`).
    pub fn move_to(mut self, x: f32, y: f32) -> Self {
        self.path.segments.push(Segment::MoveTo((x, y)));
        self
    }

    /// A straight line to (`x`, `y`).
    pub fn line_to(mut self, x: f32, y: f32) -> Self {
        self.path.segments.push(Segment::LineTo((x, y)));
        self
    }

    /// A quadratic curve bent towards (`cx`, `cy`), ending at (`x`, `y`).
    pub fn quad_to(mut self, cx: f32, cy: f32, x: f32, y: f32) -> Self {
        self.path.segments.push(Segment::QuadTo((cx, cy), (x, y)));
        self
    }

    /// A cubic curve with control points (`c1x`, `c1y`) and (`c2x`, `c2y`),
    /// ending at (`x`, `y`).
    pub fn cubic_to(mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) -> Self {
        self.path
            .segments
            .push(Segment::CubicTo((c1x, c1y), (c2x, c2y), (x, y)));
        self
    }

    /// An elliptical arc with radii `rx` and `ry`, its axes turned by
    /// `rotation` degrees, to (`x`, `y`): of the four arcs that fit, the
    /// larger one when `large`, the clockwise one when `sweep` (SVG's `A`).
    #[allow(clippy::too_many_arguments)]
    pub fn arc_to(
        mut self,
        rx: f32,
        ry: f32,
        rotation: f32,
        large: bool,
        sweep: bool,
        x: f32,
        y: f32,
    ) -> Self {
        self.path.segments.push(Segment::ArcTo {
            radii: (rx, ry),
            rotation,
            large,
            sweep,
            to: (x, y),
        });
        self
    }

    /// Close the subpath with a line back to its start.
    pub fn close(mut self) -> Self {
        self.path.segments.push(Segment::Close);
        self
    }

    /// The path.
    pub fn build(self) -> Path {
        self.path
    }
}

/// An RGBA image a scene draws, scaled smoothly into its rectangle.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasImage {
    pub(crate) width: u32,
    pub(crate) height: u32,
    /// Premultiplied by alpha, the form both renderers blend.
    pub(crate) pixels: Vec<[u8; 4]>,
}

impl CanvasImage {
    /// The largest width or height of an image.
    pub const MAX_SIDE: u32 = 4096;

    /// An image of `width` by `height` pixels, row by row, with straight
    /// (not premultiplied) alpha; each side at most [`Self::MAX_SIDE`].
    pub fn from_rgba(
        width: u32,
        height: u32,
        mut pixels: Vec<[u8; 4]>,
    ) -> Result<Self, GraphicsError> {
        let expected = u64::from(width) * u64::from(height);
        if width == 0
            || height == 0
            || width > Self::MAX_SIDE
            || height > Self::MAX_SIDE
            || expected != pixels.len() as u64
        {
            return Err(GraphicsError::Dimensions(format!(
                "{width}x{height} image with {} pixels",
                pixels.len()
            )));
        }
        for pixel in pixels.iter_mut().filter(|pixel| pixel[3] != 255) {
            let alpha = u32::from(pixel[3]);
            for channel in &mut pixel[..3] {
                *channel = ((u32::from(*channel) * alpha + 127) / 255) as u8;
            }
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// One drawing command, with the transform and clips in force when it was
/// recorded kept by the renderer, not here.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Command {
    Fill(Path, Paint),
    Stroke(Path, Stroke, Paint),
    Image((f32, f32, f32, f32), Arc<CanvasImage>),
    Text((f32, f32), f32, String, Paint),
    Cells((f32, f32), Arc<CellGrid>, (u16, u16)),
    PushTransform(Transform),
    PopTransform,
    PushClip(Path),
    PopClip,
}

/// A drawing in painting order. Transforms and clips pushed apply to what is
/// drawn until they are popped; a pop without a push does nothing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub(crate) commands: Vec<Command>,
}

impl Scene {
    /// An empty scene: a transparent picture.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same drawing under `transform`.
    pub fn transformed(&self, transform: Transform) -> Self {
        let mut commands = Vec::with_capacity(self.commands.len() + 1);
        commands.push(Command::PushTransform(transform));
        commands.extend(self.commands.iter().cloned());
        Self { commands }
    }

    /// Fill `path` with `paint`.
    pub fn fill(&mut self, path: &Path, paint: &Paint) -> &mut Self {
        self.commands
            .push(Command::Fill(path.clone(), paint.clone()));
        self
    }

    /// Draw the outline of `path` with `stroke` and `paint`.
    pub fn stroke(&mut self, path: &Path, stroke: &Stroke, paint: &Paint) -> &mut Self {
        self.commands
            .push(Command::Stroke(path.clone(), stroke.clone(), paint.clone()));
        self
    }

    /// Draw `image` scaled into the rectangle (x, y, width, height).
    pub fn image(&mut self, rect: (f32, f32, f32, f32), image: Arc<CanvasImage>) -> &mut Self {
        self.commands.push(Command::Image(rect, image));
        self
    }

    /// Draw `text` on one line with its baseline starting at `origin`, glyphs
    /// `size` pixels tall, in the canvas's font.
    pub fn text(&mut self, origin: (f32, f32), size: f32, text: &str, paint: &Paint) -> &mut Self {
        self.commands
            .push(Command::Text(origin, size, text.to_owned(), paint.clone()));
        self
    }

    /// Draw `grid` with its top left cell at `origin`, each cell `cell`
    /// pixels, through the glyph atlas in one instanced draw on the GPU.
    /// The grid moves with the transform in force and stretches with it
    /// along the axes, its cells rounded to whole pixels; it does not turn.
    pub fn cells(
        &mut self,
        origin: (f32, f32),
        grid: Arc<CellGrid>,
        cell: (u16, u16),
    ) -> &mut Self {
        self.commands.push(Command::Cells(origin, grid, cell));
        self
    }

    /// Apply `transform` to what follows, inside any transform in force:
    /// a point goes through `transform` first and then through those
    /// pushed before it.
    pub fn push_transform(&mut self, transform: Transform) -> &mut Self {
        self.commands.push(Command::PushTransform(transform));
        self
    }

    /// End the last transform pushed.
    pub fn pop_transform(&mut self) -> &mut Self {
        self.commands.push(Command::PopTransform);
        self
    }

    /// Clip what follows to the inside of `path`, within any clip in force.
    pub fn push_clip(&mut self, path: &Path) -> &mut Self {
        self.commands.push(Command::PushClip(path.clone()));
        self
    }

    /// End the last clip pushed.
    pub fn pop_clip(&mut self) -> &mut Self {
        self.commands.push(Command::PopClip);
        self
    }
}
