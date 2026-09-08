//! Event router performance enhancements
//!
//! Zero-allocation event dispatch with compile-time optimizations

use super::router::{EventHandler, EventPhase, EventResult, NodeId};
use super::types::Event;
use lru::LruCache;
use std::cell::RefCell;
use std::num::NonZeroUsize;
use std::sync::Arc;

/// Event type discriminant for zero-cost dispatch
///
/// Uses Rust's discriminant mechanism to avoid string comparisons
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventDiscriminant {
    /// Keyboard input event
    Key = 0,
    /// Mouse interaction event
    Mouse = 1,
    /// Terminal resize event
    Resize = 2,
    /// Focus change event
    Focus = 3,
    /// Text paste event
    Paste = 4,
    /// Custom application event
    Custom = 5,
}

impl EventDiscriminant {
    /// Extract discriminant from event without allocation
    #[inline(always)]
    pub fn from_event(event: &Event) -> Self {
        match event {
            Event::Key(_) => Self::Key,
            Event::Mouse(_) => Self::Mouse,
            Event::Resize(_) => Self::Resize,
            Event::Focus(_) => Self::Focus,
            Event::Paste(_) => Self::Paste,
            Event::Custom(_) => Self::Custom,
        }
    }
}

/// Pre-sorted handler chain for efficient execution
pub struct HandlerChain {
    handlers: Vec<EventHandler>,
    needs_sort: bool,
}

impl Default for HandlerChain {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerChain {
    /// Create a new handler chain
    pub fn new() -> Self {
        Self {
            handlers: Vec::with_capacity(4), // Most nodes have few handlers
            needs_sort: false,
        }
    }

    /// Add a handler to the chain
    #[inline]
    pub fn add(&mut self, handler: EventHandler) {
        self.handlers.push(handler);
        self.needs_sort = self.handlers.len() > 1;
    }

    /// Execute all handlers in priority order
    pub fn execute(&mut self, event: &Event) -> EventResult {
        if self.needs_sort {
            self.handlers.sort_unstable_by_key(|h| -h.priority);
            self.needs_sort = false;
        }

        for handler in &self.handlers {
            match (handler.handler)(event) {
                EventResult::Consumed => return EventResult::Consumed,
                EventResult::Captured | EventResult::Handled | EventResult::Ignored => {}
            }
        }

        EventResult::Ignored
    }
}

/// Type alias for cache entry to reduce complexity
type CacheEntry = (NodeId, NodeId, Option<Arc<[NodeId]>>);
/// Type alias for cache key to reduce complexity
type CacheKey = (NodeId, NodeId);

/// Path cache using perfect hashing for common cases
pub struct PathCache {
    // Small inline cache for the most common paths (e.g., root -> immediate children)
    inline_cache: [CacheEntry; 8],
    // LRU cache for everything else
    lru: RefCell<LruCache<CacheKey, Arc<[NodeId]>>>,
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PathCache {
    /// Create a new path cache
    pub fn new() -> Self {
        const EMPTY: CacheEntry = (NodeId(0), NodeId(0), None);
        Self {
            inline_cache: [EMPTY; 8],
            lru: RefCell::new(LruCache::new(NonZeroUsize::new(128).unwrap())),
        }
    }

    /// Get a cached path between two nodes
    #[inline]
    pub fn get(&self, from: NodeId, to: NodeId) -> Option<Arc<[NodeId]>> {
        // Check inline cache first (no allocation, no locks)
        let hash = (from.0 ^ to.0) & 0x7;
        let entry = &self.inline_cache[hash];

        if entry.0 == from && entry.1 == to {
            return entry.2.clone();
        }

        // Fall back to LRU
        self.lru.borrow_mut().get(&(from, to)).cloned()
    }

    /// Insert a path into the cache
    pub fn insert(&mut self, from: NodeId, to: NodeId, path: Arc<[NodeId]>) {
        // Update inline cache
        let hash = (from.0 ^ to.0) & 0x7;
        self.inline_cache[hash] = (from, to, Some(path.clone()));

        // Also update LRU
        self.lru.borrow_mut().put((from, to), path);
    }

    /// Clear all cached paths
    pub fn clear(&mut self) {
        for entry in &mut self.inline_cache {
            entry.2 = None;
        }
        self.lru.borrow_mut().clear();
    }
}

/// Handler lookup table using dense indexing
pub struct HandlerLookup {
    // Dense array indexed by (node_id.0 % BUCKET_SIZE, event_discriminant, phase)
    // This gives O(1) lookup for most cases
    buckets: Vec<Option<Arc<HandlerChain>>>,
    bucket_size: usize,
}

impl Default for HandlerLookup {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerLookup {
    const BUCKET_SIZE: usize = 256; // Tune based on typical node count
    const EVENTS_COUNT: usize = 6;
    const PHASES_COUNT: usize = 3;

    /// Create a new handler lookup table
    pub fn new() -> Self {
        let total_buckets = Self::BUCKET_SIZE * Self::EVENTS_COUNT * Self::PHASES_COUNT;
        Self {
            buckets: vec![None; total_buckets],
            bucket_size: Self::BUCKET_SIZE,
        }
    }

    #[inline(always)]
    fn index(&self, node_id: NodeId, event: EventDiscriminant, phase: EventPhase) -> usize {
        let node_bucket = node_id.0 % self.bucket_size;
        let event_offset = event as usize;
        let phase_offset = match phase {
            EventPhase::Capture => 0,
            EventPhase::Target => 1,
            EventPhase::Bubble => 2,
        };

        node_bucket * Self::EVENTS_COUNT * Self::PHASES_COUNT
            + event_offset * Self::PHASES_COUNT
            + phase_offset
    }

    /// Get handler chain for a specific node, event type, and phase
    pub fn get(
        &self,
        node_id: NodeId,
        event: EventDiscriminant,
        phase: EventPhase,
    ) -> Option<&Arc<HandlerChain>> {
        let idx = self.index(node_id, event, phase);
        self.buckets[idx].as_ref()
    }

    /// Insert a handler chain for a specific node, event type, and phase
    pub fn insert(
        &mut self,
        node_id: NodeId,
        event: EventDiscriminant,
        phase: EventPhase,
        chain: Arc<HandlerChain>,
    ) {
        let idx = self.index(node_id, event, phase);
        self.buckets[idx] = Some(chain);
    }
}

/// Batch event processing for multiple events
pub struct EventBatch {
    events: Vec<(Event, NodeId)>,
    results: Vec<EventResult>,
}

impl EventBatch {
    /// Create a new event batch with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            results: Vec::with_capacity(capacity),
        }
    }

    /// Add an event to the batch
    #[inline]
    pub fn add(&mut self, event: Event, target: NodeId) {
        self.events.push((event, target));
    }

    /// Process all events in a single pass, maximizing cache locality
    pub fn execute<F>(&mut self, mut router: F) -> &[EventResult]
    where
        F: FnMut(&Event, NodeId) -> EventResult,
    {
        self.results.clear();
        self.results.reserve(self.events.len());

        for (event, target) in &self.events {
            self.results.push(router(event, *target));
        }

        &self.results
    }
}
