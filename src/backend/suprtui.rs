//! Complete application frames rendered by SuprTUI on one owned worker.

use super::{Backend, CellFrame, CrosstermBackend};
use crate::component::{bridge::element_to_paintspec, Element};
use crate::error::{ReactiveError, Result};
use crate::event::types::Event;
use crate::layout::paint_tree::suprtui::paint_frame;
use crate::render::{reconcile::PatchOp, tree::RenderTree};
use ::suprtui::render::{RenderStatus, Renderer};
use ::suprtui::uni::pool::GraphemePool;
use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub(crate) mod graphics;
mod input;
mod output;
use output::{CheckedOutput, TerminalOutput};

/// The renderer worker's stack, and DebugBackend's paint thread's. Converting
/// and painting a frame recurses once per element level, about 14 KiB a level
/// in a debug build, so the deepest tree component expansion accepts (128
/// levels) needs about 1.8 MiB. The default thread stack is 2 MiB less the
/// static thread-local storage of the linked libraries (256 KiB with the
/// embedded-terminal feature), and RUST_MIN_STACK can lower it further, so
/// the worker sets its own.
pub(super) const RENDERER_STACK: usize = 8 << 20;

/// Host graphics settings for an owned writer. Cell dimensions are physical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageOutputOptions {
    /// Emit Kitty graphics when the host supports that protocol.
    pub kitty_graphics: bool,
    /// Emit Sixel graphics when supported by the host.
    pub sixel: bool,
    /// Emit iTerm2 inline image files when supported by the host.
    pub iterm2_inline: bool,
    /// Physical pixel width and height of a cell; each must be in 1..=256.
    pub cell_pixels: (u16, u16),
}
impl Default for ImageOutputOptions {
    fn default() -> Self {
        Self {
            kitty_graphics: false,
            sixel: false,
            iterm2_inline: false,
            cell_pixels: (8, 16),
        }
    }
}

impl ImageOutputOptions {
    pub(crate) fn protocol(
        self,
        mode: crate::widgets::ImageDisplayMode,
    ) -> Option<crate::widgets::display::image::paint::ImageProtocol> {
        use crate::widgets::{
            display::image::paint::ImageProtocol as Protocol, ImageDisplayMode as Mode,
        };
        match mode {
            Mode::Auto if self.sixel => Some(Protocol::Sixel),
            Mode::Auto | Mode::KittyGraphics if self.kitty_graphics => Some(Protocol::Kitty),
            Mode::Auto | Mode::ITerm2Inline if self.iterm2_inline => Some(Protocol::Inline),
            Mode::Sixel if self.sixel => Some(Protocol::Sixel),
            _ => None,
        }
    }
    pub(crate) fn refresh_cell_pixels(&mut self) {
        if let Ok(size) = crossterm::terminal::window_size() {
            if size.columns > 0 && size.rows > 0 {
                let cell = (size.width / size.columns, size.height / size.rows);
                if (1..=256).contains(&cell.0) && (1..=256).contains(&cell.1) {
                    self.cell_pixels = cell;
                }
            }
        }
    }
}

type Reply = mpsc::Sender<Result<()>>;

/// Geometry of the last acknowledged frame (PIP-002).
#[derive(Default)]
struct Acknowledged {
    nodes: Vec<super::PaintedNode>,
    layouts: Vec<super::PresentedLayout>,
    hits: Vec<u32>,
}

#[derive(Clone, Copy, Default)]
struct FrameOptions {
    full_redraw: bool,
    debug_overlay: bool,
}

enum Command {
    Present(
        FrameContent,
        (usize, usize),
        ImageOutputOptions,
        FrameOptions,
        mpsc::Sender<Result<super::PresentedGeometry>>,
    ),
    /// Lay a frame out at a size without painting or writing it.
    Layout(
        Arc<Element>,
        (usize, usize),
        mpsc::Sender<Result<super::FrameLayout>>,
    ),
    Shutdown(Reply, Option<String>),
    /// Reply once every earlier frame has been written and flushed.
    Sync(Reply),
    #[cfg(test)]
    Panic(&'static str),
}

enum FrameContent {
    Element(Arc<Element>),
    Cells(Arc<CellFrame>),
}

/// ANSI truecolor backend for applications inside a host terminal.
///
/// The worker owns SuprTUI's non-Send pools. Commands have no queue and each
/// presentation waits for checked output and a flush. Frames are limited to
/// 262,144 cells to reject accidental, excessive terminal allocations.
pub struct SuprTuiBackend {
    commands: Option<SyncSender<Command>>,
    worker: Option<JoinHandle<()>>,
    dimensions: (usize, usize),
    images: ImageOutputOptions,
    options: FrameOptions,
    /// The frame to present, shared with the worker without a copy (PNT-003).
    frame: Arc<Element>,
    cells: Option<Arc<CellFrame>>,
    painted_nodes: Vec<super::PaintedNode>,
    component_layouts: Vec<super::PresentedLayout>,
    /// Element index plus one per cell of the presented frame (PNT-002).
    hits: Vec<u32>,
    layout_reused: bool,
    layout_runs: u64,
    inverse_cells: u64,
    /// Geometry of the last frame whose flush the worker acknowledged; the
    /// fallback when a present reports the previous frame's failure (PIP-002).
    acknowledged: Acknowledged,
    raw_mode: Option<RawMode>,
    input: Option<crossterm::event::EventStream>,
}

impl SuprTuiBackend {
    /// Enter raw mode and the alternate screen on stdout.
    pub fn new() -> Result<Self> {
        let caps = crate::core::capabilities::TerminalQuery::detect_from_env();
        Self::new_with_images(ImageOutputOptions {
            kitty_graphics: caps.kitty_graphics,
            sixel: caps.sixel,
            iterm2_inline: caps.iterm2_graphics,
            ..Default::default()
        })
    }

    /// Enter an interactive terminal with caller-confirmed graphics support.
    /// Physical cell dimensions are refreshed from the terminal when available.
    pub fn new_with_images(mut images: ImageOutputOptions) -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let raw_mode = RawMode::enter()?;
        images.refresh_cell_pixels();
        let mut backend = Self::start(width, height, io::stdout(), true, images)?;
        backend.raw_mode = Some(raw_mode);
        Ok(backend)
    }

    /// Render to an owned writer without changing the host terminal session.
    pub fn with_writer<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
    ) -> Result<Self> {
        Self::start(width, height, writer, false, ImageOutputOptions::default())
    }

    /// Render graphics to a writer whose host capabilities and cell pixels are known.
    pub fn with_writer_and_images<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
        images: ImageOutputOptions,
    ) -> Result<Self> {
        Self::start(width, height, writer, false, images)
    }

    /// Own screen output while the native adapter owns raw mode and input.
    pub(crate) fn with_terminal_writer<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
        images: ImageOutputOptions,
    ) -> Result<Self> {
        Self::start(width, height, writer, true, images)
    }

    pub(crate) fn set_full_redraw(&mut self, enabled: bool) {
        self.options.full_redraw = enabled;
    }

    fn start<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
        terminal: bool,
        images: ImageOutputOptions,
    ) -> Result<Self> {
        if images.cell_pixels.0 == 0
            || images.cell_pixels.1 == 0
            || images.cell_pixels.0 > 256
            || images.cell_pixels.1 > 256
        {
            return Err(ReactiveError::invalid_parameter(
                "image cell dimensions must be between 1 and 256 pixels",
            ));
        }
        let dimensions = (usize::from(width), usize::from(height));
        validate_size(dimensions)?;
        let (commands, receiver) = mpsc::sync_channel(0);
        let (ready, initialized) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("suprtui-renderer".into())
            .stack_size(RENDERER_STACK)
            .spawn(move || {
                run_worker_guarded(writer, terminal, dimensions, receiver, ready, images);
            })?;
        let mut backend = Self {
            commands: Some(commands),
            worker: Some(worker),
            dimensions,
            images,
            options: FrameOptions::default(),
            frame: Arc::new(Element::empty()),
            cells: None,
            painted_nodes: Vec::new(),
            component_layouts: Vec::new(),
            hits: Vec::new(),
            layout_reused: false,
            layout_runs: 0,
            inverse_cells: 0,
            acknowledged: Acknowledged::default(),
            raw_mode: None,
            input: None,
        };
        match initialized.recv().map_err(|_| worker_stopped())? {
            Ok(()) => Ok(backend),
            Err(error) => {
                // The worker already attempted output restoration before replying.
                backend.commands.take();
                if let Some(worker) = backend.worker.take() {
                    let _ = worker.join();
                }
                Err(error)
            }
        }
    }

    /// Whether the last presented frame painted from the previous frame's
    /// layout because its layout inputs were unchanged (PNT-004).
    pub fn layout_reused(&self) -> bool {
        self.layout_reused
    }

    /// Layouts the render worker has computed for presented frames so far,
    /// counted where the layout engine runs; an unchanged spec presented at
    /// the same size adds none (PNT-004).
    pub fn layout_runs(&self) -> u64 {
        self.layout_runs
    }

    /// Cells the last presented frame's painter mapped through a node's
    /// inverse transform. Nodes that are only translated and not masked map
    /// cells by subtraction and add none (PNT-001).
    pub fn inverse_transformed_cells(&self) -> u64 {
        self.inverse_cells
    }

    /// Wait until every frame presented so far has been written and flushed.
    /// `present` returns before its frame's write (PIP-001); a caller that
    /// reads the output afterwards, such as a test, calls this first. A flush
    /// failure is reported here as it would be by the next present, and the
    /// geometry falls back the same way (PIP-002).
    pub fn sync(&mut self) -> Result<()> {
        let commands = self.commands.as_ref().ok_or_else(worker_stopped)?;
        let (reply, result) = mpsc::channel();
        commands
            .send(Command::Sync(reply))
            .map_err(|_| worker_stopped())?;
        let outcome = result.recv().map_err(|_| worker_stopped())?;
        if outcome.is_err() {
            self.fall_back_to_acknowledged();
        }
        outcome
    }

    /// Report the geometry of the last frame whose flush was acknowledged,
    /// after a failure shows the newest frame never reached the terminal.
    /// The next successful present then records that frame, not the failed
    /// one, as acknowledged (PIP-002).
    fn fall_back_to_acknowledged(&mut self) {
        self.painted_nodes = self.acknowledged.nodes.clone();
        self.component_layouts = self.acknowledged.layouts.clone();
        self.hits = self.acknowledged.hits.clone();
    }

    /// Restore the session and join the worker. Safe to call more than once.
    pub fn shutdown(&mut self) -> Result<()> {
        self.shutdown_with_panic(None)
    }

    fn shutdown_with_panic(&mut self, message: Option<&str>) -> Result<()> {
        self.input.take();
        let early_raw = if message.is_some() {
            self.raw_mode.as_mut().map_or(Ok(()), RawMode::restore)
        } else {
            Ok(())
        };
        let output_result = if let Some(commands) = self.commands.take() {
            let (reply, result) = mpsc::channel();
            commands
                .send(Command::Shutdown(reply, message.map(str::to_owned)))
                .map_err(|_| worker_stopped())
                .and_then(|()| result.recv().map_err(|_| worker_stopped()))
                .and_then(|result| result)
        } else {
            Ok(())
        };
        let join_result = self
            .worker
            .take()
            .map_or(Ok(()), |worker| worker.join().map_err(|_| worker_stopped()));
        let raw_result = self.raw_mode.as_mut().map_or(Ok(()), RawMode::restore);
        output_result
            .and(join_result)
            .and(early_raw)
            .and(raw_result)
    }
}

impl Backend for SuprTuiBackend {
    fn shutdown_after_panic(&mut self, message: &str) -> Result<()> {
        self.shutdown_with_panic(Some(message))
    }
    fn is_interactive_terminal(&self) -> bool {
        self.raw_mode.is_some()
    }

    fn painted_nodes(&self) -> Option<&[super::PaintedNode]> {
        Some(&self.painted_nodes)
    }
    fn hit_cells(&self) -> Option<&[u32]> {
        (!self.hits.is_empty()).then_some(self.hits.as_slice())
    }
    fn component_layouts(&self) -> Option<&[super::PresentedLayout]> {
        Some(&self.component_layouts)
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        if self.commands.is_none() {
            return Err(worker_stopped());
        }
        self.cells = None;
        // The one copy per present: the worker receives this handle.
        self.frame = Arc::new(element.clone());
        Ok(true)
    }

    fn layout_frame(&mut self, element: Arc<Element>) -> Result<Option<super::FrameLayout>> {
        validate_size(self.dimensions)?;
        let commands = self.commands.as_ref().ok_or_else(worker_stopped)?;
        let (reply, result) = mpsc::channel();
        commands
            .send(Command::Layout(element, self.dimensions, reply))
            .map_err(|_| worker_stopped())?;
        result.recv().map_err(|_| worker_stopped())?.map(Some)
    }

    fn render_cells(&mut self, frame: Arc<CellFrame>) -> Result<()> {
        if self.commands.is_none() {
            return Err(worker_stopped());
        }
        if frame.size() != self.size() {
            return Err(ReactiveError::invalid_parameter(
                "cell frame and backend dimensions differ",
            ));
        }
        self.cells = Some(frame);
        Ok(())
    }

    fn render_full(&mut self, element: &Element) -> Result<()> {
        self.render_frame(element).map(|_| ())
    }

    fn apply_patches(&mut self, _patches: &[PatchOp], tree: &RenderTree) -> Result<()> {
        if tree.root().is_none() {
            return self.clear();
        }
        let element = tree.root_element().ok_or_else(|| {
            ReactiveError::invalid_state("SuprTUI needs a complete Element frame")
        })?;
        self.render_full(&element)
    }

    fn clear(&mut self) -> Result<()> {
        self.render_full(&Element::empty())
    }

    fn present(&mut self) -> Result<()> {
        validate_size(self.dimensions)?;
        let commands = self.commands.as_ref().ok_or_else(worker_stopped)?;
        let (reply, result) = mpsc::channel();
        commands
            .send(Command::Present(
                self.cells.as_ref().map_or_else(
                    || FrameContent::Element(Arc::clone(&self.frame)),
                    |cells| FrameContent::Cells(Arc::clone(cells)),
                ),
                self.dimensions,
                self.images,
                self.options,
                reply,
            ))
            .map_err(|_| worker_stopped())?;
        // The worker replies as soon as layout and paint finish and writes
        // the bytes afterwards; the rendezvous channel makes the next present
        // wait for that flush, so one frame is in flight (PIP-001). An Ok
        // reply acknowledges the previous frame's flush; an Err reports its
        // failure and the geometry falls back to the last acknowledged frame
        // (PIP-002).
        match result.recv().map_err(|_| worker_stopped())? {
            Ok(geometry) => {
                self.acknowledged = Acknowledged {
                    nodes: std::mem::replace(&mut self.painted_nodes, geometry.nodes),
                    layouts: std::mem::replace(&mut self.component_layouts, geometry.layouts),
                    hits: std::mem::replace(&mut self.hits, geometry.hits),
                };
                self.layout_reused = geometry.layout_reused;
                self.layout_runs = geometry.layout_runs;
                self.inverse_cells = geometry.inverse_cells;
                Ok(())
            }
            Err(error) => {
                self.fall_back_to_acknowledged();
                Err(error)
            }
        }
    }

    fn sync(&mut self) -> Result<()> {
        SuprTuiBackend::sync(self)
    }

    fn size(&self) -> (u16, u16) {
        (
            self.dimensions.0.min(u16::MAX as usize) as u16,
            self.dimensions.1.min(u16::MAX as usize) as u16,
        )
    }

    fn resize(&mut self, width: usize, height: usize) {
        if width > 0 && height > 0 {
            self.dimensions = (width, height);
            if self.raw_mode.is_some() {
                self.images.refresh_cell_pixels();
            }
        }
    }

    fn set_debug_overlay(&mut self, enabled: bool) {
        self.options.debug_overlay = enabled;
    }

    fn poll_event(&mut self, timeout_ms: Option<u64>) -> Result<Option<Event>> {
        self.poll_event_with_wake(
            timeout_ms.map(Duration::from_millis),
            &crate::app::AppWaker::new(),
        )
    }

    fn poll_event_with_wake(
        &mut self,
        timeout: Option<Duration>,
        wake: &crate::app::AppWaker,
    ) -> Result<Option<Event>> {
        if self.commands.is_none() {
            return Err(worker_stopped());
        }
        // A resize can arrive while the first frame is being written, before
        // the lazily created input stream has subscribed to terminal events.
        if self.raw_mode.is_some() {
            let (width, height) = crossterm::terminal::size()?;
            if width > 0 && height > 0 && (width, height) != self.size() {
                return Ok(Some(Event::Resize(crate::event::types::ResizeEvent::new(
                    width, height,
                ))));
            }
        }
        let stream = self
            .input
            .get_or_insert_with(crossterm::event::EventStream::new);
        input::poll(stream, timeout, wake)
            .map(|event| event.and_then(CrosstermBackend::map_ct_event))
    }

    fn shutdown(&mut self) -> Result<()> {
        SuprTuiBackend::shutdown(self)
    }
}

impl Drop for SuprTuiBackend {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            log::warn!("SuprTUI cleanup failed: {error}");
        }
    }
}

fn validate_size((width, height): (usize, usize)) -> Result<()> {
    if width == 0
        || height == 0
        || width > u16::MAX as usize
        || height > u16::MAX as usize
        || width
            .checked_mul(height)
            .is_none_or(|cells| cells > 262_144)
    {
        return Err(ReactiveError::invalid_parameter(
            "SuprTUI dimensions must be nonzero u16 values with at most 262144 cells",
        ));
    }
    Ok(())
}

fn worker_stopped() -> ReactiveError {
    ReactiveError::terminal("SuprTUI renderer worker is closed or failed")
}

fn run_worker_guarded<W: Write>(
    writer: W,
    terminal: bool,
    dimensions: (usize, usize),
    receiver: mpsc::Receiver<Command>,
    ready: Reply,
    images: ImageOutputOptions,
) {
    let writer = Rc::new(RefCell::new(writer));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_worker(
            Rc::clone(&writer),
            terminal,
            dimensions,
            receiver,
            ready,
            images,
        );
    }));
    if let Err(payload) = &result {
        crate::app::cleanup_after_panic("Renderer output", || {
            super::write_panic(
                &mut *writer.borrow_mut(),
                crate::app::panic_message(payload.as_ref()),
            )
        });
    }
    crate::app::resume_caught_panic("SuprTUI renderer", result);
}

fn run_worker<W: Write>(
    writer: Rc<RefCell<W>>,
    terminal: bool,
    mut dimensions: (usize, usize),
    receiver: mpsc::Receiver<Command>,
    ready: Reply,
    mut images: ImageOutputOptions,
) {
    let mut session = TerminalOutput::new(Rc::clone(&writer), terminal);
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let make_renderer = |(width, height): (usize, usize)| {
        Renderer::new(
            width as u32,
            height as u32,
            Rc::clone(&pool),
            CheckedOutput::new(Rc::clone(&writer)),
        )
        .map_err(|error| ReactiveError::resource(format!("SuprTUI frame allocation: {error:?}")))
    };
    let initialized =
        make_renderer(dimensions).and_then(|renderer| session.enter().map(|()| renderer));
    let mut renderer = match initialized {
        Ok(renderer) => renderer,
        Err(error) => {
            let _ = session.restore();
            let _ = ready.send(Err(error));
            return;
        }
    };
    if ready.send(Ok(())).is_err() {
        return;
    }
    let mut force = true;
    let mut graphics = graphics::Graphics::default();
    let mut layout_cache = crate::layout::paint_tree::suprtui::LayoutCache::default();
    // A flush failure from the previous frame, reported by the next present
    // or by shutdown (PIP-002).
    let mut deferred: Option<ReactiveError> = None;
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Present(spec, size, current_images, options, reply) => {
                if let Some(error) = deferred.take() {
                    force = true;
                    let _ = reply.send(Err(error));
                    continue;
                }
                let result = (|| {
                    if images != current_images {
                        images = current_images;
                        force = true;
                    }
                    if size != dimensions {
                        renderer = make_renderer(size)?;
                        dimensions = size;
                        force = true;
                    }
                    let geometry = match spec {
                        FrameContent::Element(spec) => {
                            let started = std::time::Instant::now();
                            let spec = element_to_paintspec(&spec)?;
                            let (buffer, hits) = renderer.next_buffer_and_hit_grid();
                            let geometry =
                                paint_frame(spec, buffer, hits, &mut layout_cache, images)?;
                            renderer.set_layout_ns(started.elapsed().as_nanos() as u64);
                            geometry
                        }
                        FrameContent::Cells(frame) => {
                            let started = std::time::Instant::now();
                            frame.paint(renderer.next_buffer())?;
                            renderer.set_layout_ns(started.elapsed().as_nanos() as u64);
                            let (x, y) = frame.cursor().unwrap_or((0, 0));
                            super::PresentedGeometry {
                                cursor: frame.cursor().map(|_| ::suprtui::render::CursorState {
                                    x: u32::from(x),
                                    y: u32::from(y),
                                    visible: true,
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }
                        }
                    };
                    let mut geometry = geometry;
                    if options.debug_overlay {
                        let stats = renderer.stats();
                        let buffer = renderer.next_buffer();
                        let row = buffer.height() - 1;
                        let us = |ns: u64| ns / 1000;
                        let text = format!(
                            "frame: {} | cells: {} | bytes: {} | moves: {}/{} | fg: {}/{} | bg: {}/{} | attr: {}/{} | layout: {}us | diff: {}us | emit: {}us | write: {}us",
                            stats.frame_count,
                            stats.cells_updated,
                            stats.bytes_emitted,
                            stats.moves_emitted,
                            stats.moves_elided,
                            stats.fg_emitted,
                            stats.fg_elided,
                            stats.bg_emitted,
                            stats.bg_elided,
                            stats.attr_emitted,
                            stats.attr_elided,
                            us(stats.layout_ns),
                            us(stats.diff_ns),
                            us(stats.emit_ns),
                            us(stats.write_ns)
                        );
                        let text = format!("{text:width$}", width = buffer.width() as usize);
                        buffer
                            .draw_text(
                                &text,
                                0,
                                row,
                                ::suprtui::ansi::rgb_color(0, 0, 0, 255),
                                Some(::suprtui::ansi::rgb_color(255, 255, 255, 255)),
                                0,
                            )
                            .map_err(|error| {
                                ReactiveError::resource(format!("debug overlay: {error:?}"))
                            })?;
                        for node in &mut geometry.nodes {
                            node.bounds.height = node
                                .bounds
                                .height
                                .min((row as f32 - node.bounds.y).max(0.0));
                        }
                    }
                    let redraw = force || options.full_redraw;
                    let commands =
                        graphics.prepare(&geometry.images, images.cell_pixels, redraw)?;
                    let cursor = geometry
                        .cursor
                        .filter(|cursor| !graphics.covers_cell(cursor.x, cursor.y))
                        .unwrap_or_default();
                    renderer.set_cursor(cursor.x, cursor.y, cursor.visible);
                    renderer.set_cursor_style(cursor.style, cursor.blinking);
                    renderer.set_cursor_color(cursor.color);
                    let graphics_changed = commands.is_some();
                    let (before, after) = commands.unwrap_or_default();
                    renderer.backend_mut().set_graphics(before, after);
                    let status = renderer.render(redraw || graphics_changed);
                    if let Some(error) = renderer.backend_mut().take_error() {
                        return Err(error.into());
                    }
                    if status == RenderStatus::Failed {
                        return Err(ReactiveError::terminal(
                            "SuprTUI could not publish the frame",
                        ));
                    }
                    graphics.acknowledge(std::mem::take(&mut geometry.images));
                    geometry.hits = renderer.committed_hit_grid().to_vec();
                    Ok(geometry)
                })();
                force = result.is_err();
                let _ = reply.send(result);
                // Write and flush after the reply (PIP-001); the next present
                // waits on the rendezvous until this returns.
                // The write is timed here, where it happens, and counted in
                // the frame's stats that the next overlay shows (RAS-006).
                let pending = renderer.backend().has_pending();
                let started = std::time::Instant::now();
                let flushed = renderer.backend_mut().flush_pending();
                if pending {
                    renderer.add_write_ns(started.elapsed().as_nanos() as u64);
                }
                if let Err(error) = flushed {
                    deferred = Some(error.into());
                    renderer.flush_failed();
                    force = true;
                }
            }
            Command::Layout(element, (width, height), reply) => {
                // A flush failure stays deferred for the next present, sync
                // or shutdown.
                let _ = reply.send(element_to_paintspec(&element).and_then(|spec| {
                    crate::layout::paint_tree::suprtui::layout_frame(
                        spec,
                        (width as u32, height as u32),
                        &mut layout_cache,
                    )
                }));
            }
            Command::Sync(reply) => {
                let outcome = match deferred.take() {
                    Some(error) => {
                        force = true;
                        Err(error)
                    }
                    None => Ok(()),
                };
                let _ = reply.send(outcome);
            }
            Command::Shutdown(reply, message) => {
                let cleanup = renderer.backend_mut().finish_graphics(graphics.cleanup());
                let restored = session.restore_with_panic(message.as_deref());
                let outcome = match deferred.take() {
                    Some(error) => Err(error),
                    None => cleanup.and(restored),
                };
                let _ = reply.send(outcome);
                return;
            }
            #[cfg(test)]
            Command::Panic(marker) => panic!("{marker}"),
        }
    }
    if let Err(error) = renderer.backend_mut().finish_graphics(graphics.cleanup()) {
        log::warn!("Image output cleanup failed: {error}");
    }
}

struct RawMode {
    active: bool,
}

#[derive(Default)]
struct RawModeOwners {
    count: usize,
    enabled_by_library: bool,
}

static RAW_MODE_OWNERS: Mutex<RawModeOwners> = Mutex::new(RawModeOwners {
    count: 0,
    enabled_by_library: false,
});

impl RawModeOwners {
    fn acquire(&mut self, already_raw: bool) -> bool {
        let enable = self.count == 0 && !already_raw;
        if self.count == 0 {
            self.enabled_by_library = enable;
        }
        self.count += 1;
        enable
    }

    fn release(&mut self) -> bool {
        if self.count == 0 {
            return false;
        }
        self.count -= 1;
        self.count == 0 && self.enabled_by_library
    }
}

impl RawMode {
    fn enter() -> Result<Self> {
        let mut owners = RAW_MODE_OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        let already_raw = owners.count != 0 || crossterm::terminal::is_raw_mode_enabled()?;
        if owners.acquire(already_raw) {
            if let Err(error) = crossterm::terminal::enable_raw_mode() {
                owners.count = 0;
                owners.enabled_by_library = false;
                return Err(error.into());
            }
        }
        Ok(Self { active: true })
    }

    fn restore(&mut self) -> Result<()> {
        if self.active {
            let mut owners = RAW_MODE_OWNERS.lock().unwrap_or_else(|e| e.into_inner());
            if owners.release() {
                if let Err(error) = crossterm::terminal::disable_raw_mode() {
                    owners.count = 1;
                    return Err(error.into());
                }
                owners.enabled_by_library = false;
            }
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            log::warn!("Raw mode cleanup failed: {error}");
        }
    }
}

#[cfg(test)]
mod cursor_tests;

#[cfg(test)]
mod raw_mode_tests {
    use super::RawModeOwners;

    #[test]
    fn api019_independent_library_owners_restore_only_after_the_last_release() {
        let mut owners = RawModeOwners::default();
        assert!(owners.acquire(false));
        assert!(!owners.acquire(true));
        assert!(!owners.release());
        assert!(owners.release());
        owners.enabled_by_library = false;
        assert_eq!(owners.count, 0);

        assert!(!owners.acquire(true));
        assert!(!owners.release());
        assert!(!owners.enabled_by_library);
    }
}

#[cfg(all(test, unix))]
mod trl_001_tests {
    use super::{Command, SuprTuiBackend};

    #[derive(Clone, Default)]
    struct Capture(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn worker_panic_replays_through_the_owned_writer_after_restoration() {
        let capture = Capture::default();
        let mut backend = SuprTuiBackend::with_terminal_writer(
            20,
            8,
            capture.clone(),
            super::ImageOutputOptions::default(),
        )
        .unwrap();
        backend
            .commands
            .as_ref()
            .unwrap()
            .send(Command::Panic("OWNED_WORKER_PANIC"))
            .unwrap();
        assert!(backend.shutdown().is_err());
        let bytes = capture.0.lock().unwrap();
        let output = String::from_utf8_lossy(&bytes);
        let restored = output.rfind("\x1b[?1049l").unwrap();
        let message = output.rfind("OWNED_WORKER_PANIC").unwrap();
        assert!(message > restored, "{output:?}");
    }

    #[test]
    fn app_panic_preserves_payload_and_replays_after_restoration() {
        use crate::{app::RootComponent, component::Element};
        struct Panics;
        impl RootComponent for Panics {
            fn render(&self) -> Element {
                std::panic::panic_any(117_u32);
            }
        }
        let capture = Capture::default();
        let backend = SuprTuiBackend::with_terminal_writer(
            20,
            8,
            capture.clone(),
            super::ImageOutputOptions::default(),
        )
        .unwrap();
        let app = crate::app::App::builder()
            .backend(backend)
            .root(Panics)
            .build()
            .unwrap();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.run()));
        assert_eq!(result.unwrap_err().downcast_ref::<u32>(), Some(&117));
        let bytes = capture.0.lock().unwrap();
        let output = String::from_utf8_lossy(&bytes);
        let restored = output.rfind("\x1b[?1049l").unwrap();
        let message = output.rfind("non-string panic payload").unwrap();
        assert!(message > restored, "{output:?}");
    }

    #[test]
    fn panic_reporting_failure_preserves_the_original_worker_payload() {
        struct FailsReport(Capture);
        impl std::io::Write for FailsReport {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if bytes.starts_with(b"Framework panicked: ") {
                    panic!("REPORT_WRITER_FAILED");
                }
                std::io::Write::write(&mut self.0, bytes)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let (commands, receiver) = std::sync::mpsc::channel();
        commands
            .send(Command::Panic("ORIGINAL_WORKER_PANIC"))
            .unwrap();
        let (ready, _initialized) = std::sync::mpsc::channel();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::run_worker_guarded(
                FailsReport(Capture::default()),
                true,
                (20, 8),
                receiver,
                ready,
                super::ImageOutputOptions::default(),
            );
        }));
        let payload = result.unwrap_err();
        assert_eq!(
            crate::app::panic_message(payload.as_ref()),
            "ORIGINAL_WORKER_PANIC"
        );
    }

    #[test]
    fn panic_output_encodes_controls_and_bounds_the_message() {
        let mut bytes = Vec::new();
        crate::backend::write_panic(&mut bytes, "a\x1b[31m\n\u{009f}b").unwrap();
        assert_eq!(bytes, b"Framework panicked: a [31m  b\r\n");
        bytes.clear();
        crate::backend::write_panic(&mut bytes, &"e".repeat(9000)).unwrap();
        assert_eq!(bytes.len(), b"Framework panicked: ".len() + 8192 + 2);
    }

    #[test]
    fn app_teardown_retains_the_local_hook_scope() {
        use crate::{app::RootComponent, component::Element, reactive::Hooks};
        struct LocalRoot {
            wake: Option<crate::app::AppWaker>,
            finished: std::sync::Arc<std::sync::atomic::AtomicBool>,
        }
        impl RootComponent for LocalRoot {
            fn attach_waker(&mut self, wake: crate::app::AppWaker) {
                self.wake = Some(wake);
            }
            fn render(&self) -> Element {
                self.wake.as_ref().unwrap().request_stop();
                Element::text("local teardown")
            }
        }
        impl Drop for LocalRoot {
            fn drop(&mut self) {
                let hooks = Hooks::new();
                assert_eq!(crate::hooks::use_local_ref(&hooks, 99_u32).current(), 99);
                self.finished
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
        let finished = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let root = LocalRoot {
            wake: None,
            finished: finished.clone(),
        };
        let backend = SuprTuiBackend::with_writer(20, 8, std::io::sink()).unwrap();
        crate::app::App::builder()
            .backend(backend)
            .root(root)
            .build()
            .unwrap()
            .run()
            .unwrap();
        assert!(finished.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    #[ignore = "invoked by the TRL-001 PTY mechanism"]
    fn worker_panic_restores_the_owned_terminal() {
        if std::env::var("REACTIVE_TUI_TRL_001_PROBE").as_deref() != Ok("worker") {
            eprintln!(
                "SKIP: run by the TRL-001 PTY mechanism with REACTIVE_TUI_TRL_001_PROBE=worker"
            );
            return;
        }
        let mut backend = SuprTuiBackend::new().expect("PTY backend must start");
        backend
            .commands
            .as_ref()
            .expect("worker command sender must exist")
            .send(Command::Panic("TRL001_WORKER_PANIC"))
            .expect("worker must receive panic command");
        assert!(
            backend.shutdown().is_err(),
            "worker panic must close commands"
        );
    }
}
