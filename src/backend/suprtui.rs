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
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

mod input;
mod output;
use output::{CheckedOutput, TerminalOutput};

type Reply = mpsc::Sender<Result<()>>;

enum Command {
    Present(
        FrameContent,
        (usize, usize),
        mpsc::Sender<Result<Vec<super::PaintedNode>>>,
    ),
    Shutdown(Reply),
}

enum FrameContent {
    Element(Box<Element>),
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
    frame: Element,
    cells: Option<Arc<CellFrame>>,
    painted_nodes: Vec<super::PaintedNode>,
    raw_mode: Option<RawMode>,
    input: Option<crossterm::event::EventStream>,
}

impl SuprTuiBackend {
    /// Enter raw mode and the alternate screen on stdout.
    pub fn new() -> Result<Self> {
        let (width, height) = crossterm::terminal::size()?;
        let raw_mode = RawMode::enter()?;
        let mut backend = Self::start(width, height, io::stdout(), true)?;
        backend.raw_mode = Some(raw_mode);
        Ok(backend)
    }

    /// Render to an owned writer without changing the host terminal session.
    pub fn with_writer<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
    ) -> Result<Self> {
        Self::start(width, height, writer, false)
    }

    fn start<W: Write + Send + 'static>(
        width: u16,
        height: u16,
        writer: W,
        terminal: bool,
    ) -> Result<Self> {
        let dimensions = (usize::from(width), usize::from(height));
        validate_size(dimensions)?;
        let (commands, receiver) = mpsc::sync_channel(0);
        let (ready, initialized) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("suprtui-renderer".into())
            .spawn(move || {
                run_worker(writer, terminal, dimensions, receiver, ready);
            })?;
        let mut backend = Self {
            commands: Some(commands),
            worker: Some(worker),
            dimensions,
            frame: Element::empty(),
            cells: None,
            painted_nodes: Vec::new(),
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

    /// Restore the session and join the worker. Safe to call more than once.
    pub fn shutdown(&mut self) -> Result<()> {
        self.input.take();
        let output_result = if let Some(commands) = self.commands.take() {
            let (reply, result) = mpsc::channel();
            commands
                .send(Command::Shutdown(reply))
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
        output_result.and(join_result).and(raw_result)
    }
}

impl Backend for SuprTuiBackend {
    fn painted_nodes(&self) -> Option<&[super::PaintedNode]> {
        Some(&self.painted_nodes)
    }
    fn render_frame(&mut self, element: &Element) -> Result<bool> {
        if self.commands.is_none() {
            return Err(worker_stopped());
        }
        self.cells = None;
        self.frame = element.clone();
        Ok(true)
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
        let element = tree
            .root()
            .and_then(|root| root.as_element())
            .ok_or_else(|| {
                ReactiveError::invalid_state("SuprTUI needs a complete Element frame")
            })?;
        self.render_full(element)
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
                    || FrameContent::Element(Box::new(self.frame.clone())),
                    |cells| FrameContent::Cells(Arc::clone(cells)),
                ),
                self.dimensions,
                reply,
            ))
            .map_err(|_| worker_stopped())?;
        self.painted_nodes = result.recv().map_err(|_| worker_stopped())??;
        Ok(())
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
        }
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
            eprintln!("SuprTUI cleanup failed: {error}");
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

fn run_worker<W: Write>(
    writer: W,
    terminal: bool,
    mut dimensions: (usize, usize),
    receiver: mpsc::Receiver<Command>,
    ready: Reply,
) {
    let writer = Rc::new(RefCell::new(writer));
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
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Present(spec, size, reply) => {
                let result = (|| {
                    if size != dimensions {
                        renderer = make_renderer(size)?;
                        dimensions = size;
                        force = true;
                    }
                    let geometry = match spec {
                        FrameContent::Element(spec) => {
                            let spec = element_to_paintspec(&spec)?;
                            let geometry = paint_frame(&spec, renderer.next_buffer())?;
                            renderer.set_cursor(0, 0, false);
                            geometry
                        }
                        FrameContent::Cells(frame) => {
                            frame.paint(renderer.next_buffer())?;
                            let (x, y) = frame.cursor().unwrap_or((0, 0));
                            renderer.set_cursor(
                                u32::from(x),
                                u32::from(y),
                                frame.cursor().is_some(),
                            );
                            Vec::new()
                        }
                    };
                    let status = renderer.render(force);
                    if let Some(error) = renderer.backend_mut().take_error() {
                        return Err(error.into());
                    }
                    if status == RenderStatus::Failed {
                        return Err(ReactiveError::terminal(
                            "SuprTUI could not publish the frame",
                        ));
                    }
                    Ok(geometry)
                })();
                force = result.is_err();
                let _ = reply.send(result);
            }
            Command::Shutdown(reply) => {
                let _ = reply.send(session.restore());
                return;
            }
        }
    }
}

struct RawMode {
    active: bool,
}

impl RawMode {
    fn enter() -> Result<Self> {
        // Do not take ownership of raw mode established by the caller.
        let active = !crossterm::terminal::is_raw_mode_enabled()?;
        if active {
            crossterm::terminal::enable_raw_mode()?;
        }
        Ok(Self { active })
    }

    fn restore(&mut self) -> Result<()> {
        if self.active {
            crossterm::terminal::disable_raw_mode()?;
            self.active = false;
        }
        Ok(())
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            eprintln!("Raw mode cleanup failed: {error}");
        }
    }
}
