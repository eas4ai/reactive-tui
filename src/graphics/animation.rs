use super::{pixel_count, GraphicsError, GraphicsFrame};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

/// The bounded animation cadence: at most twenty scheduled requests per second.
pub const FRAME_INTERVAL: Duration = Duration::from_millis(50);

/// A validated viewport and elapsed animation time, not a frame counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameRequest {
    columns: u32,
    rows: u32,
    elapsed: Duration,
}

impl FrameRequest {
    /// Reject invalid dimensions before queueing or allocating anything.
    pub fn new(columns: u32, rows: u32, elapsed: Duration) -> Result<Self, GraphicsError> {
        let height = rows
            .checked_mul(2)
            .ok_or_else(|| GraphicsError::Dimensions("row overflow".into()))?;
        pixel_count(columns, height)?;
        Ok(Self {
            columns,
            rows,
            elapsed,
        })
    }
    /// Terminal columns.
    pub fn columns(self) -> u32 {
        self.columns
    }
    /// Terminal rows.
    pub fn rows(self) -> u32 {
        self.rows
    }
    /// Elapsed animation time supplied by the clock.
    pub fn elapsed(self) -> Duration {
        self.elapsed
    }
}

/// Deadline-based scheduling that skips missed frames rather than replaying them.
#[derive(Debug)]
pub struct FrameClock {
    next_deadline: Option<Duration>,
}

impl Default for FrameClock {
    fn default() -> Self {
        Self {
            next_deadline: Some(Duration::ZERO),
        }
    }
}

impl FrameClock {
    /// Submit at most one request per deadline using the current elapsed time.
    /// Invalid viewport requests fail even when the deadline has not arrived.
    pub fn request_at(
        &mut self,
        elapsed: Duration,
        columns: u32,
        rows: u32,
    ) -> Result<Option<FrameRequest>, GraphicsError> {
        let request = FrameRequest::new(columns, rows, elapsed)?;
        let Some(deadline) = self.next_deadline else {
            return Ok(None);
        };
        if elapsed < deadline {
            return Ok(None);
        }
        self.next_deadline = elapsed.checked_add(FRAME_INTERVAL);
        Ok(Some(request))
    }
}

/// A completed frame with its viewport/time identity, including failures.
pub struct WorkerOutput {
    /// The request that produced this result.
    pub request: FrameRequest,
    /// A readback frame or an actionable error.
    pub frame: Result<GraphicsFrame, GraphicsError>,
}

/// Current queue bounds and cumulative worker progress.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkerStats {
    /// Zero or one render in progress.
    pub active: usize,
    /// Zero or one replaceable request waiting.
    pub pending: usize,
    /// Number of requests replaced before they started.
    pub replaced: u64,
    /// Number of completed renders, including failures.
    pub completed: u64,
}

#[derive(Default)]
struct Mailbox {
    pending: Option<FrameRequest>,
    output: Option<WorkerOutput>,
    stopped: bool,
    stats: WorkerStats,
}

struct Shared {
    mailbox: Mutex<Mailbox>,
    ready: Condvar,
}

/// Shared cancellation observed by initialization, readback, and CPU work.
#[derive(Clone, Default)]
pub struct GraphicsCancellation(Arc<AtomicBool>);
impl GraphicsCancellation {
    /// Whether the owning canvas has begun shutdown.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    /// Stop future work; the owner still joins its worker.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

/// One owned render thread, one active render, and one replaceable pending slot.
/// Both requests and completed results are bounded independently.
pub struct GraphicsWorker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
    cancellation: GraphicsCancellation,
}

impl GraphicsWorker {
    /// Start a sleeping render worker. Rendering and notification run outside locks.
    pub fn spawn(
        mut render: impl FnMut(FrameRequest) -> Result<GraphicsFrame, GraphicsError> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        Self::spawn_cancellable(move |request, _| render(request), notify)
    }

    /// Start a worker whose active render can observe shutdown cancellation.
    pub fn spawn_cancellable(
        mut render: impl FnMut(FrameRequest, &GraphicsCancellation) -> Result<GraphicsFrame, GraphicsError>
            + Send
            + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let shared = Arc::new(Shared {
            mailbox: Mutex::new(Mailbox::default()),
            ready: Condvar::new(),
        });
        let owner = Arc::clone(&shared);
        let cancellation = GraphicsCancellation::default();
        let worker_cancellation = cancellation.clone();
        let thread = thread::spawn(move || 'worker: loop {
            let request = {
                let mut mailbox = match owner.mailbox.lock() {
                    Ok(mailbox) => mailbox,
                    // Poisoned by a panicked holder: queue state is
                    // untrustworthy, so the worker exits instead of
                    // panicking with it.
                    Err(_) => break 'worker,
                };
                while !mailbox.stopped && mailbox.pending.is_none() {
                    mailbox = match owner.ready.wait(mailbox) {
                        Ok(mailbox) => mailbox,
                        // Poisoned while parked: wait() consumed the
                        // guard with it, so there is nothing to
                        // re-check. Exit the worker outright.
                        Err(_) => break 'worker,
                    };
                }
                if mailbox.stopped {
                    break 'worker;
                }
                let request = match mailbox.pending.take() {
                    Some(request) => request,
                    // Unreachable: the wait loop above only exits with a
                    // pending request or a stop, both checked. Loop back
                    // to waiting rather than panic on the impossible.
                    None => continue,
                };
                mailbox.stats.pending = 0;
                mailbox.stats.active = 1;
                request
            };
            let frame = render(request, &worker_cancellation);
            let publish = {
                let mut mailbox = match owner.mailbox.lock() {
                    Ok(mailbox) => mailbox,
                    Err(_) => break,
                };
                mailbox.stats.active = 0;
                mailbox.stats.completed = mailbox.stats.completed.saturating_add(1);
                if mailbox.stopped {
                    false
                } else {
                    mailbox.output = Some(WorkerOutput { request, frame });
                    true
                }
            };
            if publish {
                notify();
            }
        });
        Self {
            shared,
            thread: Some(thread),
            cancellation,
        }
    }

    /// Replace the single pending request; false means shutdown has begun
    /// or the queue was poisoned by a panicked holder.
    pub fn request(&self, request: FrameRequest) -> bool {
        let mut mailbox = match self.shared.mailbox.lock() {
            Ok(mailbox) => mailbox,
            Err(_) => return false,
        };
        if mailbox.stopped {
            return false;
        }
        if mailbox.pending.replace(request).is_some() {
            mailbox.stats.replaced = mailbox.stats.replaced.saturating_add(1);
        }
        mailbox.stats.pending = 1;
        self.shared.ready.notify_one();
        true
    }

    /// Take the newest result, discarding older results when rendering outruns UI.
    /// A poisoned queue yields no result instead of panicking.
    pub fn take_latest(&self) -> Option<WorkerOutput> {
        self.shared
            .mailbox
            .lock()
            .map(|mut mailbox| mailbox.output.take())
            .unwrap_or(None)
    }

    /// Read queue bounds without waiting for rendering.
    /// A poisoned queue reports zeroed stats instead of panicking.
    pub fn stats(&self) -> WorkerStats {
        self.shared
            .mailbox
            .lock()
            .map(|mailbox| mailbox.stats)
            .unwrap_or_default()
    }

    /// Cancel pending work, discard results, and join the current render.
    /// Render implementations must honor their own bounded wait/cancellation.
    pub fn shutdown(&mut self) -> Result<(), GraphicsError> {
        self.cancel();
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| GraphicsError::Readback("graphics worker panicked".into()))?;
        }
        Ok(())
    }

    /// Cancel active/pending work immediately, without detaching the thread.
    pub fn cancel(&self) {
        self.cancellation.cancel();
        if let Ok(mut mailbox) = self.shared.mailbox.lock() {
            mailbox.stopped = true;
            mailbox.pending = None;
            mailbox.output = None;
            mailbox.stats.pending = 0;
            self.shared.ready.notify_one();
        }
    }
}

impl Drop for GraphicsWorker {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            log::error!("{error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poisoned_worker() -> GraphicsWorker {
        let shared = Arc::new(Shared {
            mailbox: Mutex::new(Mailbox::default()),
            ready: Condvar::new(),
        });
        let victim = Arc::clone(&shared);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = victim.mailbox.lock().unwrap();
            panic!("poison the mailbox");
        }));
        assert!(shared.mailbox.is_poisoned());
        GraphicsWorker {
            shared,
            thread: None,
            cancellation: GraphicsCancellation::default(),
        }
    }

    #[test]
    fn poisoned_mailbox_degrades_instead_of_panicking() {
        let worker = poisoned_worker();
        let request = FrameRequest::new(80, 24, Duration::from_millis(16)).expect("valid request");
        assert!(!worker.request(request));
        assert!(worker.take_latest().is_none());
        assert_eq!(worker.stats(), WorkerStats::default());
        worker.cancel();
    }
}
