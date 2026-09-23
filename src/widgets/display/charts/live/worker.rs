//! The chart rasterizer thread (CHT-021): one named worker per chart, a
//! single replaceable pending job, and a snapshot slot the main thread
//! copies from. Finishing a picture sets a signal that asks the App to
//! redraw through its waker.

use super::super::ChartProps;
use super::canvas::{self, Picture};
use crate::reactive::ThreadSafeSignal;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// One rasterization request.
pub(super) struct Job {
    pub id: u64,
    pub props: Arc<ChartProps>,
    pub width: usize,
    pub height: usize,
    pub values: Arc<Vec<Vec<f64>>>,
    pub progress: f64,
}

#[derive(Default)]
struct Slots {
    request: Option<Job>,
    response: Option<(u64, Arc<Picture>)>,
}

struct Shared {
    slots: Mutex<Slots>,
    ready: Condvar,
    done: Condvar,
    closed: AtomicBool,
    changed: ThreadSafeSignal<u64>,
}

pub(super) struct Worker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

static NEXT_NAME: AtomicUsize = AtomicUsize::new(0);

impl Worker {
    pub fn new() -> std::io::Result<Self> {
        let shared = Arc::new(Shared {
            slots: Mutex::default(),
            ready: Condvar::new(),
            done: Condvar::new(),
            closed: AtomicBool::new(false),
            changed: ThreadSafeSignal::new(0),
        });
        let owner = shared.clone();
        let name = format!("rtui-chart-{}", NEXT_NAME.fetch_add(1, Ordering::Relaxed));
        let thread = thread::Builder::new()
            .name(name)
            .spawn(move || run(owner))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    /// Subscribe the rendering App to picture completion.
    pub fn observe(&self) {
        self.shared.changed.get();
    }

    /// Replace any pending job with `job`.
    pub fn submit(&self, job: Job) {
        let mut slots = self.shared.slots.lock().unwrap_or_else(|e| e.into_inner());
        slots.request = Some(job);
        drop(slots);
        self.shared.ready.notify_one();
    }

    /// The most recently finished picture.
    pub fn latest(&self) -> Option<(u64, Arc<Picture>)> {
        self.shared
            .slots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .response
            .clone()
    }

    /// Wait up to `timeout` for job `id` (or a later one) to finish.
    pub fn wait_for(&self, id: u64, timeout: Duration) -> Option<(u64, Arc<Picture>)> {
        let deadline = Instant::now() + timeout;
        let mut slots = self.shared.slots.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if let Some((done, picture)) = &slots.response {
                if *done >= id {
                    return Some((*done, picture.clone()));
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return slots.response.clone();
            }
            slots = self
                .shared
                .done
                .wait_timeout(slots, deadline - now)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.shared.closed.store(true, Ordering::Release);
        self.shared
            .slots
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .request = None;
        self.shared.ready.notify_one();
        if self
            .thread
            .take()
            .is_some_and(|thread| thread.join().is_err())
        {
            log::error!("Chart rasterizer panicked during shutdown");
        }
    }
}

fn run(shared: Arc<Shared>) {
    loop {
        let job = {
            let mut slots = shared.slots.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                if shared.closed.load(Ordering::Acquire) {
                    return;
                }
                if let Some(job) = slots.request.take() {
                    break job;
                }
                slots = shared.ready.wait(slots).unwrap_or_else(|e| e.into_inner());
            }
        };
        let diag = Instant::now();
        let picture = canvas::draw(&canvas::Job {
            props: &job.props,
            width: job.width,
            height: job.height,
            values: &job.values,
            progress: job.progress,
            unicode_glyphs: crate::widgets::display::charts::glyph_support(),
        });
        eprintln!("DIAG raster {}x{} {:.2}", job.width, job.height, diag.elapsed().as_secs_f64() * 1000.0);
        let mut slots = shared.slots.lock().unwrap_or_else(|e| e.into_inner());
        slots.response = Some((job.id, Arc::new(picture)));
        drop(slots);
        shared.done.notify_all();
        shared.changed.set(job.id);
    }
}
