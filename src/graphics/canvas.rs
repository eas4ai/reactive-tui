use super::{
    FrameClock, GraphicsError, GraphicsFrame, GraphicsOptions, GraphicsWorker, HybridCubeRenderer,
};
use crate::{app::AppWaker, component::Element};
use std::time::Duration;

/// App-owned viewport canvas. All GPU/CPU rendering runs on its owned worker.
pub struct GraphicsCanvas {
    worker: GraphicsWorker,
    clock: FrameClock,
    frame: Option<GraphicsFrame>,
    viewport: Option<(u32, u32)>,
    stopped: bool,
}

impl GraphicsCanvas {
    /// Attach one worker to an App's coalescing wake handle.
    /// Adapter initialization is lazy and never executes on the App loop.
    pub fn new(wake: AppWaker, options: GraphicsOptions) -> Self {
        Self::start(wake, options, None)
    }

    /// Move a renderer initialized before terminal setup onto the owned worker.
    /// This keeps driver startup diagnostics out of the raw terminal session.
    pub fn with_renderer(wake: AppWaker, renderer: HybridCubeRenderer) -> Self {
        Self::start(wake, GraphicsOptions::default(), Some(renderer))
    }

    fn start(
        wake: AppWaker,
        options: GraphicsOptions,
        mut renderer: Option<HybridCubeRenderer>,
    ) -> Self {
        let worker = GraphicsWorker::spawn_cancellable(
            move |request, cancellation| {
                let renderer = renderer.get_or_insert_with(|| {
                    HybridCubeRenderer::new_cancellable(options, cancellation)
                });
                renderer.render(request, cancellation)
            },
            move || wake.wake(),
        );
        Self {
            worker,
            clock: FrameClock::default(),
            frame: None,
            viewport: None,
            stopped: false,
        }
    }

    /// Schedule a deadline request and consume the newest matching-viewport result.
    /// Returns true only when visible pixels or the viewport changed.
    pub fn advance(
        &mut self,
        elapsed: Duration,
        columns: u32,
        rows: u32,
    ) -> Result<bool, GraphicsError> {
        if self.stopped {
            return Err(GraphicsError::Cancelled);
        }
        let mut changed = self.viewport != Some((columns, rows));
        if changed {
            self.viewport = Some((columns, rows));
            self.frame = None;
        }
        if let Some(output) = self.worker.take_latest() {
            if output.request.columns() == columns && output.request.rows() == rows {
                self.frame = Some(output.frame?);
                changed = true;
            }
        }
        if let Some(request) = self.clock.request_at(elapsed, columns, rows)? {
            if !self.worker.request(request) {
                return Err(GraphicsError::Cancelled);
            }
        }
        Ok(changed)
    }

    /// Current pixels, or None during initialization/resize.
    pub fn frame(&self) -> Option<&GraphicsFrame> {
        self.frame.as_ref()
    }

    /// Visible provenance of the current pixels, or an explicit loading label.
    pub fn mode_label(&self) -> String {
        self.frame
            .as_ref()
            .map(|frame| frame.mode().label())
            .unwrap_or_else(|| "Starting graphics…".into())
    }

    /// Compose through ordinary styled text; no image protocol or window is used.
    pub fn element(&self) -> Result<Element, GraphicsError> {
        self.frame.as_ref().map_or_else(
            || Ok(Element::text("Preparing viewport…")),
            GraphicsFrame::to_half_block_element,
        )
    }

    /// Cancel pending/active work and join before the canvas is released.
    pub fn shutdown(&mut self) -> Result<(), GraphicsError> {
        self.stopped = true;
        self.worker.shutdown()
    }
}
