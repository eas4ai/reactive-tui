//! The `Canvas` widget (GFX-001): placed in an Element tree like a chart,
//! it fills the rectangle its parent allots and shows the picture of its
//! scene there. Its worker draws; the App's thread submits the scene and
//! shows the newest finished picture (GFX-003).

use super::hybrid::{GraphicsOptions, CELL_PIXELS};
use super::output::{CanvasLink, CanvasOutput, CanvasPaint};
use super::scene::{Scene, Transform};
use super::worker::{Finished, GraphicsWorker, Job, Want};
use super::{MAX_HEIGHT, MAX_WIDTH};
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
        }
    }

    /// Draw on `worker` instead of a worker the canvas starts itself. An
    /// application starts one before it sets the terminal up
    /// ([`GraphicsWorker::wait_ready`]); the worker's own options decide
    /// its renderer and font. One worker serves one canvas.
    pub fn worker(mut self, worker: Arc<GraphicsWorker>) -> Self {
        self.worker = Some(worker);
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
}

impl Submitted {
    fn is(&self, job: &Job) -> bool {
        self.size == job.size
            && self.want == job.want
            && self.base == job.base
            && (Arc::ptr_eq(&self.scene, &job.scene) || self.scene == job.scene)
    }
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
    /// Why there is no worker.
    failure: Option<String>,
    link: Arc<CanvasLink>,
    image_id: u32,
    viewport: Option<LayoutInfo>,
    view: Mutex<View>,
}

impl Canvas {
    /// The worker `props` names, else one started for them, or why there
    /// is none.
    fn start(props: &CanvasProps) -> (Option<Arc<GraphicsWorker>>, Option<String>) {
        if let Some(worker) = &props.worker {
            return (Some(worker.clone()), None);
        }
        match GraphicsWorker::spawn(props.options.clone()) {
            Ok(worker) => (Some(Arc::new(worker)), None),
            Err(error) => (None, Some(error.to_string())),
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

    /// The job for the canvas's area as it is, and the output it is for;
    /// `None` before the first layout, which gives the area its size.
    fn job(&self, waited: bool) -> Option<(Job, CanvasOutput)> {
        let (columns, rows) = self.cells();
        if columns == 0 || rows == 0 {
            return None;
        }
        let host = self.link.host();
        // Until a painter has reported what the host takes, only an
        // override says how to show the picture; a canvas no painter
        // reports to after a frame shows block glyphs.
        let output = CanvasOutput::choose(
            host,
            self.props.options.output,
            CanvasOutput::from_environment(),
        );
        let named =
            self.props.options.output.is_some() || CanvasOutput::from_environment().is_some();
        if host.is_none() && !named && !waited {
            return None;
        }
        let cell = host.map_or(CELL_PIXELS, |host| host.cell);
        let (cell_width, cell_height) = (u32::from(cell.0.max(1)), u32::from(cell.1.max(1)));
        let (want, size, columns, rows) = match output {
            CanvasOutput::Blocks => {
                let blitter = crate::widgets::display::image::image_blitter();
                let (across, down) = blitter.cell_pixels();
                let columns = columns.min(MAX_WIDTH / across);
                let rows = rows.min(MAX_HEIGHT / down);
                (
                    Want::Blocks { blitter, cell },
                    (columns, rows),
                    columns,
                    rows,
                )
            }
            CanvasOutput::Kitty | CanvasOutput::Sixel => {
                let columns = columns.min(MAX_WIDTH / cell_width);
                let rows = rows.min(MAX_HEIGHT / cell_height);
                (
                    Want::Pixels,
                    (columns * cell_width, rows * cell_height),
                    columns,
                    rows,
                )
            }
        };
        if size.0 == 0 || size.1 == 0 {
            return None;
        }
        let picture = ((columns * cell_width) as f32, (rows * cell_height) as f32);
        let base = self
            .props
            .view
            .map_or(Transform::identity(), |view| Transform::fit(view, picture));
        Some((
            Job {
                scene: self.props.scene.clone(),
                size,
                want,
                base,
            },
            output,
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
        let (worker, failure) = Self::start(&props);
        Self {
            props,
            worker,
            failure,
            link: Arc::default(),
            image_id: crate::widgets::display::image::ProtocolRenderer::new().generate_image_id(),
            viewport: None,
            view: Mutex::default(),
        }
    }

    fn update(&mut self, props: &Self::Props, _: &mut ()) -> bool {
        if !Self::same_worker(&self.props, props) {
            // Other options are another renderer: the worker starts anew.
            let (worker, failure) = Self::start(props);
            self.worker = worker;
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
        if let Some(worker) = &self.worker {
            worker.observe();
            if let Some((job, _)) = &wanted {
                if !view.submitted.as_ref().is_some_and(|last| last.is(job)) {
                    worker.submit_job(job.clone());
                    view.submitted = Some(Submitted {
                        scene: job.scene.clone(),
                        size: job.size,
                        want: job.want,
                        base: job.base,
                    });
                }
            }
            if let Some(finished) = worker.latest() {
                view.shown = Some(finished);
            }
        }
        // A picture drawn for another size or another output is never
        // shown (BAR-003).
        let shown = view
            .shown
            .clone()
            .zip(wanted.as_ref())
            .filter(|(shown, (job, _))| shown.size == job.size && shown.want == job.want);
        drop(view);

        let (columns, rows) = self.cells();
        let insets = self.viewport.map_or([0.0; 4], |viewport| viewport.insets);
        let failure = self
            .failure
            .clone()
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
        if let (Some((shown, (_, output))), None) = (&shown, &message) {
            match (output, &shown.cells, &shown.frame) {
                (CanvasOutput::Blocks, Some(grid), _) => {
                    content = content.with_cells(grid.clone());
                }
                (CanvasOutput::Kitty | CanvasOutput::Sixel, _, Some(frame)) => {
                    pixels = Some((frame.clone(), *output));
                }
                _ => {}
            }
        }
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
        // Nothing to show yet for this size: the worker's finish signal, or
        // the painter's report, redraws the canvas.
        if shown.is_none() && self.worker.is_some() && columns > 0 && rows > 0 {
            node.set_busy();
        }
        root.metadata.accessibility = Some(node);
        root.metadata.canvas = Some(Arc::new(CanvasPaint {
            id: self.image_id,
            link: self.link.clone(),
            pixels,
        }));
        root
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _: &mut ()) {
        if event == LifecycleEvent::Unmount {
            self.worker = None;
            *self.view.get_mut().unwrap_or_else(|e| e.into_inner()) = View::default();
        }
    }
}
