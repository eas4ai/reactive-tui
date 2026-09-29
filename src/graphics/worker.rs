//! The canvas's worker (GFX-003): one named thread that owns the renderer,
//! draws the newest scene it was given and tells the App through its waker
//! when a picture is finished. A scene submitted while it draws replaces
//! the one still waiting, and it waits for nothing between pictures.

use super::hybrid::{GraphicsMode, GraphicsOptions, HybridRenderer};
use super::scene::{Scene, Transform};
use super::{GraphicsError, GraphicsFrame};
use crate::layout::CellGrid;
use crate::reactive::ThreadSafeSignal;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use suprtui::blit::Blitter;

/// What the worker makes of a scene.
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

/// What the worker made of a job.
#[derive(Clone)]
pub(crate) struct Finished {
    /// The number [`GraphicsWorker::submit_job`] gave the job.
    #[cfg_attr(not(test), allow(dead_code))]
    pub id: u64,
    pub size: (u32, u32),
    pub want: Want,
    /// The picture; for block glyphs, the picture they were made from.
    pub frame: Option<Arc<GraphicsFrame>>,
    pub cells: Option<Arc<CellGrid>>,
    /// Why there is no picture: both renderers failed (GFX-007).
    pub error: Option<String>,
    /// [`crate::theme::Theme::generation`] when the worker began to draw:
    /// the theme whose colors the picture's tokens took (GFX-001).
    pub theme: u64,
}

/// What the worker has done so far.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkerStats {
    /// The most scenes that waited at once: never more than one.
    pub waiting_max: usize,
    /// Scenes replaced by a newer one before the worker drew them.
    pub replaced: u64,
    /// Pictures finished, failed ones included.
    pub rendered: u64,
}

#[derive(Default)]
struct Slots {
    waiting: Option<(u64, Job)>,
    finished: Option<Finished>,
    /// Whether [`GraphicsWorker::take_latest`] gave `finished` out.
    taken: bool,
    /// The renderer that draws, once the worker has made it.
    mode: Option<GraphicsMode>,
    next_id: u64,
    stats: WorkerStats,
}

struct Shared {
    slots: Mutex<Slots>,
    /// A job waits, or the worker is to end.
    ready: Condvar,
    /// The worker has made its renderer.
    started: Condvar,
    closed: AtomicBool,
    changed: ThreadSafeSignal<u64>,
}

impl Shared {
    /// The slots, also after a holder of the lock panicked: they hold no
    /// state a panic can leave half written.
    fn slots(&self) -> MutexGuard<'_, Slots> {
        self.slots.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// A worker thread named `rtui-canvas-*` that draws scenes.
pub struct GraphicsWorker {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for GraphicsWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slots = self.shared.slots();
        f.debug_struct("GraphicsWorker")
            .field("mode", &slots.mode)
            .field("stats", &slots.stats)
            .finish()
    }
}

static NEXT_NAME: AtomicUsize = AtomicUsize::new(0);

impl GraphicsWorker {
    /// Start a worker. It makes its renderer itself, so waiting for the
    /// adapter takes none of the caller's time.
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
        Ok(Self { shared })
    }

    /// Draw `scene` at `size` pixels, instead of any scene still waiting.
    /// Returns at once.
    pub fn submit(&self, scene: Arc<Scene>, size: (u32, u32)) {
        self.submit_job(Job {
            scene,
            size,
            want: Want::Pixels,
            base: Transform::identity(),
        });
    }

    /// Draw `job` instead of any still waiting; the number it finishes
    /// under.
    pub(crate) fn submit_job(&self, job: Job) -> u64 {
        let mut slots = self.shared.slots();
        slots.next_id += 1;
        let id = slots.next_id;
        if slots.waiting.replace((id, job)).is_some() {
            slots.stats.replaced += 1;
        }
        slots.stats.waiting_max = slots.stats.waiting_max.max(1);
        drop(slots);
        self.shared.ready.notify_one();
        id
    }

    /// The newest finished picture, once.
    pub fn take_latest(&self) -> Option<Arc<GraphicsFrame>> {
        let mut slots = self.shared.slots();
        if slots.taken {
            return None;
        }
        let frame = slots.finished.as_ref()?.frame.clone()?;
        slots.taken = true;
        Some(frame)
    }

    /// What the worker made of the newest job it finished.
    pub(crate) fn latest(&self) -> Option<Finished> {
        self.shared.slots().finished.clone()
    }

    /// Have the App that is rendering redraw when a picture is finished.
    pub(crate) fn observe(&self) {
        self.shared.changed.get();
    }

    /// The renderer that draws; `None` until the worker has made it.
    pub fn mode(&self) -> Option<GraphicsMode> {
        self.shared.slots().mode.clone()
    }

    /// Wait up to `timeout` for the worker to make its renderer, and return
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

    /// What the worker has done so far.
    pub fn stats(&self) -> WorkerStats {
        self.shared.slots().stats
    }
}

impl Drop for GraphicsWorker {
    /// The worker ends after the picture it is drawing. Nothing waits for
    /// it: the owner may be the App's thread (GFX-003).
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared.slots().waiting = None;
        self.shared.ready.notify_one();
    }
}

fn run(shared: Arc<Shared>, options: GraphicsOptions) {
    let mut renderer = HybridRenderer::new(options);
    shared.slots().mode = Some(renderer.mode().clone());
    shared.started.notify_all();
    loop {
        let (id, job) = {
            let mut slots = shared.slots();
            loop {
                if shared.closed.load(Ordering::Acquire) {
                    return;
                }
                if let Some(job) = slots.waiting.take() {
                    break job;
                }
                slots = shared.ready.wait(slots).unwrap_or_else(|e| e.into_inner());
            }
        };
        // Read before the scene's tokens are: a theme that changes while
        // the worker draws leaves a picture that names the older one.
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
        slots.stats.rendered += 1;
        slots.finished = Some(Finished {
            id,
            size: job.size,
            want: job.want,
            frame,
            cells,
            error,
            theme,
        });
        slots.taken = false;
        drop(slots);
        if shared.closed.load(Ordering::Acquire) {
            return;
        }
        shared.changed.set(id);
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

    fn wait_for(worker: &GraphicsWorker, id: u64) -> Finished {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(finished) = worker.latest().filter(|finished| finished.id >= id) {
                return finished;
            }
            assert!(Instant::now() < deadline, "the worker finished nothing");
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
        let finished = wait_for(&worker, id);
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
        let finished = wait_for(&worker, id);
        let grid = finished.cells.expect("block glyphs");
        assert_eq!((grid.width(), grid.height()), (10, 4));
        assert!(grid.background(9, 3).is_some() || grid.is_set(9, 3));
        let frame = finished.frame.expect("the picture");
        assert_eq!((frame.width(), frame.height()), (20, 12));
        assert_eq!(worker.mode(), Some(frame.mode().clone()));
    }
}
