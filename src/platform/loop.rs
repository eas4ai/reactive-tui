//! Event loop architecture for terminal input processing
//!
//! High-performance event loop with async support

use super::{parser::EscapeSequenceParser, TerminalEvent};
#[cfg(feature = "tokio")]
use crate::error::ReactiveError;
use crate::error::Result;
#[allow(unused_imports)]
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[cfg(feature = "tokio")]
use tokio::sync::watch;

/// Event loop trait for different async backends
pub trait EventLoop<T> {
    /// Initialize the event loop
    fn init(&mut self) -> Result<()>;

    /// Start the event loop
    fn start(&mut self) -> Result<()>;

    /// Stop the event loop
    fn stop(&mut self) -> Result<()>;

    /// Get the next event (blocking)
    fn next_event(&mut self) -> Option<T>;

    /// Try to get an event (non-blocking)
    fn try_event(&mut self) -> Option<T>;

    /// Post an event to the queue
    fn post_event(&mut self, event: T) -> Result<()>;
}

/// Lock-free event queue with fixed capacity
pub struct EventQueue<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: Arc<std::sync::atomic::AtomicUsize>,
    tail: Arc<std::sync::atomic::AtomicUsize>,
    count: Arc<std::sync::atomic::AtomicUsize>,
}

impl<T> EventQueue<T> {
    /// Create new queue with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);

        Self {
            buffer,
            capacity,
            head: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            tail: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Push event to queue (blocking if full)
    pub fn push(&mut self, event: T) -> bool
    where
        T: Clone,
    {
        loop {
            if self.try_push(event.clone()) {
                return true;
            }
            // Yield to other threads
            std::thread::yield_now();
        }
    }

    /// Try to push event (non-blocking)
    pub fn try_push(&mut self, event: T) -> bool {
        let current_count = self.count.load(Ordering::Acquire);
        if current_count >= self.capacity {
            return false; // Queue is full
        }

        let tail = self.tail.load(Ordering::Acquire);
        let next_tail = (tail + 1) % self.capacity;

        // Try to reserve the slot
        if self
            .tail
            .compare_exchange_weak(tail, next_tail, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            // We got the slot, store the event
            unsafe {
                let slot = self.buffer.as_mut_ptr().add(tail);
                std::ptr::write(slot, Some(event));
            }
            self.count.fetch_add(1, Ordering::Release);
            true
        } else {
            false
        }
    }

    /// Pop event from queue (blocking)
    pub fn pop(&mut self) -> T {
        loop {
            if let Some(event) = self.try_pop() {
                return event;
            }
            // Wait a bit before retrying
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// Try to pop event (non-blocking)
    pub fn try_pop(&mut self) -> Option<T> {
        let current_count = self.count.load(Ordering::Acquire);
        if current_count == 0 {
            return None; // Queue is empty
        }

        let head = self.head.load(Ordering::Acquire);
        let next_head = (head + 1) % self.capacity;

        // Try to reserve the slot
        if self
            .head
            .compare_exchange_weak(head, next_head, Ordering::Release, Ordering::Relaxed)
            .is_ok()
        {
            // We got the slot, take the event
            let event = unsafe {
                let slot = self.buffer.as_mut_ptr().add(head);
                match std::ptr::replace(slot, None) {
                    Some(event) => event,
                    None => {
                        log::error!("Event queue corruption: expected event but found None");
                        return None;
                    }
                }
            };
            self.count.fetch_sub(1, Ordering::Release);
            Some(event)
        } else {
            None
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.count.load(Ordering::Acquire) == 0
    }

    /// Get current queue size
    pub fn len(&self) -> usize {
        self.count.load(Ordering::Acquire)
    }
}

struct ThreadedQueue<T> {
    state: Mutex<ThreadedQueueState<T>>,
    available: Condvar,
    space: Condvar,
}

struct ThreadedQueueState<T> {
    events: EventQueue<T>,
    stopped: bool,
}

impl<T> ThreadedQueue<T> {
    fn new(capacity: usize) -> Self {
        Self {
            state: Mutex::new(ThreadedQueueState {
                events: EventQueue::new(capacity),
                stopped: false,
            }),
            available: Condvar::new(),
            space: Condvar::new(),
        }
    }

    fn reset(&self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .stopped = false;
    }

    fn push(&self, event: T) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.stopped && state.events.len() == state.events.capacity {
            state = self
                .space
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        if state.stopped {
            return false;
        }
        let pushed = state.events.try_push(event);
        debug_assert!(pushed);
        self.available.notify_one();
        pushed
    }

    fn try_push(&self, event: T) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.stopped || !state.events.try_push(event) {
            return false;
        }
        self.available.notify_one();
        true
    }

    fn pop(&self) -> Option<T> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while !state.stopped && state.events.is_empty() {
            state = self
                .available
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
        let event = state.events.try_pop();
        if event.is_some() {
            self.space.notify_one();
        }
        event
    }

    fn try_pop(&self) -> Option<T> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let event = state.events.try_pop();
        if event.is_some() {
            self.space.notify_one();
        }
        event
    }

    fn stop(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.stopped = true;
        self.available.notify_all();
        self.space.notify_all();
    }
}

#[cfg(unix)]
fn spawn_unix_input(
    queue: Arc<ThreadedQueue<TerminalEvent>>,
    parser: Arc<Mutex<EscapeSequenceParser>>,
) -> Result<(JoinHandle<()>, Arc<super::input_receiver::Cancellation>)> {
    use std::os::fd::{FromRawFd, OwnedFd};
    let descriptor = unsafe {
        let fd = libc::fcntl(libc::STDIN_FILENO, libc::F_DUPFD_CLOEXEC, 0);
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        OwnedFd::from_raw_fd(fd)
    };
    let cancellation = Arc::new(super::input_receiver::Cancellation::new()?);
    let reader = Arc::clone(&cancellation);
    let handle = thread::Builder::new()
        .name("rtui-event-loop-input".into())
        .spawn(move || {
            let mut buffer = [0u8; 1024];
            while let Ok(count) = reader.read(&descriptor, &mut buffer) {
                if count == 0 {
                    break;
                }
                let events = parser
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .parse(&buffer[..count]);
                if events.into_iter().any(|event| !queue.push(event)) {
                    break;
                }
            }
        })?;
    Ok((handle, cancellation))
}

#[cfg(windows)]
fn spawn_windows_input(
    queue: Arc<ThreadedQueue<TerminalEvent>>,
    should_quit: Arc<AtomicBool>,
) -> Result<JoinHandle<()>> {
    let terminal = super::windows::WindowsTty::init()?;
    Ok(thread::Builder::new()
        .name("rtui-event-loop-input".into())
        .spawn(move || {
            while !should_quit.load(Ordering::Acquire) {
                match terminal.read_input_events(Some(Duration::from_millis(50))) {
                    Ok(events) => {
                        if events.into_iter().any(|event| !queue.push(event)) {
                            break;
                        }
                    }
                    Err(error) => {
                        log::error!("ThreadedEventLoop input failed: {error}");
                        break;
                    }
                }
            }
        })?)
}

/// Standard threaded event loop implementation
pub struct ThreadedEventLoop {
    /// Event queue for terminal events
    queue: Arc<ThreadedQueue<TerminalEvent>>,

    /// Input thread handle
    thread: Option<JoinHandle<()>>,

    /// Flag to signal thread shutdown
    should_quit: Arc<AtomicBool>,

    #[cfg(unix)]
    cancellation: Option<Arc<super::input_receiver::Cancellation>>,

    /// Parser for escape sequences
    parser: Arc<Mutex<EscapeSequenceParser>>,
}

impl ThreadedEventLoop {
    /// Create new threaded event loop
    pub fn new() -> Self {
        Self {
            queue: Arc::new(ThreadedQueue::new(512)),
            thread: None,
            should_quit: Arc::new(AtomicBool::new(false)),
            #[cfg(unix)]
            cancellation: None,
            parser: Arc::new(Mutex::new(EscapeSequenceParser::new())),
        }
    }

    /// Start the input reading thread
    fn start_input_thread(&mut self) -> Result<()> {
        if self.thread.is_some() {
            return Ok(()); // Already started
        }

        #[cfg(unix)]
        {
            let (handle, cancellation) =
                spawn_unix_input(Arc::clone(&self.queue), Arc::clone(&self.parser))?;
            self.thread = Some(handle);
            self.cancellation = Some(cancellation);
            Ok(())
        }

        #[cfg(windows)]
        {
            self.thread = Some(spawn_windows_input(
                Arc::clone(&self.queue),
                Arc::clone(&self.should_quit),
            )?);
            Ok(())
        }

        #[cfg(not(any(unix, windows)))]
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "ThreadedEventLoop input supports Unix and Windows",
        )
        .into())
    }

    fn shutdown(&mut self) -> Result<()> {
        self.should_quit.store(true, Ordering::Release);
        self.queue.stop();
        #[cfg(unix)]
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.cancel();
        }
        if let Some(handle) = self.thread.take() {
            handle
                .join()
                .map_err(|_| std::io::Error::other("ThreadedEventLoop input thread panicked"))?;
        }
        Ok(())
    }
}

impl EventLoop<TerminalEvent> for ThreadedEventLoop {
    fn init(&mut self) -> Result<()> {
        // Initialize any resources needed
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        self.should_quit.store(false, Ordering::Release);
        self.queue.reset();
        self.start_input_thread()
    }

    fn stop(&mut self) -> Result<()> {
        self.shutdown()
    }

    fn next_event(&mut self) -> Option<TerminalEvent> {
        self.queue.pop()
    }

    fn try_event(&mut self) -> Option<TerminalEvent> {
        self.queue.try_pop()
    }

    fn post_event(&mut self, event: TerminalEvent) -> Result<()> {
        if self.queue.try_push(event) {
            Ok(())
        } else {
            Err(std::io::Error::other("ThreadedEventLoop queue is full or stopped").into())
        }
    }
}

impl Drop for ThreadedEventLoop {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown() {
            log::debug!("ThreadedEventLoop shutdown after failure: {error}");
        }
    }
}

impl Default for ThreadedEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Async event loop using tokio
#[cfg(feature = "tokio")]
pub struct TokioEventLoop {
    /// Async channel sender for terminal events
    sender: Option<tokio::sync::mpsc::UnboundedSender<TerminalEvent>>,
    /// Async channel receiver for terminal events
    receiver: Option<tokio::sync::mpsc::UnboundedReceiver<TerminalEvent>>,
    /// Parser for escape sequences
    parser: Arc<Mutex<EscapeSequenceParser>>,
    /// Handle to the input reading task
    input_task: Option<tokio::task::JoinHandle<()>>,
    /// Shutdown signal sender
    shutdown_tx: Option<watch::Sender<bool>>,
    /// Shutdown signal receiver
    shutdown_rx: Option<watch::Receiver<bool>>,
    /// Flag to track if the loop is running
    is_running: Arc<AtomicBool>,
}

#[cfg(feature = "tokio")]
struct TokioRunningGuard(Arc<AtomicBool>);

#[cfg(feature = "tokio")]
impl Drop for TokioRunningGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

#[cfg(feature = "tokio")]
impl TokioEventLoop {
    /// Create a new tokio-based event loop
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        Self {
            sender: Some(sender),
            receiver: Some(receiver),
            parser: Arc::new(Mutex::new(EscapeSequenceParser::new())),
            input_task: None,
            shutdown_tx: Some(shutdown_tx),
            shutdown_rx: Some(shutdown_rx),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the async input reading task
    fn start_input_task(&mut self) -> Result<()> {
        if self.input_task.is_some() {
            return Ok(()); // Already started
        }

        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| ReactiveError::internal("Sender not initialized"))?
            .clone();
        let parser = Arc::clone(&self.parser);
        let shutdown_rx = self
            .shutdown_rx
            .as_ref()
            .ok_or_else(|| ReactiveError::internal("Shutdown receiver not initialized"))?
            .clone();
        #[cfg(not(unix))]
        let mut shutdown_rx = shutdown_rx;
        self.is_running.store(true, Ordering::Release);
        let running = TokioRunningGuard(Arc::clone(&self.is_running));

        #[cfg(unix)]
        let handle = Self::start_cancellable_unix_input(sender, parser, shutdown_rx, running)?;
        #[cfg(not(unix))]
        let handle = tokio::spawn(async move {
            use tokio::io::{stdin, AsyncReadExt};

            let _running = running;
            let mut stdin = stdin();
            let mut buffer = [0u8; 1024];

            loop {
                tokio::select! {
                    // Check for shutdown signal
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }

                    // Read from stdin
                    result = stdin.read(&mut buffer) => {
                        match result {
                            Ok(0) => break, // EOF
                            Ok(n) => {
                                // Parse input and generate events
                                if let Ok(mut parser) = parser.lock() {
                                    let events = parser.parse(&buffer[..n]);

                                    // Send events through the channel
                                    for event in events {
                                        if sender.send(event).is_err() {
                                            // Channel closed, exit
                                            break;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                // Send error event
                                let error_event = TerminalEvent::Error(format!("Input error: {}", e));
                                if sender.send(error_event).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        self.input_task = Some(handle);
        Ok(())
    }

    /// Unix input reader that stops promptly while blocked with no input.
    ///
    /// `tokio::io::stdin` parks a runtime blocking-pool thread in `read`
    /// until bytes or EOF arrive, so awaiting the reader is not enough to
    /// let the runtime shut down. This mirrors the threaded loop: a private
    /// stdin descriptor woken through `Cancellation`, read on a blocking
    /// thread the supervisor joins after signalling shutdown.
    #[cfg(unix)]
    fn start_cancellable_unix_input(
        sender: tokio::sync::mpsc::UnboundedSender<TerminalEvent>,
        parser: Arc<Mutex<EscapeSequenceParser>>,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
        running: TokioRunningGuard,
    ) -> Result<tokio::task::JoinHandle<()>> {
        use std::os::fd::{FromRawFd, OwnedFd};

        let descriptor = unsafe {
            let fd = libc::fcntl(libc::STDIN_FILENO, libc::F_DUPFD_CLOEXEC, 0);
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            OwnedFd::from_raw_fd(fd)
        };
        let flags = unsafe { libc::fcntl(descriptor.as_raw_fd(), libc::F_GETFL) };
        if flags < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        if unsafe {
            libc::fcntl(
                descriptor.as_raw_fd(),
                libc::F_SETFL,
                flags | libc::O_NONBLOCK,
            )
        } < 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        use std::os::fd::AsRawFd as _;
        let cancellation = Arc::new(super::input_receiver::Cancellation::new()?);
        let reader_cancellation = Arc::clone(&cancellation);
        let reader = tokio::task::spawn_blocking(move || {
            let mut buffer = [0u8; 1024];
            loop {
                match reader_cancellation.read(&descriptor, &mut buffer) {
                    Ok(0) => break, // EOF or cancellation wake
                    Ok(n) => {
                        // Parse input and generate events
                        if let Ok(mut parser) = parser.lock() {
                            let events = parser.parse(&buffer[..n]);

                            // Send events through the channel
                            let mut closed = false;
                            for event in events {
                                if sender.send(event).is_err() {
                                    // Channel closed, exit
                                    closed = true;
                                    break;
                                }
                            }
                            if closed {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        // Send error event
                        let error_event = TerminalEvent::Error(format!("Input error: {}", e));
                        if sender.send(error_event).is_err() {
                            break;
                        }
                        break;
                    }
                }
            }
        });

        Ok(tokio::spawn(async move {
            let _running = running;
            // Wait for the stop signal (or owner teardown), then wake the
            // blocked read so the joined reader cannot park a runtime thread.
            let _ = shutdown_rx.changed().await;
            cancellation.cancel();
            let _ = reader.await;
        }))
    }

    fn begin_shutdown(&mut self) -> Option<tokio::task::JoinHandle<()>> {
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(true);
        }
        self.input_task.take()
    }

    /// Check if the event loop is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Acquire)
    }

    /// Get the next event asynchronously
    pub async fn next_event_async(&mut self) -> Option<TerminalEvent> {
        if let Some(ref mut receiver) = self.receiver {
            receiver.recv().await
        } else {
            None
        }
    }

    /// Try to get an event without blocking
    pub fn try_event_async(&mut self) -> Option<TerminalEvent> {
        if let Some(ref mut receiver) = self.receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    /// Post an event asynchronously
    pub async fn post_event_async(&self, event: TerminalEvent) -> Result<()> {
        if let Some(ref sender) = self.sender {
            sender
                .send(event)
                .map_err(|_| std::io::Error::other("Failed to send event").into())
        } else {
            Err(std::io::Error::other("No sender available").into())
        }
    }
}

#[cfg(feature = "tokio")]
impl EventLoop<TerminalEvent> for TokioEventLoop {
    fn init(&mut self) -> Result<()> {
        // Initialize any resources needed
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        tokio::runtime::Handle::try_current().map_err(|_| {
            std::io::Error::other(
                "No tokio runtime found. Use start_async() instead or run within a tokio runtime.",
            )
        })?;
        self.start_input_task()
    }

    fn stop(&mut self) -> Result<()> {
        if self.input_task.is_some() {
            return Err(std::io::Error::other(
                "TokioEventLoop is running; use stop_async() to await input task shutdown",
            )
            .into());
        }
        Ok(())
    }

    fn next_event(&mut self) -> Option<TerminalEvent> {
        // For sync interface, try non-blocking first
        self.try_event()
    }

    fn try_event(&mut self) -> Option<TerminalEvent> {
        if let Some(ref mut receiver) = self.receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    fn post_event(&mut self, event: TerminalEvent) -> Result<()> {
        if let Some(ref sender) = self.sender {
            sender
                .send(event)
                .map_err(|_| std::io::Error::other("Failed to send event").into())
        } else {
            Err(std::io::Error::other("No sender available").into())
        }
    }
}

#[cfg(feature = "tokio")]
impl TokioEventLoop {
    /// Async version of start - preferred when using tokio
    pub async fn start_async(&mut self) -> Result<()> {
        self.start_input_task()
    }

    /// Async version of stop - preferred when using tokio
    pub async fn stop_async(&mut self) -> Result<()> {
        if let Some(handle) = self.begin_shutdown() {
            let _ = handle.await;
        }

        Ok(())
    }
}

#[cfg(feature = "tokio")]
impl Default for TokioEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Async event loop utilities
#[cfg(feature = "tokio")]
pub mod async_utils {
    use super::*;
    use tokio::time::{timeout, Duration};

    /// Create a new tokio event loop with timeout support
    pub fn new_with_timeout() -> TokioEventLoop {
        TokioEventLoop::new()
    }

    /// Wait for an event with a timeout
    pub async fn wait_for_event_timeout(
        event_loop: &mut TokioEventLoop,
        timeout_duration: Duration,
    ) -> Result<Option<TerminalEvent>> {
        match timeout(timeout_duration, event_loop.next_event_async()).await {
            Ok(event) => Ok(event),
            Err(_) => Ok(None), // Timeout occurred
        }
    }

    /// Collect multiple events within a time window
    pub async fn collect_events_batch(
        event_loop: &mut TokioEventLoop,
        max_events: usize,
        timeout_duration: Duration,
    ) -> Vec<TerminalEvent> {
        let mut events = Vec::with_capacity(max_events);
        let deadline = tokio::time::Instant::now() + timeout_duration;

        while events.len() < max_events && tokio::time::Instant::now() < deadline {
            let remaining_time = deadline - tokio::time::Instant::now();

            match timeout(remaining_time, event_loop.next_event_async()).await {
                Ok(Some(event)) => events.push(event),
                Ok(None) | Err(_) => break,
            }
        }

        events
    }

    /// Run an event loop with a custom handler
    pub async fn run_event_loop<F, Fut>(
        mut event_loop: TokioEventLoop,
        mut handler: F,
    ) -> Result<()>
    where
        F: FnMut(TerminalEvent) -> Fut,
        Fut: std::future::Future<Output = bool>, // Return false to stop the loop
    {
        event_loop.start_async().await?;

        while event_loop.is_running() {
            if let Some(event) = event_loop.next_event_async().await {
                if !handler(event).await {
                    break;
                }
            }
        }

        event_loop.stop_async().await?;
        Ok(())
    }
}

unsafe impl<T> Send for EventQueue<T> where T: Send {}
unsafe impl<T> Sync for EventQueue<T> where T: Send {}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "tokio")]
    use std::time::Duration;

    #[test]
    fn test_event_queue_basic_operations() {
        let mut queue = EventQueue::new(4);

        // Test empty queue
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        assert!(queue.try_pop().is_none());

        // Test push and pop
        assert!(queue.try_push(42));
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);

        assert_eq!(queue.try_pop(), Some(42));
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_event_queue_capacity() {
        let mut queue = EventQueue::new(2);

        // Fill the queue
        assert!(queue.try_push(1));
        assert!(queue.try_push(2));
        assert_eq!(queue.len(), 2);

        // Queue should be full
        assert!(!queue.try_push(3));
        assert_eq!(queue.len(), 2);

        // Pop one item
        assert_eq!(queue.try_pop(), Some(1));
        assert_eq!(queue.len(), 1);

        // Should be able to push again
        assert!(queue.try_push(3));
        assert_eq!(queue.len(), 2);

        // Verify order
        assert_eq!(queue.try_pop(), Some(2));
        assert_eq!(queue.try_pop(), Some(3));
        assert!(queue.is_empty());
    }

    #[test]
    fn test_threaded_event_loop_creation() {
        let mut event_loop = ThreadedEventLoop::new();

        // Test initialization
        assert!(event_loop.init().is_ok());

        // Test that we can post events
        let test_event = TerminalEvent::FocusGained;
        assert!(event_loop.post_event(test_event.clone()).is_ok());

        // Test that we can retrieve events
        if let Some(event) = event_loop.try_event() {
            assert_eq!(event, test_event);
        }
    }

    #[test]
    fn rtr_002_direct_post_reports_capacity_and_recovers_after_consumption() {
        let mut event_loop = ThreadedEventLoop::new();
        for _ in 0..512 {
            event_loop.post_event(TerminalEvent::FocusGained).unwrap();
        }
        let started = std::time::Instant::now();
        assert!(event_loop
            .post_event(TerminalEvent::FocusLost)
            .unwrap_err()
            .to_string()
            .contains("queue is full"));
        // The behavior under test: a post to a full queue fails at once
        // instead of blocking.
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert_eq!(event_loop.try_event(), Some(TerminalEvent::FocusGained));
        event_loop.post_event(TerminalEvent::FocusLost).unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "run in an isolated PTY by the RTR-002 mechanism"]
    fn rtr_002_threaded_event_loop_probe() {
        use std::io::Write;
        use std::time::{Duration, Instant};

        fn thread_count() -> usize {
            std::fs::read_dir("/proc/self/task").unwrap().count()
        }

        let mode = std::env::var("RTR002_MODE").expect("RTR002_MODE");
        let baseline_threads = thread_count();
        let mut event_loop = ThreadedEventLoop::new();
        event_loop.start().unwrap();
        println!("RTR002 READY {mode}");
        std::io::stdout().flush().unwrap();

        match mode.as_str() {
            "saturation" => {
                // Setup for the behavior under test: let the parent's input
                // fill the queue before draining it.
                std::thread::sleep(Duration::from_millis(250));
                // A hang guard, not a timing check.
                let deadline = Instant::now() + Duration::from_secs(30);
                for _ in 0..600 {
                    while event_loop.try_event().is_none() {
                        assert!(Instant::now() < deadline, "consumer made no progress");
                        std::thread::yield_now();
                    }
                }
                event_loop.stop().unwrap();
            }
            "idle-stop" => {
                let started = Instant::now();
                event_loop.stop().unwrap();
                // The behavior under test: stop ends an idle loop within its
                // 50 ms input poll.
                assert!(started.elapsed() < Duration::from_secs(2));
            }
            "drop" => {
                drop(event_loop);
                // A hang guard, not a timing check.
                let deadline = Instant::now() + Duration::from_secs(30);
                while thread_count() > baseline_threads && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(10));
                }
                assert_eq!(
                    thread_count(),
                    baseline_threads,
                    "input reader survived Drop"
                );
            }
            _ => panic!("unknown RTR002_MODE {mode}"),
        }
        println!("RTR002 PASS {mode}");
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_tokio_event_loop_creation() {
        let mut event_loop = TokioEventLoop::new();

        // Test initialization
        assert!(event_loop.init().is_ok());
        assert!(!event_loop.is_running());

        // Test that we can post events
        let test_event = TerminalEvent::FocusGained;
        assert!(event_loop
            .post_event_async(test_event.clone())
            .await
            .is_ok());

        // Test that we can retrieve events
        if let Some(event) = event_loop.try_event_async() {
            assert_eq!(event, test_event);
        }
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_tokio_event_loop_async_operations() {
        let mut event_loop = TokioEventLoop::new();

        // Test posting multiple events
        let events = vec![
            TerminalEvent::FocusGained,
            TerminalEvent::FocusLost,
            TerminalEvent::Resize {
                width: 80,
                height: 24,
            },
        ];

        for event in &events {
            assert!(event_loop.post_event_async(event.clone()).await.is_ok());
        }

        // Test retrieving events
        for expected_event in &events {
            if let Some(event) = event_loop.next_event_async().await {
                assert_eq!(&event, expected_event);
            } else {
                panic!("Expected event not received");
            }
        }
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_utils_timeout() {
        use super::async_utils::*;

        let mut event_loop = TokioEventLoop::new();

        // Test timeout with no events
        let result = wait_for_event_timeout(&mut event_loop, Duration::from_millis(10)).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Test timeout with event
        let test_event = TerminalEvent::FocusGained;
        assert!(event_loop
            .post_event_async(test_event.clone())
            .await
            .is_ok());

        let result = wait_for_event_timeout(&mut event_loop, Duration::from_millis(100)).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(test_event));
    }

    #[cfg(feature = "tokio")]
    #[tokio::test]
    async fn test_async_utils_batch_collection() {
        use super::async_utils::*;

        let mut event_loop = TokioEventLoop::new();

        // Post multiple events
        let events = vec![
            TerminalEvent::FocusGained,
            TerminalEvent::FocusLost,
            TerminalEvent::Resize {
                width: 80,
                height: 24,
            },
        ];

        for event in &events {
            assert!(event_loop.post_event_async(event.clone()).await.is_ok());
        }

        // Collect events in batch
        let collected = collect_events_batch(&mut event_loop, 5, Duration::from_millis(100)).await;
        assert_eq!(collected.len(), 3);
        assert_eq!(collected, events);
    }
}
