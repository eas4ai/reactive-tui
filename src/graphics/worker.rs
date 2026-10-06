//! The drawing thread (GFX-003): one named thread that owns the renderer
//! and draws the newest scene of every picture it serves. A canvas is one
//! picture; a scene it submits while an earlier scene of the same picture
//! still waits replaces that scene and no other's. When a picture is
//! finished the thread tells the Apps through its waker, and it waits for
//! nothing between pictures. The canvases of a process that draw with the
//! same renderer options share one such thread; an application may start
//! one of its own and hand it to its canvases.

use super::hybrid::{GraphicsFault, GraphicsMode, GraphicsOptions, HybridRenderer};
use super::scene::{Scene, Transform};
use super::{GraphicsError, GraphicsFrame, PictureLimits};
use crate::layout::CellGrid;
use crate::reactive::ThreadSafeSignal;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Weak};
use suprtui::blit::Blitter;

/// What the thread makes of a scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Want {
    /// A picture of `size` pixels.
    Pixels,
    /// Block glyphs for `size` cells, each of `cell` pixels in the scene.
    Blocks { blitter: Blitter, cell: (u16, u16) },
}

/// One scene to draw.
#[derive(Clone)]
pub(crate) struct Job {
    pub scene: Arc<Scene>,
    /// Pixels, or cells for [`Want::Blocks`].
    pub size: (u32, u32),
    pub want: Want,
    /// Applied to every scene coordinate.
    pub base: Transform,
}

/// What the thread made of a job.
#[derive(Clone)]
pub(crate) struct Finished {
    /// The number [`Picture::submit_job`] gave the job.
    #[cfg_attr(not(test), allow(dead_code))]
    pub id: u64,
    pub size: (u32, u32),
    pub want: Want,
    /// The picture; for block glyphs, the picture they were made from.
    pub frame: Option<Arc<GraphicsFrame>>,
    pub cells: Option<Arc<CellGrid>>,
    /// Why there is no picture: both renderers failed (GFX-007).
    pub error: Option<String>,
    /// [`crate::theme::Theme::generation`] when the thread began to draw:
    /// the theme whose colors the picture's tokens took (GFX-001).
    pub theme: u64,
}

/// What the thread has done so far.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkerStats {
    /// Scenes replaced by a newer one of the same picture before the thread
    /// drew them.
    pub replaced: u64,
    /// Pictures finished, failed ones included.
    pub rendered: u64,
}

/// Names one picture the thread serves.
type Slot = u64;

/// The newest finished picture of one slot.
#[derive(Default)]
struct Served {
    finished: Option<Finished>,
    /// Whether [`GraphicsWorker::take_latest`] gave `finished` out.
    taken: bool,
}

#[derive(Default)]
struct Slots {
    /// The scene of each picture that waits, oldest submission first; one
    /// per picture, so a newer scene of a picture takes its place.
    waiting: Vec<(Slot, u64, Job)>,
    /// Every picture the thread serves, by its slot.
    served: HashMap<Slot, Served>,
    /// The renderer that draws, once the thread has made it.
    mode: Option<GraphicsMode>,
    /// The largest picture that renderer draws (GFX-010).
    limits: Option<PictureLimits>,
    next_id: u64,
    next_slot: Slot,
    stats: WorkerStats,
}

struct Shared {
    slots: Mutex<Slots>,
    /// A job waits, or the thread is to end.
    ready: Condvar,
    /// The thread has made its renderer.
    started: Condvar,
    closed: AtomicBool,
    /// Counts the pictures finished, and the renderer's readiness once, so
    /// the Apps that observe it redraw each time.
    changed: ThreadSafeSignal<u64>,
}

impl Shared {
    /// The slots, also after a holder of the lock panicked: they hold no
    /// state a panic can leave half written.
    fn slots(&self) -> MutexGuard<'_, Slots> {
        self.slots.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn open_slot(self: &Arc<Self>) -> Picture {
        let mut slots = self.slots();
        let slot = slots.next_slot;
        slots.next_slot += 1;
        slots.served.insert(slot, Served::default());
        Picture {
            shared: Arc::clone(self),
            slot,
        }
    }
}

/// One picture served by a drawing thread: a canvas submits its scenes
/// through it and reads what the thread made of the newest. Dropping it
/// forgets the picture's waiting scene and its finished picture.
pub(crate) struct Picture {
    shared: Arc<Shared>,
    slot: Slot,
}

impl Picture {
    /// Draw `job` instead of any scene of this picture still waiting; the
    /// number it finishes under.
    pub fn submit_job(&self, job: Job) -> u64 {
        let mut slots = self.shared.slots();
        slots.next_id += 1;
        let id = slots.next_id;
        // One place per picture holds the scene that waits, so a newer
        // scene of the same picture takes it and no other picture's.
        match slots
            .waiting
            .iter_mut()
            .find(|(slot, _, _)| *slot == self.slot)
        {
            Some(waiting) => {
                *waiting = (self.slot, id, job);
                slots.stats.replaced += 1;
            }
            None => slots.waiting.push((self.slot, id, job)),
        }
        drop(slots);
        self.shared.ready.notify_one();
        id
    }

    /// What the thread made of the newest job of this picture it finished.
    pub fn latest(&self) -> Option<Finished> {
        self.shared
            .slots()
            .served
            .get(&self.slot)
            .and_then(|served| served.finished.clone())
    }

    /// The newest finished picture, once.
    fn take_latest(&self) -> Option<Arc<GraphicsFrame>> {
        let mut slots = self.shared.slots();
        let served = slots.served.get_mut(&self.slot)?;
        if served.taken {
            return None;
        }
        let frame = served.finished.as_ref()?.frame.clone()?;
        served.taken = true;
        Some(frame)
    }

    /// Have the App that is rendering redraw when a picture is finished or
    /// the renderer is ready.
    pub fn observe(&self) {
        self.shared.changed.get();
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        let mut slots = self.shared.slots();
        slots.waiting.retain(|(slot, _, _)| *slot != self.slot);
        slots.served.remove(&self.slot);
    }
}

/// What tells two renderers apart: the options that choose and make one.
/// The output a canvas shows its picture as is not among them.
#[derive(Clone, PartialEq, Eq)]
struct RendererKey {
    force_cpu: bool,
    fault: Option<GraphicsFault>,
    font: super::fonts::FontSource,
}

impl RendererKey {
    fn of(options: &GraphicsOptions) -> Self {
        Self {
            force_cpu: options.force_cpu,
            fault: options.fault,
            font: options.font.clone(),
        }
    }
}

/// The drawing threads of this process by their renderer, for the canvases
/// that name no worker of their own (GFX-003).
static SHARED: Mutex<Vec<(RendererKey, Weak<GraphicsWorker>)>> = Mutex::new(Vec::new());

/// A drawing thread named `rtui-canvas-*` that draws the scenes of every
/// picture it serves, and the handle's own picture among them.
pub struct GraphicsWorker {
    shared: Arc<Shared>,
    /// The handle's own picture, which [`GraphicsWorker::submit`] draws.
    own: Picture,
}

impl std::fmt::Debug for GraphicsWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slots = self.shared.slots();
        f.debug_struct("GraphicsWorker")
            .field("mode", &slots.mode)
            .field("pictures", &slots.served.len())
            .field("stats", &slots.stats)
            .finish()
    }
}

static NEXT_NAME: AtomicUsize = AtomicUsize::new(0);

impl GraphicsWorker {
    /// Start a drawing thread of its own. It makes its renderer itself, so
    /// waiting for the adapter takes none of the caller's time. Hand it to
    /// several canvases with [`super::CanvasProps::worker`] and it serves
    /// them all.
    pub fn spawn(options: GraphicsOptions) -> Result<Self, GraphicsError> {
        let shared = Arc::new(Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            started: Condvar::new(),
            closed: AtomicBool::new(false),
            changed: ThreadSafeSignal::new(0),
        });
        let owner = shared.clone();
        let name = format!("rtui-canvas-{}", NEXT_NAME.fetch_add(1, Ordering::Relaxed));
        std::thread::Builder::new()
            .name(name)
            .spawn(move || run(owner, options))
            .map_err(|error| GraphicsError::Worker(error.to_string()))?;
        let own = shared.open_slot();
        Ok(Self { shared, own })
    }

    /// The drawing thread of this process for the renderer `options`
    /// choose, started when none runs; it ends when its last handle is
    /// dropped. Every canvas that names no worker draws on it, so a tree of
    /// many canvases starts one thread (GFX-003).
    pub fn shared(options: &GraphicsOptions) -> Result<Arc<Self>, GraphicsError> {
        let key = RendererKey::of(options);
        let mut threads = SHARED.lock().unwrap_or_else(|e| e.into_inner());
        threads.retain(|(_, weak)| weak.strong_count() > 0);
        if let Some(worker) = threads
            .iter()
            .find(|(known, _)| *known == key)
            .and_then(|(_, weak)| weak.upgrade())
        {
            return Ok(worker);
        }
        let worker = Arc::new(Self::spawn(options.clone())?);
        threads.push((key, Arc::downgrade(&worker)));
        Ok(worker)
    }

    /// A picture of its own on this thread, for a canvas.
    pub(crate) fn picture(&self) -> Picture {
        self.shared.open_slot()
    }

    /// Draw `scene` at `size` pixels as the handle's own picture, instead
    /// of any scene of it still waiting. Returns at once.
    pub fn submit(&self, scene: Arc<Scene>, size: (u32, u32)) {
        self.own.submit_job(Job {
            scene,
            size,
            want: Want::Pixels,
            base: Transform::identity(),
        });
    }

    /// Draw `job` as the handle's own picture, instead of any scene of it
    /// still waiting; the number it finishes under.
    #[cfg(test)]
    pub(crate) fn submit_job(&self, job: Job) -> u64 {
        self.own.submit_job(job)
    }

    /// The newest finished picture of the handle's own, once.
    pub fn take_latest(&self) -> Option<Arc<GraphicsFrame>> {
        self.own.take_latest()
    }

    /// The renderer that draws; `None` until the thread has made it.
    pub fn mode(&self) -> Option<GraphicsMode> {
        self.shared.slots().mode.clone()
    }

    /// The largest picture the renderer draws; `None` until the thread has
    /// made it (GFX-010).
    pub fn limits(&self) -> Option<PictureLimits> {
        self.shared.slots().limits
    }

    /// Wait up to `timeout` for the thread to make its renderer, and return
    /// it. An application calls this before it sets the terminal up, so
    /// that what a graphics driver prints while it starts does not land on
    /// the App's screen; the App's thread never calls it.
    pub fn wait_ready(&self, timeout: std::time::Duration) -> Option<GraphicsMode> {
        let deadline = std::time::Instant::now() + timeout;
        let mut slots = self.shared.slots();
        while slots.mode.is_none() {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                break;
            }
            slots = self
                .shared
                .started
                .wait_timeout(slots, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        slots.mode.clone()
    }

    /// What the thread has done so far.
    pub fn stats(&self) -> WorkerStats {
        self.shared.slots().stats
    }
}

impl Drop for GraphicsWorker {
    /// The thread ends after the picture it is drawing. Nothing waits for
    /// it: the owner may be the App's thread (GFX-003).
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared.slots().waiting.clear();
        self.shared.ready.notify_one();
    }
}

/// PLT-016: filter the driver's warning while preserving other stderr bytes.
#[cfg(unix)]
struct QuietTerminalStderr {
    saved: Option<std::fs::File>,
    forwarder: Option<std::thread::JoinHandle<std::io::Result<()>>>,
    /// Renderer startups take turns so each saves the actual terminal.
    _turn: std::sync::MutexGuard<'static, ()>,
}

#[cfg(unix)]
static QUIET_STDERR: Mutex<()> = Mutex::new(());

#[cfg(unix)]
impl QuietTerminalStderr {
    fn start() -> Self {
        let turn = QUIET_STDERR.lock().unwrap_or_else(|e| e.into_inner());
        let mut guard = Self {
            saved: None,
            forwarder: None,
            _turn: turn,
        };
        // SAFETY: isatty only inspects the process's stderr descriptor.
        if unsafe { libc::isatty(libc::STDERR_FILENO) } != 1 {
            return guard;
        }
        match Self::capture() {
            Ok((saved, forwarder)) => {
                guard.saved = Some(saved);
                guard.forwarder = Some(forwarder);
            }
            Err(error) => eprintln!("graphics stderr filter could not start: {error}"),
        }
        guard
    }

    fn capture() -> std::io::Result<(std::fs::File, std::thread::JoinHandle<std::io::Result<()>>)> {
        use std::os::fd::{AsRawFd, FromRawFd};
        // SAFETY: each new descriptor is immediately owned by a File.
        let (saved, reader, writer) = unsafe {
            let fd = libc::fcntl(libc::STDERR_FILENO, libc::F_DUPFD_CLOEXEC, 0);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let saved = std::fs::File::from_raw_fd(fd);
            let mut pipe = [-1; 2];
            if libc::pipe(pipe.as_mut_ptr()) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let reader = std::fs::File::from_raw_fd(pipe[0]);
            let writer = std::fs::File::from_raw_fd(pipe[1]);
            for fd in pipe {
                if libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            (saved, reader, writer)
        };
        let terminal = saved.try_clone()?;
        let forwarder = std::thread::Builder::new()
            .name("rtui-stderr".into())
            .spawn(move || forward_stderr(reader, terminal))?;
        // SAFETY: writer owns a live pipe descriptor; dup2 replaces descriptor 2.
        if unsafe { libc::dup2(writer.as_raw_fd(), libc::STDERR_FILENO) } < 0 {
            let error = std::io::Error::last_os_error();
            drop(writer);
            let _ = forwarder.join();
            return Err(error);
        }
        // Only descriptor 2 keeps the write end alive; restoring it produces EOF.
        drop(writer);
        Ok((saved, forwarder))
    }
}

#[cfg(unix)]
fn forward_stderr(mut reader: std::fs::File, mut terminal: std::fs::File) -> std::io::Result<()> {
    use std::io::{Read, Write};
    let mut buffer = [0; 4096];
    let mut line = Vec::new();
    let mut passthrough = false;
    loop {
        let count = match reader.read(&mut buffer) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            if passthrough || !driver_conformance_warning(&line) {
                terminal.write_all(&line)?;
            }
            return Ok(());
        }
        for &byte in &buffer[..count] {
            line.push(byte);
            if byte == b'\n' {
                if passthrough || !driver_conformance_warning(&line) {
                    terminal.write_all(&line)?;
                }
                line.clear();
                passthrough = false;
            } else if line.len() == 8192 {
                // Bound memory for arbitrary diagnostics without dropping bytes.
                terminal.write_all(&line)?;
                line.clear();
                passthrough = true;
            }
        }
    }
}

#[cfg(unix)]
fn driver_conformance_warning(line: &[u8]) -> bool {
    // PLT-016: exactly the line a Mesa Vulkan driver prints about itself,
    // "WARNING: <driver> is not a conformant Vulkan implementation, testing
    // use only." (radv, lavapipe and the others), and no other diagnostic
    // that happens to quote those words.
    let text = String::from_utf8_lossy(line);
    let text = text.trim_end_matches(['\r', '\n', ' ']);
    let Some(rest) = text.strip_prefix("WARNING: ") else {
        return false;
    };
    let Some((driver, tail)) = rest.split_once(' ') else {
        return false;
    };
    !driver.is_empty()
        && driver
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && tail.eq_ignore_ascii_case("is not a conformant Vulkan implementation, testing use only.")
}

#[cfg(unix)]
impl Drop for QuietTerminalStderr {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        if let Some(saved) = self.saved.take() {
            // SAFETY: saved owns the original stderr. Restore before joining so
            // later writes go directly to the terminal and the pipe reaches EOF.
            loop {
                if unsafe { libc::dup2(saved.as_raw_fd(), libc::STDERR_FILENO) } >= 0 {
                    break;
                }
                let error = std::io::Error::last_os_error();
                if error.kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // SAFETY: close the remaining writer to let the reader drain.
                unsafe {
                    libc::close(libc::STDERR_FILENO);
                }
                log::error!("could not restore graphics stderr: {error}");
                break;
            }
            if let Some(forwarder) = self.forwarder.take() {
                match forwarder.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(error)) => log::error!("graphics stderr forwarding failed: {error}"),
                    Err(_) => log::error!("graphics stderr forwarding thread panicked"),
                }
            }
        }
    }
}

fn run(shared: Arc<Shared>, options: GraphicsOptions) {
    let mut renderer = {
        #[cfg(unix)]
        let _quiet = QuietTerminalStderr::start();
        // PLT-016: use Mesa's own warning switch unless the host already set it.
        if std::env::var_os("MESA_VK_IGNORE_CONFORMANCE_WARNING").is_none() {
            std::env::set_var("MESA_VK_IGNORE_CONFORMANCE_WARNING", "true");
        }
        HybridRenderer::new(options)
    };
    {
        let mut slots = shared.slots();
        slots.mode = Some(renderer.mode().clone());
        slots.limits = Some(renderer.limits());
    }
    shared.started.notify_all();
    // A canvas that waited for the renderer's limits draws its first
    // picture now.
    shared.changed.update_atomic(|count| *count += 1);
    loop {
        let (slot, id, job) = {
            let mut slots = shared.slots();
            loop {
                if shared.closed.load(Ordering::Acquire) {
                    return;
                }
                if !slots.waiting.is_empty() {
                    // The oldest submission first: every picture gets its
                    // turn.
                    break slots.waiting.remove(0);
                }
                slots = shared.ready.wait(slots).unwrap_or_else(|e| e.into_inner());
            }
        };
        // Read before the scene's tokens are: a theme that changes while
        // the thread draws leaves a picture that names the older one.
        let theme = crate::theme::Theme::generation();
        let (frame, cells, error) = match job.want {
            Want::Pixels => match renderer.render_under(&job.scene, job.size, &job.base) {
                Ok(frame) => (Some(Arc::new(frame)), None, None),
                Err(error) => (None, None, Some(error.to_string())),
            },
            Want::Blocks { blitter, cell } => {
                let cells = (
                    job.size.0.min(u32::from(u16::MAX)) as u16,
                    job.size.1.min(u32::from(u16::MAX)) as u16,
                );
                match renderer.render_blocks(&job.scene, cells, cell, blitter, &job.base) {
                    Ok((grid, frame)) => (Some(Arc::new(frame)), Some(Arc::new(grid)), None),
                    Err(error) => (None, None, Some(error.to_string())),
                }
            }
        };
        let mut slots = shared.slots();
        slots.mode = Some(renderer.mode().clone());
        slots.limits = Some(renderer.limits());
        slots.stats.rendered += 1;
        // A picture whose canvas is gone is not kept.
        if let Some(served) = slots.served.get_mut(&slot) {
            served.finished = Some(Finished {
                id,
                size: job.size,
                want: job.want,
                frame,
                cells,
                error,
                theme,
            });
            served.taken = false;
        }
        drop(slots);
        if shared.closed.load(Ordering::Acquire) {
            return;
        }
        shared.changed.update_atomic(|count| *count += 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::scene::{Color, Paint, Path};
    use std::time::{Duration, Instant};

    fn worker() -> GraphicsWorker {
        GraphicsWorker::spawn(GraphicsOptions {
            force_cpu: true,
            font: crate::graphics::fonts::FontSource::Bundled,
            ..Default::default()
        })
        .expect("a worker")
    }

    fn wait_for(picture: &Picture, id: u64) -> Finished {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(finished) = picture.latest().filter(|finished| finished.id >= id) {
                return finished;
            }
            assert!(Instant::now() < deadline, "the thread finished nothing");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn a_failed_picture_is_finished_with_its_reason() {
        let worker = worker();
        let id = worker.submit_job(Job {
            scene: Arc::new(Scene::new()),
            size: (0, 10),
            want: Want::Pixels,
            base: Transform::identity(),
        });
        let finished = wait_for(&worker.own, id);
        assert!(finished.frame.is_none());
        assert!(finished.error.is_some_and(|error| error.contains("0x10")));
        assert!(worker.take_latest().is_none());
    }

    #[test]
    fn block_glyphs_come_with_the_picture_they_were_made_from() {
        let worker = worker();
        let mut scene = Scene::new();
        scene.fill(
            &Path::rect(0.0, 0.0, 800.0, 800.0),
            &Paint::solid(Color::rgba(200, 40, 40, 255)),
        );
        let id = worker.submit_job(Job {
            scene: Arc::new(scene),
            size: (10, 4),
            want: Want::Blocks {
                blitter: Blitter::Sextant,
                cell: (8, 16),
            },
            base: Transform::identity(),
        });
        let finished = wait_for(&worker.own, id);
        let grid = finished.cells.expect("block glyphs");
        assert_eq!((grid.width(), grid.height()), (10, 4));
        assert!(grid.background(9, 3).is_some() || grid.is_set(9, 3));
        let frame = finished.frame.expect("the picture");
        assert_eq!((frame.width(), frame.height()), (20, 12));
        assert_eq!(worker.mode(), Some(frame.mode().clone()));
    }

    /// Two pictures on one thread: each gets its own newest scene, and a
    /// scene of one never takes the other's place (GFX-003).
    #[test]
    fn gfx_003_pictures_on_one_thread_keep_their_own_scenes() {
        let worker = worker();
        let a = worker.picture();
        let b = worker.picture();
        let solid = |shade: u8| {
            let mut scene = Scene::new();
            scene.fill(
                &Path::rect(0.0, 0.0, 64.0, 64.0),
                &Paint::solid(Color::rgba(shade, 0, 0, 255)),
            );
            Arc::new(scene)
        };
        let job = |scene: Arc<Scene>| Job {
            scene,
            size: (8, 8),
            want: Want::Pixels,
            base: Transform::identity(),
        };
        let first_a = a.submit_job(job(solid(10)));
        let first_b = b.submit_job(job(solid(20)));
        let second_a = a.submit_job(job(solid(11)));
        let (last_a, last_b) = (wait_for(&a, second_a), wait_for(&b, first_b));
        let shade = |finished: &Finished| finished.frame.as_ref().unwrap().pixels()[0][0];
        assert!(
            shade(&last_a) == 11 && shade(&last_b) == 20 && last_a.id >= first_a,
            "GFX-003: picture a shows shade {} and picture b shade {}",
            shade(&last_a),
            shade(&last_b)
        );
        // A picture's slot goes with it.
        drop(b);
        assert_eq!(worker.shared.slots().served.len(), 2);
    }

    /// The process's shared thread is one per renderer, and ends with its
    /// last handle (GFX-003).
    #[test]
    fn gfx_003_the_shared_thread_is_one_per_renderer() {
        let options = GraphicsOptions {
            force_cpu: true,
            font: crate::graphics::fonts::FontSource::Bundled,
            fault: Some(GraphicsFault::Readback),
            ..Default::default()
        };
        let first = GraphicsWorker::shared(&options).expect("a thread");
        let again = GraphicsWorker::shared(&GraphicsOptions {
            output: Some(super::super::CanvasOutput::Blocks),
            ..options.clone()
        })
        .expect("the same thread");
        let other = GraphicsWorker::shared(&GraphicsOptions {
            fault: Some(GraphicsFault::Software),
            ..options.clone()
        })
        .expect("another thread");
        assert!(Arc::ptr_eq(&first, &again) && !Arc::ptr_eq(&first, &other));
        let weak = Arc::downgrade(&first);
        drop((first, again));
        assert!(
            weak.upgrade().is_none(),
            "the thread's handle outlived its holders"
        );
        let fresh = GraphicsWorker::shared(&options).expect("a new thread");
        assert!(!Arc::ptr_eq(&fresh, &other));
    }

    /// Two threads that quiet a terminal stderr at the same time take turns,
    /// so neither saves the other's /dev/null as the terminal: afterwards
    /// stderr is on the terminal it was on. A pseudo-terminal stands in for
    /// the terminal for the test's few milliseconds.
    /// PLT-016: a line another thread writes to stderr while graphics start
    /// reaches the terminal, in order with what follows.
    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn plt_016_stderr_written_while_graphics_start_reaches_the_terminal() {
        // SAFETY: descriptor calls on a pseudo-terminal this test opens and
        // closes, and on stderr, which it saves first and restores last.
        unsafe {
            let (mut master, mut slave) = (0, 0);
            assert_eq!(
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0,
                "a pseudo-terminal"
            );
            let flags = libc::fcntl(master, libc::F_GETFL);
            assert!(libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK) >= 0);
            let saved = libc::fcntl(libc::STDERR_FILENO, libc::F_DUPFD_CLOEXEC, 0);
            assert!(saved >= 0);
            assert_eq!(libc::dup2(slave, libc::STDERR_FILENO), libc::STDERR_FILENO);

            let guard = QuietTerminalStderr::start();
            std::thread::spawn(|| {
                let line = b"PLT016 first\n";
                libc::write(libc::STDERR_FILENO, line.as_ptr().cast(), line.len());
            })
            .join()
            .unwrap();
            drop(guard);
            let line = b"PLT016 second\n";
            libc::write(libc::STDERR_FILENO, line.as_ptr().cast(), line.len());

            let mut got = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(2);
            while Instant::now() < deadline && !String::from_utf8_lossy(&got).contains("second") {
                let mut buffer = [0u8; 256];
                let read = libc::read(master, buffer.as_mut_ptr().cast(), buffer.len());
                if read > 0 {
                    got.extend_from_slice(&buffer[..read as usize]);
                } else {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            libc::dup2(saved, libc::STDERR_FILENO);
            libc::close(saved);
            libc::close(slave);
            libc::close(master);

            let text = String::from_utf8_lossy(&got).into_owned();
            let first = text.find("PLT016 first");
            let second = text.find("PLT016 second");
            assert!(
                first.is_some() && second.is_some() && first < second,
                "PLT-016: the terminal received {text:?}: the line written to stderr while graphics started is missing or out of order"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn two_guards_at_once_leave_stderr_on_the_terminal() {
        // SAFETY: descriptor calls on a pseudo-terminal this test opens and
        // closes, and on stderr, which it saves first and restores last.
        unsafe {
            let (mut master, mut slave) = (0, 0);
            assert_eq!(
                // The termios and winsize pointers are `*mut` on macOS and
                // `*const` on Linux; a null `*mut` suits both.
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0,
                "a pseudo-terminal"
            );
            // The test swaps stderr in the guards' turn: a drawing thread
            // another test started may be making its renderer under a guard
            // right now, and would restore the stderr it saved over the
            // pseudo-terminal.
            let turn = QUIET_STDERR.lock().unwrap_or_else(|e| e.into_inner());
            let saved = libc::fcntl(libc::STDERR_FILENO, libc::F_DUPFD_CLOEXEC, 0);
            assert!(saved >= 0);
            assert_eq!(libc::dup2(slave, libc::STDERR_FILENO), libc::STDERR_FILENO);
            let file_of = |fd: libc::c_int| {
                let mut stat: libc::stat = std::mem::zeroed();
                assert_eq!(libc::fstat(fd, &mut stat), 0);
                (stat.st_dev, stat.st_ino)
            };
            let terminal = file_of(libc::STDERR_FILENO);
            assert_eq!(libc::isatty(libc::STDERR_FILENO), 1);
            drop(turn);
            let guards: Vec<_> = (0..2)
                .map(|_| {
                    std::thread::spawn(|| {
                        let quiet = QuietTerminalStderr::start();
                        std::thread::sleep(std::time::Duration::from_millis(30));
                        drop(quiet);
                    })
                })
                .collect();
            for guard in guards {
                guard.join().expect("a guard's thread");
            }
            let turn = QUIET_STDERR.lock().unwrap_or_else(|e| e.into_inner());
            let after = file_of(libc::STDERR_FILENO);
            libc::dup2(saved, libc::STDERR_FILENO);
            libc::close(saved);
            drop(turn);
            libc::close(slave);
            libc::close(master);
            assert_eq!(
                after, terminal,
                "stderr is on the terminal again after both guards, not on /dev/null"
            );
        }
    }
}

#[cfg(all(test, unix))]
mod stderr_tests {
    use super::QuietTerminalStderr;
    use std::io::Read;
    use std::os::fd::{AsRawFd, FromRawFd};

    // PLT-016: isolate descriptor 2 from other tests and exercise terminal forwarding.
    #[test]
    fn plt_016_startup_preserves_other_threads_stderr() {
        const CHILD: &str = "RTUI_PLT_016_STDERR_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "graphics::worker::stderr_tests::plt_016_startup_preserves_other_threads_stderr", "--nocapture"])
                .env(CHILD, "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            return;
        }
        // SAFETY: the child owns these descriptors and restores stderr before assertions.
        let (mut master, slave, saved) = unsafe {
            let (mut master, mut slave) = (-1, -1);
            assert_eq!(
                libc::openpty(
                    &mut master,
                    &mut slave,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut()
                ),
                0
            );
            let saved = libc::dup(libc::STDERR_FILENO);
            assert!(saved >= 0);
            assert_eq!(libc::dup2(slave, libc::STDERR_FILENO), libc::STDERR_FILENO);
            (
                std::fs::File::from_raw_fd(master),
                std::fs::File::from_raw_fd(slave),
                std::fs::File::from_raw_fd(saved),
            )
        };
        let guard = QuietTerminalStderr::start();
        let writer = std::thread::spawn(|| {
            let bytes = b"during-startup\nWARNING: radv is not a conformant Vulkan implementation, testing use only.\nordinary diagnostic\nradv: not a conformant Vulkan implementation diagnostic\nWARNING: unrelated warning\npartial-diagnostic";
            // SAFETY: descriptor 2 is the child's writable stderr.
            unsafe { libc::write(libc::STDERR_FILENO, bytes.as_ptr().cast(), bytes.len()) }
        });
        let written = writer.join().unwrap();
        drop(guard);
        // SAFETY: descriptor 2 is restored to the slave; saved owns the original stderr.
        unsafe {
            let after = b"after-startup\n";
            libc::write(libc::STDERR_FILENO, after.as_ptr().cast(), after.len());
            libc::dup2(saved.as_raw_fd(), libc::STDERR_FILENO);
            libc::fcntl(master.as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK);
        }
        let mut bytes = Vec::new();
        let result = master.read_to_end(&mut bytes);
        drop(slave);
        assert!(result.is_ok() || result.unwrap_err().kind() == std::io::ErrorKind::WouldBlock);
        assert!(written > 0);
        let text = String::from_utf8_lossy(&bytes);
        let during = text.find("during-startup").expect("marker during guard");
        let diagnostic = text
            .find("ordinary diagnostic")
            .expect("unrelated diagnostic");
        let after = text.find("after-startup").expect("marker after guard");
        let partial = text
            .find("partial-diagnostic")
            .expect("final bytes without a newline");
        assert!(
            during < diagnostic && diagnostic < partial && partial < after,
            "{text}"
        );
        assert!(
            text.contains("radv: not a conformant Vulkan implementation diagnostic"),
            "{text}"
        );
        assert!(text.contains("WARNING: unrelated warning"), "{text}");
        assert!(
            !text.contains("WARNING: radv is not a conformant Vulkan implementation"),
            "{text}"
        );
    }
}

#[cfg(all(test, unix))]
mod filter_tests {
    use super::driver_conformance_warning;

    /// PLT-016 (the adversary's finding 4): only the driver's own warning
    /// line is filtered, whichever Mesa driver prints it.
    #[test]
    fn plt_016_only_the_drivers_warning_line_is_filtered() {
        for line in [
            "WARNING: radv is not a conformant Vulkan implementation, testing use only.\n",
            "WARNING: lavapipe is not a conformant vulkan implementation, testing use only.\n",
        ] {
            assert!(
                driver_conformance_warning(line.as_bytes()),
                "PLT-016: the driver line {line:?} is not filtered"
            );
        }
        for line in [
            "WARNING: application could not suppress radv text 'not a conformant Vulkan implementation'\n",
            "WARNING: radv is not a conformant Vulkan implementation, testing use only. (seen twice)\n",
            "radv is not a conformant Vulkan implementation, testing use only.\n",
        ] {
            assert!(
                !driver_conformance_warning(line.as_bytes()),
                "PLT-016: the diagnostic {line:?} is filtered as the driver's line"
            );
        }
    }
}
