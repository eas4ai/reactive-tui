//! Caller-owned grapheme-cluster pool, ported from the reference `grapheme.zig`.
//!
//! The reference keeps one process-global pool; this port excludes it by
//! design (UNI-008 forbids process-global mutable state), so every pool is
//! an explicit caller-owned value. Two deliberate structural deviations
//! keep the crate `unsafe`-free:
//!
//! - Unowned slots store a shared borrow (`&'a [u8]`) instead of a raw
//!   pointer, so `get` still aliases the caller's memory (pointer-identity
//!   holds) with the lifetime checked by the compiler. The pool is
//!   therefore lifetime-parameterized: `GraphemePool<'a>`.
//! - `GraphemeTracker` shares its pool through `Rc<RefCell<..>>` so two
//!   trackers can reference one pool at the same time, exactly as the
//!   reference does with raw pointers.
//!
//! Everything else mirrors the reference: five size classes (8/16/32/64/
//! 128 bytes), page growth with a LIFO free list, 7-bit wrapping
//! generations, 26-bit id payloads `[class 3 | generation 7 | slot 16]`,
//! live-id interning with stale-entry invalidation, and owned (128-byte)
//! versus unowned (`u16`-length) bounds.

use super::segments::GRAPHEME_ID_MASK;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// Pool id payload: `[class (3 bits) | generation (7 bits) | slot (16 bits)]`.
pub type IdPayload = u32;

pub const CLASS_BITS: u32 = 3;
pub const GENERATION_BITS: u32 = 7;
pub const SLOT_BITS: u32 = 16;
pub const CLASS_MASK: u32 = (1 << CLASS_BITS) - 1;
pub const GENERATION_MASK: u32 = (1 << GENERATION_BITS) - 1;
pub const SLOT_MASK: u32 = (1 << SLOT_BITS) - 1;

const MAX_CLASSES: usize = 5;
const CLASS_SIZES: [u32; MAX_CLASSES] = [8, 16, 32, 64, 128];
const DEFAULT_SLOTS_PER_PAGE: [u32; MAX_CLASSES] = [256, 128, 64, 16, 8];

/// Reference `GraphemePoolError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphemePoolError {
    OutOfMemory,
    GraphemeTooLong,
    InvalidId,
    WrongGeneration,
}

impl fmt::Display for GraphemePoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphemePoolError::OutOfMemory => write!(f, "grapheme pool out of memory"),
            GraphemePoolError::GraphemeTooLong => write!(f, "grapheme exceeds storage bound"),
            GraphemePoolError::InvalidId => write!(f, "invalid grapheme id"),
            GraphemePoolError::WrongGeneration => write!(f, "stale grapheme id generation"),
        }
    }
}

impl std::error::Error for GraphemePoolError {}

/// Reference `GraphemePool.InitOptions`.
#[derive(Clone, Copy, Debug, Default)]
pub struct InitOptions {
    /// Slots per page per size class; `None` selects the defaults.
    /// Used to limit pool size for testing. Entries must be non-zero.
    pub slots_per_page: Option<[u32; MAX_CLASSES]>,
}

#[derive(Debug)]
enum SlotData<'a> {
    Empty,
    Owned(Vec<u8>),
    Unowned(&'a [u8]),
}

#[derive(Debug)]
struct Slot<'a> {
    len: u16,
    refcount: u32,
    generation: u32,
    owned: bool,
    allocated: bool,
    data: SlotData<'a>,
}

impl<'a> Slot<'a> {
    fn fresh() -> Self {
        Slot {
            len: 0,
            refcount: 0,
            generation: 0,
            owned: false,
            allocated: false,
            data: SlotData::Empty,
        }
    }

    fn bytes(&self) -> &[u8] {
        match &self.data {
            SlotData::Owned(v) => v.as_slice(),
            SlotData::Unowned(s) => s,
            SlotData::Empty => &[],
        }
    }
}

#[derive(Debug)]
struct ClassPool<'a> {
    slot_capacity: u32,
    slots_per_page: u32,
    slots: Vec<Slot<'a>>,
    free_list: Vec<u32>,
}

impl<'a> ClassPool<'a> {
    fn new(slot_capacity: u32, slots_per_page: u32) -> Self {
        debug_assert!(slot_capacity > 0);
        debug_assert!(slots_per_page > 0);
        ClassPool {
            slot_capacity,
            slots_per_page,
            slots: Vec::new(),
            free_list: Vec::new(),
        }
    }

    fn num_slots(&self) -> u32 {
        self.slots.len() as u32
    }

    fn grow(&mut self) -> Result<(), GraphemePoolError> {
        if self.slots_per_page > SLOT_MASK + 1 {
            return Err(GraphemePoolError::OutOfMemory);
        }
        if self.num_slots() > SLOT_MASK + 1 - self.slots_per_page {
            return Err(GraphemePoolError::OutOfMemory);
        }
        let base = self.num_slots();
        self.slots
            .extend((0..self.slots_per_page).map(|_| Slot::fresh()));
        self.free_list.extend(base..base + self.slots_per_page);
        Ok(())
    }

    /// Pop a free slot, growing the page when empty. Shared by both paths.
    fn take_slot(&mut self) -> Result<u32, GraphemePoolError> {
        if self.free_list.is_empty() {
            self.grow()?;
        }
        let slot_index = self.free_list.pop().expect("grow guarantees a free slot");
        debug_assert!(slot_index < self.num_slots());
        Ok(slot_index)
    }

    /// Open a slot header for (re)use. Generation advances on every use,
    /// wrapping at 7 bits, exactly like the reference.
    fn open_slot(&mut self, slot_index: u32, len: u16, owned: bool) {
        let slot = &mut self.slots[slot_index as usize];
        debug_assert!(slot.refcount == 0);
        debug_assert!(!slot.allocated);
        slot.len = len;
        slot.refcount = 0;
        slot.generation = (slot.generation + 1) & GENERATION_MASK;
        slot.owned = owned;
        slot.allocated = true;
        debug_assert!(slot.generation <= GENERATION_MASK);
    }

    fn alloc_owned(&mut self, bytes: &[u8]) -> Result<u32, GraphemePoolError> {
        if bytes.len() as u64 > self.slot_capacity as u64 {
            return Err(GraphemePoolError::GraphemeTooLong);
        }
        let slot_index = self.take_slot()?;
        self.open_slot(slot_index, bytes.len() as u16, true);
        self.slots[slot_index as usize].data = SlotData::Owned(bytes.to_vec());
        Ok(slot_index)
    }

    fn alloc_shared(&mut self, bytes: &'a [u8]) -> Result<u32, GraphemePoolError> {
        if bytes.len() > u16::MAX as usize {
            return Err(GraphemePoolError::GraphemeTooLong);
        }
        let slot_index = self.take_slot()?;
        self.open_slot(slot_index, bytes.len() as u16, false);
        // The borrow `'a` keeps the alias checked; no raw pointer involved.
        self.slots[slot_index as usize].data = SlotData::Unowned(bytes);
        Ok(slot_index)
    }

    fn check(
        &self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<&Slot<'a>, GraphemePoolError> {
        if slot_index >= self.num_slots() {
            return Err(GraphemePoolError::InvalidId);
        }
        let slot = &self.slots[slot_index as usize];
        if slot.generation != expected_generation {
            return Err(GraphemePoolError::WrongGeneration);
        }
        if !slot.allocated {
            return Err(GraphemePoolError::InvalidId);
        }
        Ok(slot)
    }

    fn generation(&self, slot_index: u32) -> u32 {
        debug_assert!(slot_index < self.num_slots());
        self.slots[slot_index as usize].generation
    }

    fn incref(
        &mut self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<(), GraphemePoolError> {
        let slot = self.check(slot_index, expected_generation)?;
        debug_assert!(slot.refcount < u32::MAX);
        let slot = &mut self.slots[slot_index as usize];
        slot.refcount = slot.refcount.wrapping_add(1);
        debug_assert!(slot.refcount > 0);
        Ok(())
    }

    fn decref(
        &mut self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<(), GraphemePoolError> {
        let slot = self.check(slot_index, expected_generation)?;
        if slot.refcount == 0 {
            return Err(GraphemePoolError::InvalidId);
        }
        let slot = &mut self.slots[slot_index as usize];
        slot.refcount -= 1;
        if slot.refcount == 0 {
            slot.allocated = false;
            self.free_list.push(slot_index);
        }
        Ok(())
    }

    fn free_unreferenced(
        &mut self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<(), GraphemePoolError> {
        let slot = self.check(slot_index, expected_generation)?;
        if slot.refcount != 0 {
            return Err(GraphemePoolError::InvalidId);
        }
        self.slots[slot_index as usize].allocated = false;
        self.free_list.push(slot_index);
        Ok(())
    }

    fn get(&self, slot_index: u32, expected_generation: u32) -> Result<&[u8], GraphemePoolError> {
        let slot = self.check(slot_index, expected_generation)?;
        if slot.owned {
            debug_assert!((slot.len as u32) <= self.slot_capacity);
        }
        Ok(slot.bytes())
    }

    fn get_refcount(
        &self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<u32, GraphemePoolError> {
        Ok(self.check(slot_index, expected_generation)?.refcount)
    }

    fn is_owned(
        &self,
        slot_index: u32,
        expected_generation: u32,
    ) -> Result<bool, GraphemePoolError> {
        Ok(self.check(slot_index, expected_generation)?.owned)
    }
}

fn class_for_size(size: usize) -> usize {
    if size <= 8 {
        0
    } else if size <= 16 {
        1
    } else if size <= 32 {
        2
    } else if size <= 64 {
        3
    } else {
        4
    }
}

fn pack_id(
    class_id: u32,
    slot_index: u32,
    generation: u32,
) -> Result<IdPayload, GraphemePoolError> {
    debug_assert!(class_id < MAX_CLASSES as u32);
    debug_assert!(generation <= GENERATION_MASK);
    if slot_index > SLOT_MASK {
        return Err(GraphemePoolError::OutOfMemory);
    }
    let id = (class_id << (GENERATION_BITS + SLOT_BITS))
        | ((generation & GENERATION_MASK) << SLOT_BITS)
        | (slot_index & SLOT_MASK);
    debug_assert!(id & !GRAPHEME_ID_MASK == 0);
    Ok(id)
}

fn decode_id(id: IdPayload) -> Result<(usize, u32, u32), GraphemePoolError> {
    let class_id = ((id >> (GENERATION_BITS + SLOT_BITS)) & CLASS_MASK) as usize;
    if class_id >= MAX_CLASSES {
        return Err(GraphemePoolError::InvalidId);
    }
    let slot_index = id & SLOT_MASK;
    let generation = (id >> SLOT_BITS) & GENERATION_MASK;
    Ok((class_id, slot_index, generation))
}

/// Caller-owned slab pool for grapheme-cluster bytes. Reference `GraphemePool`.
#[derive(Debug)]
pub struct GraphemePool<'a> {
    classes: [ClassPool<'a>; MAX_CLASSES],
    interned_live_ids: HashMap<Vec<u8>, IdPayload>,
}

impl<'a> Default for GraphemePool<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> GraphemePool<'a> {
    pub fn new() -> Self {
        Self::with_options(InitOptions::default())
    }

    pub fn with_options(options: InitOptions) -> Self {
        let slots_per_page = options.slots_per_page.unwrap_or(DEFAULT_SLOTS_PER_PAGE);
        let classes = std::array::from_fn(|i| {
            if slots_per_page[i] == 0 {
                panic!("GraphemePool: slots_per_page must be non-zero");
            }
            ClassPool::new(CLASS_SIZES[i], slots_per_page[i])
        });
        GraphemePool {
            classes,
            interned_live_ids: HashMap::new(),
        }
    }

    fn remove_interned_live_id(&mut self, bytes: &[u8], expected_id: IdPayload) {
        if self.interned_live_ids.get(bytes) != Some(&expected_id) {
            return;
        }
        self.interned_live_ids.remove(bytes);
    }

    fn lookup_or_invalidate(&mut self, bytes: &[u8]) -> Option<IdPayload> {
        let live_id = *self.interned_live_ids.get(bytes)?;
        let live_bytes = self.get(live_id).ok()?;
        if live_bytes != bytes {
            self.remove_interned_live_id(bytes, live_id);
            return None;
        }
        let live_refcount = self.get_refcount(live_id).ok()?;
        if live_refcount == 0 {
            self.remove_interned_live_id(bytes, live_id);
            return None;
        }
        Some(live_id)
    }

    fn intern_live_id(&mut self, id: IdPayload, bytes: &[u8]) -> Result<(), GraphemePoolError> {
        if self.lookup_or_invalidate(bytes).is_some() {
            return Ok(());
        }
        // HashMap insert can only fail by aborting on OOM; the reference
        // maps that to OutOfMemory, which has no Rust equivalent here.
        self.interned_live_ids.insert(bytes.to_vec(), id);
        Ok(())
    }

    /// Copy `bytes` into the pool (interned while live). Reference `alloc`.
    pub fn alloc(&mut self, bytes: &[u8]) -> Result<IdPayload, GraphemePoolError> {
        if bytes.len() > CLASS_SIZES[MAX_CLASSES - 1] as usize {
            return Err(GraphemePoolError::GraphemeTooLong);
        }
        if let Some(live_id) = self.lookup_or_invalidate(bytes) {
            return Ok(live_id);
        }
        let class_id = class_for_size(bytes.len());
        let slot_index = self.classes[class_id].alloc_owned(bytes)?;
        let generation = self.classes[class_id].generation(slot_index);
        let id = pack_id(class_id as u32, slot_index, generation)?;
        debug_assert!(self.get_refcount(id).unwrap_or(u32::MAX) == 0);
        Ok(id)
    }

    /// Reference externally managed bytes without copying. Reference
    /// `allocUnowned`. The borrow `'a` keeps the alias checked: the id must
    /// not outlive the lent bytes.
    pub fn alloc_unowned(&mut self, bytes: &'a [u8]) -> Result<IdPayload, GraphemePoolError> {
        if bytes.len() > u16::MAX as usize {
            return Err(GraphemePoolError::GraphemeTooLong);
        }
        let class_id = class_for_size(std::mem::size_of::<&[u8]>());
        debug_assert!((std::mem::size_of::<&[u8]>() as u32) <= CLASS_SIZES[class_id]);
        let slot_index = self.classes[class_id].alloc_shared(bytes)?;
        let generation = self.classes[class_id].generation(slot_index);
        let id = pack_id(class_id as u32, slot_index, generation)?;
        debug_assert!(self.get_refcount(id).unwrap_or(u32::MAX) == 0);
        Ok(id)
    }

    pub fn incref(&mut self, id: IdPayload) -> Result<(), GraphemePoolError> {
        let (class_id, slot_index, generation) = decode_id(id)?;
        let old_refcount = self.classes[class_id].get_refcount(slot_index, generation)?;
        if old_refcount == 0 && self.classes[class_id].is_owned(slot_index, generation)? {
            // Intern before publishing the first live reference so OOM
            // does not leave the caller holding an unreported reference.
            // The copy ends the pool borrow before the mutable call.
            let key = self.classes[class_id].get(slot_index, generation)?.to_vec();
            self.intern_live_id(id, &key)?;
        }
        self.classes[class_id].incref(slot_index, generation)?;
        debug_assert!(
            self.classes[class_id]
                .get_refcount(slot_index, generation)
                .unwrap_or(0)
                == old_refcount + 1
        );
        Ok(())
    }

    pub fn decref(&mut self, id: IdPayload) -> Result<(), GraphemePoolError> {
        let (class_id, slot_index, generation) = decode_id(id)?;
        let old_refcount = self.classes[class_id].get_refcount(slot_index, generation)?;
        if old_refcount == 1 && self.classes[class_id].is_owned(slot_index, generation)? {
            let key = self.classes[class_id].get(slot_index, generation)?.to_vec();
            self.remove_interned_live_id(&key, id);
        }
        self.classes[class_id].decref(slot_index, generation)?;
        if old_refcount > 1 {
            debug_assert!(
                self.classes[class_id]
                    .get_refcount(slot_index, generation)
                    .unwrap_or(0)
                    + 1
                    == old_refcount
            );
        } else {
            debug_assert!(old_refcount == 1);
            debug_assert!(
                self.classes[class_id]
                    .get_refcount(slot_index, generation)
                    .is_err()
            );
        }
        Ok(())
    }

    /// Free a freshly allocated slot that was never incref'd. Reference
    /// `freeUnreferenced`.
    pub fn free_unreferenced(&mut self, id: IdPayload) -> Result<(), GraphemePoolError> {
        let (class_id, slot_index, generation) = decode_id(id)?;
        if self.classes[class_id].is_owned(slot_index, generation)? {
            let key = self.classes[class_id].get(slot_index, generation)?.to_vec();
            self.remove_interned_live_id(&key, id);
        }
        self.classes[class_id].free_unreferenced(slot_index, generation)
    }

    pub fn get(&self, id: IdPayload) -> Result<&[u8], GraphemePoolError> {
        let (class_id, slot_index, generation) = decode_id(id)?;
        self.classes[class_id].get(slot_index, generation)
    }

    pub fn get_refcount(&self, id: IdPayload) -> Result<u32, GraphemePoolError> {
        let (class_id, slot_index, generation) = decode_id(id)?;
        self.classes[class_id].get_refcount(slot_index, generation)
    }
}

// ---- GraphemeTracker ----

/// Per-buffer reference holder for pool ids. Reference `GraphemeTracker`.
///
/// Holds each tracked id's cell count; the pool itself is shared through
/// `Rc<RefCell<..>>` so any number of trackers can reference one pool at
/// the same time, exactly as the reference does with raw pointers. Pool
/// errors inside tracker bookkeeping panic, matching the reference.
#[derive(Debug)]
pub struct GraphemeTracker<'a> {
    pool: Rc<RefCell<GraphemePool<'a>>>,
    used_ids: HashMap<IdPayload, u32>,
}

impl<'a> GraphemeTracker<'a> {
    pub fn new(pool: Rc<RefCell<GraphemePool<'a>>>) -> Self {
        GraphemeTracker {
            pool,
            used_ids: HashMap::new(),
        }
    }

    fn decref_all(&mut self) {
        // Pool refs are tracked per id (first/last cell transition), so
        // decref once per tracked id, not once per per-buffer cell count.
        let ids: Vec<IdPayload> = self.used_ids.keys().copied().collect();
        for id in ids {
            self.pool
                .borrow_mut()
                .decref(id)
                .unwrap_or_else(|err| panic!("GraphemeTracker decref failed: {err}"));
        }
    }

    pub fn clear(&mut self) {
        self.decref_all();
        self.used_ids.clear();
    }

    pub fn add(&mut self, id: IdPayload) {
        match self.used_ids.get_mut(&id) {
            Some(count) => {
                debug_assert!(*count > 0);
                debug_assert!(*count < u32::MAX);
                *count += 1;
            }
            None => {
                self.used_ids.insert(id, 1);
                self.pool
                    .borrow_mut()
                    .incref(id)
                    .unwrap_or_else(|err| panic!("GraphemeTracker.add incref failed: {err}"));
            }
        }
        debug_assert!(self.used_ids[&id] > 0);
    }

    pub fn remove(&mut self, id: IdPayload) {
        let count = match self.used_ids.get_mut(&id) {
            None => return,
            Some(count) => count,
        };
        debug_assert!(*count > 0);
        if *count > 1 {
            *count -= 1;
            debug_assert!(*count > 0);
            return;
        }
        self.used_ids.remove(&id);
        self.pool
            .borrow_mut()
            .decref(id)
            .unwrap_or_else(|err| panic!("GraphemeTracker.remove decref failed: {err}"));
    }

    pub fn replace(&mut self, old_id: Option<IdPayload>, new_id: Option<IdPayload>) {
        if old_id.is_some() && old_id == new_id {
            return;
        }
        if let Some(id) = new_id {
            self.add(id);
        }
        if let Some(id) = old_id {
            self.remove(id);
        }
    }

    pub fn contains(&self, id: IdPayload) -> bool {
        self.used_ids.contains_key(&id)
    }

    pub fn has_any(&self) -> bool {
        !self.used_ids.is_empty()
    }

    pub fn grapheme_count(&self) -> u32 {
        self.used_ids.len() as u32
    }

    pub fn cell_count(&self) -> u32 {
        let mut total: u32 = 0;
        for count in self.used_ids.values() {
            debug_assert!(*count > 0);
            debug_assert!(total <= u32::MAX - *count);
            total += *count;
        }
        total
    }

    pub fn total_bytes(&self) -> u32 {
        let pool = self.pool.borrow();
        let mut total_bytes: u32 = 0;
        for (id, count) in self.used_ids.iter() {
            let bytes = pool
                .get(*id)
                .unwrap_or_else(|err| panic!("GraphemeTracker.total_bytes get failed: {err}"));
            debug_assert!(*count > 0);
            let bytes_count = bytes.len() as u32;
            debug_assert!(bytes_count == 0 || *count <= u32::MAX / bytes_count);
            let bytes_total = bytes_count * *count;
            debug_assert!(total_bytes <= u32::MAX - bytes_total);
            total_bytes += bytes_total;
        }
        total_bytes
    }
}

impl<'a> Drop for GraphemeTracker<'a> {
    fn drop(&mut self) {
        self.decref_all();
    }
}
