//! The `Canvas` widget (GFX-001): placed in an Element tree like a chart,
//! it fills the rectangle its parent allots and shows the picture of its
//! scene there, one picture pixel per screen pixel unless the application
//! pins fewer pixels per cell or a hard limit forces fewer (GFX-010). The
//! drawing thread draws; the App's thread submits the scene and shows the
//! newest finished picture (GFX-003).

use super::hybrid::{GraphicsOptions, CELL_PIXELS};
use super::output::{CanvasLink, CanvasOutput, CanvasPaint, CanvasPixels};
use super::scene::{Scene, Transform};
use super::worker::{Finished, GraphicsWorker, Job, Picture, Want};
use super::PictureLimits;
use crate::builder::ElementBuilder;
use crate::component::{
    Component, Element, ElementType, LayoutInfo, LayoutType, LifecycleEvent, Props,
};
use crate::layout::style::StyleBuilder;
use std::sync::{Arc, Mutex};

/// What a [`Canvas`] draws and how.
#[derive(Clone, Debug)]
pub struct CanvasProps {
    scene: Arc<Scene>,
    options: GraphicsOptions,
    view: Option<(f32, f32)>,
    label: Option<String>,
    worker: Option<Arc<GraphicsWorker>>,
    cell_pixels: Option<(u16, u16)>,
}

impl CanvasProps {
    /// A canvas that draws `scene`, in pixels from its top left corner. A
    /// cell of the terminal is as many pixels as the host reports, 8 by 16
    /// where it reports none.
    pub fn new(scene: Arc<Scene>) -> Self {
        Self {
            scene,
            options: GraphicsOptions::default(),
            view: None,
            label: None,
            worker: None,
            cell_pixels: None,
        }
    }

    /// Draw on `worker` instead of the drawing thread the process shares
    /// for the canvas's options. An application starts one before it sets
    /// the terminal up ([`GraphicsWorker::wait_ready`]); the worker's own
    /// options decide its renderer and font, and it serves every canvas it
    /// is handed (GFX-003).
    pub fn worker(mut self, worker: Arc<GraphicsWorker>) -> Self {
        self.worker = Some(worker);
        self
    }

    /// Draw the picture with `width` by `height` pixels per cell instead
    /// of the terminal's, fewer in each direction; the terminal scales the
    /// picture to the cells it covers (GFX-010). The scene keeps its
    /// coordinates: a cell is still as many scene pixels as the terminal's
    /// cell measures. A pin of zero in either direction is ignored.
    pub fn cell_pixels(mut self, width: u16, height: u16) -> Self {
        self.cell_pixels = (width > 0 && height > 0).then_some((width, height));
        self
    }

    /// How the canvas renders and shows its pictures.
    pub fn options(mut self, options: GraphicsOptions) -> Self {
        self.options = options;
        self
    }

    /// The scene is drawn for a picture of `width` by `height`: the canvas
    /// scales it to fit its area, keeping its shape, and centres it.
    pub fn view(mut self, width: f32, height: f32) -> Self {
        self.view = (width > 0.0 && height > 0.0).then_some((width, height));
        self
    }

    /// What the picture shows, for a screen reader; "Canvas" when not set.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl PartialEq for CanvasProps {
    fn eq(&self, other: &Self) -> bool {
        (Arc::ptr_eq(&self.scene, &other.scene) || self.scene == other.scene)
            && self.options == other.options
            && self.view == other.view
            && self.label == other.label
            && self.cell_pixels == other.cell_pixels
            && match (&self.worker, &other.worker) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}

impl Props for CanvasProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// What the last job was submitted for, to submit a scene once.
#[derive(Clone)]
struct Submitted {
    scene: Arc<Scene>,
    size: (u32, u32),
    want: Want,
    base: Transform,
    /// The theme in force when the job was submitted: a scene's tokens
    /// take their colors when the worker draws (GFX-001).
    theme: u64,
}

impl Submitted {
    fn is(&self, job: &Job, theme: u64) -> bool {
        self.theme == theme
            && self.size == job.size
            && self.want == job.want
            && self.base == job.base
            && (Arc::ptr_eq(&self.scene, &job.scene) || self.scene == job.scene)
    }
}

/// The most pixels of a picture that is sent in the command itself. Such a
/// picture is written as base64, four bytes for three, and a frame's output
/// holds 64 MiB.
const DIRECT_PIXELS: u64 = 12_000_000;

/// The most pixels of one picture a frame holds: 64 MiB of them (GFX-007).
const FRAME_PIXELS: u64 = 64 * 1024 * 1024 / 4;

/// What a canvas asks of its drawing thread, for which output, the columns
/// and rows its picture covers, and the picture's pixels per cell.
type Wanted = (Job, CanvasOutput, (u32, u32), (u16, u16));

/// The most whole pixels per cell, at most `wanted` in each direction, for
/// which `columns` by `rows` cells make a picture within `limits` and
/// `budget` pixels: the most pixels first, the cell's own proportions
/// second (GFX-010). `None` when not even one pixel per cell fits.
fn pixels_per_cell(
    columns: u32,
    rows: u32,
    wanted: (u16, u16),
    limits: PictureLimits,
    budget: u64,
) -> Option<(u16, u16)> {
    let wanted = (u32::from(wanted.0.max(1)), u32::from(wanted.1.max(1)));
    let pixels = limits.pixels.min(budget);
    let proportion = wanted.0 as f64 / wanted.1 as f64;
    let mut best: Option<((u32, u32), u64, f64)> = None;
    for across in (1..=wanted.0).rev() {
        let width = u64::from(columns) * u64::from(across);
        if width > u64::from(limits.side) {
            continue;
        }
        let down = (pixels / width / u64::from(rows))
            .min(u64::from(limits.side / rows.max(1)))
            .min(u64::from(wanted.1)) as u32;
        if down == 0 {
            continue;
        }
        let area = u64::from(across) * u64::from(down);
        let skew = ((across as f64 / down as f64) / proportion).ln().abs();
        if best.is_none_or(|(_, most, least)| area > most || (area == most && skew < least)) {
            best = Some(((across, down), area, skew));
        }
    }
    best.map(|((across, down), _, _)| (across as u16, down as u16))
}

#[derive(Default)]
struct View {
    submitted: Option<Submitted>,
    /// The newest finished picture for the size and output in force.
    shown: Option<Finished>,
    /// Whether a render has passed with no report from a painter, after
    /// which the canvas stops waiting for one.
    waited: bool,
}

/// A widget that draws a scene.
pub struct Canvas {
    props: CanvasProps,
    worker: Option<Arc<GraphicsWorker>>,
    /// The canvas's picture on the drawing thread.
    picture: Option<Picture>,
    /// Why there is no drawing thread.
    failure: Option<String>,
    link: Arc<CanvasLink>,
    image_id: u32,
    viewport: Option<LayoutInfo>,
    view: Mutex<View>,
}

impl Canvas {
    /// The worker `props` names, else the drawing thread the process
    /// shares for their options (GFX-003), with a picture of the canvas's
    /// own on it; or why there is none.
    fn start(
        props: &CanvasProps,
    ) -> (Option<Arc<GraphicsWorker>>, Option<Picture>, Option<String>) {
        let worker = match &props.worker {
            Some(worker) => Ok(worker.clone()),
            None => GraphicsWorker::shared(&props.options),
        };
        match worker {
            Ok(worker) => {
                let picture = worker.picture();
                (Some(worker), Some(picture), None)
            }
            Err(error) => (None, None, Some(error.to_string())),
        }
    }

    /// Whether `a` and `b` draw on the same worker.
    fn same_worker(a: &CanvasProps, b: &CanvasProps) -> bool {
        match (&a.worker, &b.worker) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            // A worker the canvas started serves while the options stay.
            (None, None) => a.options == b.options,
            _ => false,
        }
    }

    /// The canvas's area in cells.
    fn cells(&self) -> (u32, u32) {
        self.viewport.map_or((0, 0), |viewport| {
            let (width, height) = viewport.content_size();
            (width.max(0.0) as u32, height.max(0.0) as u32)
        })
    }

    /// The job for the canvas's area as it is, the output it is for, the
    /// cells its picture covers and the picture's pixels per cell: the
    /// terminal's, unless the application pinned fewer or a hard limit
    /// forces fewer (GFX-010). `None` before the first layout, which gives
    /// the area its size, and before the drawing thread has made its
    /// renderer, whose limits decide the picture.
    fn job(&self, waited: bool) -> Option<Wanted> {
        let (columns, rows) = self.cells();
        if columns == 0 || rows == 0 {
            return None;
        }
        let limits = self.worker.as_ref()?.limits()?;
        let host = self.link.host();
        // Until a painter has reported what the host takes, only an
        // override says how to show the picture; a canvas no painter
        // reports to after a frame shows block glyphs.
        let environment = CanvasOutput::from_environment();
        let output = CanvasOutput::choose(host, self.props.options.output, environment);
        let named = self.props.options.output.is_some() || environment.is_some();
        if host.is_none() && !named && !waited {
            return None;
        }
        let cell = host.map_or(CELL_PIXELS, |host| host.cell);
        let cell = (cell.0.max(1), cell.1.max(1));
        let (want, size, columns, rows, pixels) = match output {
            CanvasOutput::Blocks => {
                let blitter = crate::widgets::display::image::image_blitter();
                let (across, down) = blitter.cell_pixels();
                let columns = columns.min(limits.side / across.max(1));
                let rows = rows.min(limits.side / down.max(1)).min(
                    (limits.pixels
                        / u64::from(columns.max(1) * across.max(1))
                        / u64::from(down.max(1))) as u32,
                );
                (
                    Want::Blocks { blitter, cell },
                    (columns, rows),
                    columns,
                    rows,
                    cell,
                )
            }
            CanvasOutput::Kitty | CanvasOutput::Sixel => {
                // What the output can carry: a Kitty picture sent in the
                // command has the room its base64 takes in the frame's
                // output; a picture through shared memory, or as Sixel, the
                // frame's room for one picture.
                let shared = host.is_some_and(|host| host.kitty_shared_memory);
                let budget = if output == CanvasOutput::Kitty && !shared {
                    DIRECT_PIXELS
                } else {
                    FRAME_PIXELS
                };
                let wanted = self.props.cell_pixels.map_or(cell, |pin| {
                    (pin.0.min(cell.0).max(1), pin.1.min(cell.1).max(1))
                });
                let pixels = pixels_per_cell(columns, rows, wanted, limits, budget)?;
                let size = (columns * u32::from(pixels.0), rows * u32::from(pixels.1));
                (Want::Pixels, size, columns, rows, pixels)
            }
        };
        if size.0 == 0 || size.1 == 0 {
            return None;
        }
        // The scene's coordinates are the terminal's pixels; a picture with
        // fewer pixels per cell is drawn smaller by the same ratio.
        let picture = (
            (columns * u32::from(pixels.0)) as f32,
            (rows * u32::from(pixels.1)) as f32,
        );
        let base = match (self.props.view, want) {
            (Some(view), _) => Transform::fit(view, picture),
            (None, Want::Pixels) if pixels != cell => Transform::scale(
                f32::from(pixels.0) / f32::from(cell.0),
                f32::from(pixels.1) / f32::from(cell.1),
            ),
            (None, _) => Transform::identity(),
        };
        Some((
            Job {
                scene: self.props.scene.clone(),
                size,
                want,
                base,
            },
            output,
            (columns, rows),
            pixels,
        ))
    }

    /// The renderer in words, once the worker has made it.
    fn renderer(&self) -> String {
        match (&self.worker, &self.failure) {
            (Some(worker), _) => worker
                .mode()
                .map_or_else(|| "starting".into(), |mode| mode.label()),
            (None, Some(failure)) => failure.clone(),
            (None, None) => "stopped".into(),
        }
    }
}

impl Component for Canvas {
    type Props = CanvasProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        let (worker, picture, failure) = Self::start(&props);
        Self {
            props,
            worker,
            picture,
            failure,
            link: Arc::default(),
            image_id: crate::widgets::display::image::ProtocolRenderer::new().generate_image_id(),
            viewport: None,
            view: Mutex::default(),
        }
    }

    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if !Self::same_worker(&self.props, props) {
            // Other options are another renderer: the canvas moves to its
            // thread.
            let (worker, picture, failure) = Self::start(props);
            self.picture = None;
            self.worker = worker;
            self.picture = picture;
            self.failure = failure;
            *self.view.get_mut().unwrap_or_else(|e| e.into_inner()) = View::default();
        }
        self.props = props.clone();
        true
    }

    fn layout(&mut self, layout: LayoutInfo, _: &mut Self::Props, _: &mut ()) -> bool {
        let changed = self
            .viewport
            .is_none_or(|old| old.content_size() != layout.content_size());
        self.viewport = Some(layout);
        changed
    }

    fn render(&self, _: &Self::Props, _: &()) -> Element {
        self.link.observe();
        let mut view = self.view.lock().unwrap_or_else(|e| e.into_inner());
        let wanted = self.job(view.waited);
        view.waited = true;
        let theme = crate::theme::Theme::generation();
        // A frame that could not hold the picture said so: the canvas
        // shows why and draws no more pictures of that size (GFX-007).
        let refused = wanted.as_ref().and_then(|(job, output, _, _)| {
            matches!(output, CanvasOutput::Kitty | CanvasOutput::Sixel)
                .then(|| self.link.refused(job.size))
                .flatten()
        });
        if let Some(picture) = &self.picture {
            picture.observe();
            if let Some((job, _, _, _)) = wanted.as_ref().filter(|_| refused.is_none()) {
                if !view
                    .submitted
                    .as_ref()
                    .is_some_and(|last| last.is(job, theme))
                {
                    picture.submit_job(job.clone());
                    view.submitted = Some(Submitted {
                        scene: job.scene.clone(),
                        size: job.size,
                        want: job.want,
                        base: job.base,
                        theme,
                    });
                }
            }
            if let Some(finished) = picture.latest() {
                view.shown = Some(finished);
            }
        }
        // A picture drawn for another size or another output is never
        // shown (BAR-003).
        let shown = view
            .shown
            .clone()
            .zip(wanted.as_ref())
            .filter(|(shown, (job, _, _, _))| shown.size == job.size && shown.want == job.want);
        drop(view);

        // The picture's cells: the area's, up to the largest picture. The
        // painter shows the picture over them, one picture pixel per screen
        // pixel unless the picture has fewer pixels per cell, which the
        // terminal scales to the cells (GFX-010).
        let area = self.cells();
        let (columns, rows) = wanted.as_ref().map_or(area, |(_, _, cells, _)| *cells);
        let insets = self.viewport.map_or([0.0; 4], |viewport| viewport.insets);
        let failure = self
            .failure
            .clone()
            .or_else(|| refused.clone())
            .or_else(|| shown.as_ref().and_then(|(shown, _)| shown.error.clone()));
        let message = failure.as_ref().map(|failure| format!("Canvas: {failure}"));
        let mut content =
            ElementBuilder::new(ElementType::Text(message.clone().unwrap_or_default()))
                .styles(
                    StyleBuilder::new()
                        .position_absolute()
                        .inset_left(insets[0])
                        .inset_top(insets[1])
                        .width_px(columns as f32)
                        .height_px(rows as f32)
                        .overflow_hidden(),
                )
                .class(if message.is_some() {
                    "whitespace-normal"
                } else {
                    "whitespace-pre"
                })
                .build();
        content.metadata.inert = true;
        let mut pixels = None;
        if let (Some((shown, (_, output, _, cell))), None) = (&shown, &message) {
            match (output, &shown.cells, &shown.frame) {
                (CanvasOutput::Blocks, Some(grid), _) => {
                    content = content.with_cells(grid.clone());
                }
                (CanvasOutput::Kitty | CanvasOutput::Sixel, _, Some(frame)) => {
                    pixels = Some(CanvasPixels {
                        frame: frame.clone(),
                        output: *output,
                        cell: *cell,
                    });
                }
                _ => {}
            }
        }
        // The painter shows the picture over the element that carries it.
        content.metadata.canvas = Some(Arc::new(CanvasPaint {
            id: self.image_id,
            link: self.link.clone(),
            pixels,
        }));
        let renderer = self.renderer();
        let label = self.props.label.as_deref().unwrap_or("Canvas");
        let mut root = ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
            .styles(
                StyleBuilder::new()
                    .display_flex()
                    .width_percent(100.0)
                    .height_percent(100.0)
                    .min_width_px(0.0)
                    .min_height_px(0.0)
                    .overflow_hidden(),
            )
            .children(vec![
                content,
                Element::text(format!("Canvas: {label}, drawn by {renderer}"))
                    .with_key("canvas-renderer")
                    .class("sr-only aria-live-polite"),
            ])
            .build();
        let mut node = crate::accessibility::Node::new(crate::accessibility::Role::Image);
        node.set_label(label);
        node.set_description(match &message {
            Some(message) => message.clone(),
            None => format!("Drawn by {renderer}"),
        });
        // Nothing to show yet for this size, or a picture in the colors of
        // a theme that is no longer the active one, which stays until the
        // next is drawn: the worker's finish signal, or the painter's
        // report, redraws the canvas.
        let finished = refused.is_some()
            || shown
                .as_ref()
                .is_some_and(|(shown, _)| shown.theme == theme);
        if !finished && self.worker.is_some() && area.0 > 0 && area.1 > 0 {
            node.set_busy();
        }
        root.metadata.accessibility = Some(node);
        root
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.picture = None;
            self.worker = None;
            *self.view.get_mut().unwrap_or_else(|e| e.into_inner()) = View::default();
        }
    }
}
