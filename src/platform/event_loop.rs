//! Event loop architecture for terminal input processing
//!
//! Based on libvaxis Loop.zig with async support

use super::{parser::EscapeSequenceParser, TerminalEvent};
use crate::error::Result;
use std::io::{self, Read};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
                std::ptr::replace(slot, None).unwrap()
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
            Err(
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire queue lock")
                    .into(),
            )
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
    sender: Option<Sender<TerminalEvent>>,
    receiver: Option<Receiver<TerminalEvent>>,
    parser: EscapeSequenceParser,
}

#[cfg(feature = "tokio")]
impl TokioEventLoop {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender: Some(sender),
            receiver: Some(receiver),
            parser: EscapeSequenceParser::new(),
        }
    }
}

#[cfg(feature = "tokio")]
impl EventLoop<TerminalEvent> for TokioEventLoop {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        // Implementation would use tokio::spawn for async input reading
        todo!("Tokio event loop implementation")
    }

    fn stop(&mut self) -> Result<()> {
        todo!("Tokio event loop implementation")
    }

    fn next_event(&mut self) -> Option<TerminalEvent> {
        if let Some(ref receiver) = self.receiver {
            receiver.recv().ok()
        } else {
            None
        }
    }

    fn try_event(&mut self) -> Option<TerminalEvent> {
        if let Some(ref receiver) = self.receiver {
            receiver.try_recv().ok()
        } else {
            None
        }
    }

    fn post_event(&mut self, event: TerminalEvent) -> Result<()> {
        if let Some(ref sender) = self.sender {
            sender.send(event).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to send event").into()
            })
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "No sender available").into())
        }
    }
}

unsafe impl<T> Send for EventQueue<T> where T: Send {}
unsafe impl<T> Sync for EventQueue<T> where T: Send {}
