use super::{pixel_count, GraphicsError, GraphicsFrame};
use std::{
    sync::{Arc, Condvar, Mutex},
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
#[derive(Debug, Default, Clone, Copy)]
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

/// One owned render thread, one active render, and one replaceable pending slot.
/// Both requests and completed results are bounded independently.
pub struct GraphicsWorker {
    shared: Arc<Shared>,
    thread: Option<JoinHandle<()>>,
}

impl GraphicsWorker {
    /// Start a sleeping render worker. Rendering and notification run outside locks.
    pub fn spawn(
        mut render: impl FnMut(FrameRequest) -> Result<GraphicsFrame, GraphicsError> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let shared = Arc::new(Shared {
            mailbox: Mutex::new(Mailbox::default()),
            ready: Condvar::new(),
        });
        let owner = Arc::clone(&shared);
        let thread = thread::spawn(move || loop {
            let request = {
                let mut mailbox = owner.mailbox.lock().unwrap();
                while !mailbox.stopped && mailbox.pending.is_none() {
                    mailbox = owner.ready.wait(mailbox).unwrap();
                }
                if mailbox.stopped {
                    break;
                }
                let request = mailbox
                    .pending
                    .take()
                    .expect("pending request after condition wait");
                mailbox.stats.pending = 0;
                mailbox.stats.active = 1;
                request
            };
            let frame = render(request);
            let publish = {
                let mut mailbox = owner.mailbox.lock().unwrap();
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
        }
    }

    /// Replace the single pending request; false means shutdown has begun.
    pub fn request(&self, request: FrameRequest) -> bool {
        let mut mailbox = self.shared.mailbox.lock().unwrap();
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
    pub fn take_latest(&self) -> Option<WorkerOutput> {
        self.shared.mailbox.lock().unwrap().output.take()
    }

    /// Read queue bounds without waiting for rendering.
    pub fn stats(&self) -> WorkerStats {
        self.shared.mailbox.lock().unwrap().stats
    }

    /// Cancel pending work, discard results, and join the current render.
    /// Render implementations must honor their own bounded wait/cancellation.
    pub fn shutdown(&mut self) -> Result<(), GraphicsError> {
        {
            let mut mailbox = self.shared.mailbox.lock().unwrap();
            mailbox.stopped = true;
            mailbox.pending = None;
            mailbox.output = None;
            mailbox.stats.pending = 0;
            self.shared.ready.notify_one();
        }
        if let Some(thread) = self.thread.take() {
            thread
                .join()
                .map_err(|_| GraphicsError::Readback("graphics worker panicked".into()))?;
        }
        Ok(())
    }
}

impl Drop for GraphicsWorker {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            log::error!("{error}");
        }
    }
}
