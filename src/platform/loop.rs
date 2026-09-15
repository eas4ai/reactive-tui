//! Event loop architecture for terminal input processing
//!
//! High-performance event loop with async support

use super::{parser::EscapeSequenceParser, TerminalEvent};
#[cfg(feature = "tokio")]
use crate::error::ReactiveError;
use crate::error::Result;
use std::io::{self, Read};
#[allow(unused_imports)]
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
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

/// Standard threaded event loop implementation
pub struct ThreadedEventLoop {
    /// Event queue for terminal events
    queue: Arc<Mutex<EventQueue<TerminalEvent>>>,

    /// Input thread handle
    thread: Option<JoinHandle<()>>,

    /// Flag to signal thread shutdown
    should_quit: Arc<AtomicBool>,

    /// Parser for escape sequences
    parser: Arc<Mutex<EscapeSequenceParser>>,
}

impl ThreadedEventLoop {
    /// Create new threaded event loop
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(EventQueue::new(512))),
            thread: None,
            should_quit: Arc::new(AtomicBool::new(false)),
            parser: Arc::new(Mutex::new(EscapeSequenceParser::new())),
        }
    }

    /// Start the input reading thread
    fn start_input_thread(&mut self) -> Result<()> {
        if self.thread.is_some() {
            return Ok(()); // Already started
        }

        let queue = Arc::clone(&self.queue);
        let should_quit = Arc::clone(&self.should_quit);
        let parser = Arc::clone(&self.parser);

        let handle = thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 1024];

            while !should_quit.load(Ordering::Acquire) {
                match stdin.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        // Parse input and generate events
                        if let Ok(mut parser) = parser.lock() {
                            let events = parser.parse(&buffer[..n]);

                            // Push events to queue
                            if let Ok(mut queue) = queue.lock() {
                                for event in events {
                                    queue.push(event);
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // No data available, sleep briefly
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break, // Other error, exit thread
                }
            }
        });

        self.thread = Some(handle);
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
        self.start_input_thread()
    }

    fn stop(&mut self) -> Result<()> {
        self.should_quit.store(true, Ordering::Release);

        if let Some(handle) = self.thread.take() {
            // Signal the thread to quit and wait for it
            let _ = handle.join();
        }

        Ok(())
    }

    fn next_event(&mut self) -> Option<TerminalEvent> {
        if let Ok(mut queue) = self.queue.lock() {
            if queue.is_empty() {
                None
            } else {
                Some(queue.pop())
            }
        } else {
            None
        }
    }

    fn try_event(&mut self) -> Option<TerminalEvent> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.try_pop()
        } else {
            None
        }
    }

    fn post_event(&mut self, event: TerminalEvent) -> Result<()> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(event);
            Ok(())
        } else {
            Err(std::io::Error::other("Failed to acquire queue lock").into())
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
    async fn start_input_task(&mut self) -> Result<()> {
        if self.input_task.is_some() {
            return Ok(()); // Already started
        }

        let sender = self
            .sender
            .as_ref()
            .ok_or_else(|| ReactiveError::internal("Sender not initialized"))?
            .clone();
        let parser = Arc::clone(&self.parser);
        let mut shutdown_rx = self
            .shutdown_rx
            .as_ref()
            .ok_or_else(|| ReactiveError::internal("Shutdown receiver not initialized"))?
            .clone();
        let is_running = Arc::clone(&self.is_running);

        let handle = tokio::spawn(async move {
            use tokio::io::{stdin, AsyncReadExt};

            let mut stdin = stdin();
            let mut buffer = [0u8; 1024];

            is_running.store(true, Ordering::Release);

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

            is_running.store(false, Ordering::Release);
        });

        self.input_task = Some(handle);
        Ok(())
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
        // For the sync trait, we need to use a runtime
        let rt = tokio::runtime::Handle::try_current().map_err(|_| {
            std::io::Error::other(
                "No tokio runtime found. Use start_async() instead or run within a tokio runtime.",
            )
        })?;

        rt.block_on(async { self.start_async().await })
    }

    fn stop(&mut self) -> Result<()> {
        // Signal shutdown
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(true);
        }

        // Wait for the task to complete if we have a runtime
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            if let Some(handle) = self.input_task.take() {
                rt.block_on(async {
                    let _ = handle.await;
                });
            }
        } else {
            // If no runtime, just take the handle and let it be dropped
            self.input_task.take();
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
        self.start_input_task().await
    }

    /// Async version of stop - preferred when using tokio
    pub async fn stop_async(&mut self) -> Result<()> {
        // Signal shutdown
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(true);
        }

        // Wait for the task to complete
        if let Some(handle) = self.input_task.take() {
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
                std::thread::sleep(Duration::from_millis(250));
                let deadline = Instant::now() + Duration::from_secs(2);
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
                assert!(started.elapsed() < Duration::from_secs(2));
            }
            "drop" => {
                drop(event_loop);
                let deadline = Instant::now() + Duration::from_secs(2);
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
