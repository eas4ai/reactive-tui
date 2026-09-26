//! URL slab pool and per-buffer link tracker, ported from `link.zig`.
//!
//! The reference process-global link pool is excluded by design (same
//! rule as UNI-008 for grapheme pools): every pool is an explicit
//! caller-owned value, defaulting to a freshly owned pool when the
//! buffer is not given one. Generations are 8-bit and never wrap: a
//! slot decref'd at generation 255 retires permanently.
//!
//! One deliberate soundness deviation: when `add_cell_ref` meets an id
//! the pool rejects, the reference leaves a garbage cell count behind;
//! this port records a defined zero instead (same map presence, no
//! undefined value). No ported vector covers that path.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// ID layout within 24 bits: `[generation (8 bits) | slot (16 bits)]`.
pub type IdPayload = u32;

/// Number of id bits that hold the slot generation.
pub const GEN_BITS: u32 = 8;
/// Number of id bits that hold the slot index.
pub const SLOT_BITS: u32 = 16;
/// Mask for the generation after shifting an id right by [`SLOT_BITS`].
pub const GEN_MASK: u32 = (1 << GEN_BITS) - 1;
/// Mask for the slot index in the low bits of an id.
pub const SLOT_MASK: u32 = (1 << SLOT_BITS) - 1;
/// Longest URL a pool slot stores, in bytes.
pub const MAX_URL_LENGTH: usize = 512;
const RETIRED_GENERATION: u32 = GEN_MASK + 1;

/// Reference `LinkPoolError`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkPoolError {
    /// The pool cannot grow past 65,536 slots.
    OutOfMemory,
    /// The id's slot does not exist, or `decref` found no reference to drop.
    InvalidId,
    /// The id's generation does not match its slot, so the id is stale.
    WrongGeneration,
    /// The URL is longer than [`MAX_URL_LENGTH`] bytes.
    UrlTooLong,
}

impl fmt::Display for LinkPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkPoolError::OutOfMemory => write!(f, "link pool out of memory"),
            LinkPoolError::InvalidId => write!(f, "invalid link id"),
            LinkPoolError::WrongGeneration => write!(f, "stale link id generation"),
            LinkPoolError::UrlTooLong => write!(f, "URL exceeds storage bound"),
        }
    }
}

impl std::error::Error for LinkPoolError {}

#[derive(Debug)]
struct Slot {
    len: u32,
    refcount: u32,
    generation: u32,
    data: Vec<u8>,
}

/// URL pool with reusable ids. Reference `LinkPool`.
#[derive(Debug)]
pub struct LinkPool {
    slot_capacity: u32,
    slots_per_page: u32,
    slots: Vec<Slot>,
    free_list: Vec<u32>,
    retired_slot_count: u32,
    interned_live_ids: HashMap<Vec<u8>, IdPayload>,
}

fn pack_id(slot_index: u32, generation: u32) -> Result<IdPayload, LinkPoolError> {
    if slot_index > SLOT_MASK {
        return Err(LinkPoolError::OutOfMemory);
    }
    Ok(((generation & GEN_MASK) << SLOT_BITS) | (slot_index & SLOT_MASK))
}

fn unpack_id(id: IdPayload) -> (u32, u32) {
    (id & SLOT_MASK, (id >> SLOT_BITS) & GEN_MASK)
}

impl Default for LinkPool {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkPool {
    /// Create an empty pool. It adds slots 64 at a time as needed.
    pub fn new() -> Self {
        LinkPool {
            slot_capacity: MAX_URL_LENGTH as u32,
            slots_per_page: 64,
            slots: Vec::new(),
            free_list: Vec::new(),
            retired_slot_count: 0,
            interned_live_ids: HashMap::new(),
        }
    }

    fn num_slots(&self) -> u32 {
        self.slots.len() as u32
    }

    fn grow(&mut self) -> Result<(), LinkPoolError> {
        if self.num_slots() > SLOT_MASK + 1 - self.slots_per_page {
            return Err(LinkPoolError::OutOfMemory);
        }
        let base = self.num_slots();
        self.slots.extend((0..self.slots_per_page).map(|_| Slot {
            len: 0,
            refcount: 0,
            generation: 0,
            data: Vec::new(),
        }));
        self.free_list.extend(base..base + self.slots_per_page);
        // A slot is on the free list at most once, so a list that can hold
        // every slot never grows when a release pushes one back. Releases
        // run on the render path (RAS-003); growth happens here, on write.
        self.free_list
            .reserve(self.slots.len() - self.free_list.len());
        Ok(())
    }

    fn remove_interned_live_id(&mut self, url: &[u8], expected_id: IdPayload) {
        if self.interned_live_ids.get(url) != Some(&expected_id) {
            return;
        }
        self.interned_live_ids.remove(url);
    }

    fn lookup_or_invalidate(&mut self, url: &[u8]) -> Option<IdPayload> {
        let live_id = *self.interned_live_ids.get(url)?;
        let live_url = self.get(live_id).ok()?;
        if live_url != url {
            self.remove_interned_live_id(url, live_id);
            return None;
        }
        let live_refcount = self.get_refcount(live_id).ok()?;
        if live_refcount == 0 {
            self.remove_interned_live_id(url, live_id);
            return None;
        }
        Some(live_id)
    }

    fn intern_live_id(&mut self, id: IdPayload, url: &[u8]) {
        if self.lookup_or_invalidate(url).is_some() {
            return;
        }
        self.interned_live_ids.insert(url.to_vec(), id);
    }

    /// Store `url` and return its id with a refcount of zero.
    /// A URL that already has a referenced id returns that id instead.
    /// Fails with `UrlTooLong` or `OutOfMemory`.
    pub fn alloc(&mut self, url: &[u8]) -> Result<IdPayload, LinkPoolError> {
        if url.len() > self.slot_capacity as usize {
            return Err(LinkPoolError::UrlTooLong);
        }
        if let Some(live_id) = self.lookup_or_invalidate(url) {
            return Ok(live_id);
        }
        if self.free_list.is_empty() {
            self.grow()?;
        }
        let slot_index = self.free_list.pop().expect("grow guarantees a free slot");
        debug_assert!(slot_index < self.num_slots());
        let slot = &mut self.slots[slot_index as usize];
        debug_assert!(slot.generation < GEN_MASK);
        slot.len = url.len() as u32;
        slot.refcount = 0;
        slot.generation += 1;
        slot.data.clear();
        slot.data.extend_from_slice(url);
        pack_id(slot_index, slot.generation)
    }

    /// Add one reference to `id`. The first reference makes `id` the one
    /// [`LinkPool::alloc`] returns for its URL. Fails for an unknown or stale id.
    pub fn incref(&mut self, id: IdPayload) -> Result<(), LinkPoolError> {
        let (slot_index, generation) = unpack_id(id);
        if slot_index >= self.num_slots() {
            return Err(LinkPoolError::InvalidId);
        }
        let slot = &mut self.slots[slot_index as usize];
        if slot.generation != generation {
            return Err(LinkPoolError::WrongGeneration);
        }
        let old_refcount = slot.refcount;
        slot.refcount = slot.refcount.wrapping_add(1);
        if old_refcount == 0 {
            let url = slot.data.clone();
            self.intern_live_id(id, &url);
        }
        Ok(())
    }

    /// Drop one reference from `id`. At zero references the slot is freed
    /// for reuse, or retired for good at generation 255. Fails for an
    /// unknown or stale id, or one with no references.
    pub fn decref(&mut self, id: IdPayload) -> Result<(), LinkPoolError> {
        let (slot_index, generation) = unpack_id(id);
        if slot_index >= self.num_slots() {
            return Err(LinkPoolError::InvalidId);
        }
        let (refcount, slot_generation) = {
            let slot = &self.slots[slot_index as usize];
            (slot.refcount, slot.generation)
        };
        if refcount == 0 {
            return Err(LinkPoolError::InvalidId);
        }
        if slot_generation != generation {
            return Err(LinkPoolError::WrongGeneration);
        }
        if refcount == 1 {
            // Borrow the key from the slot: releasing a link runs on the
            // render path (RAS-003) and must not copy its URL.
            let url = self.slots[slot_index as usize].data.as_slice();
            if self.interned_live_ids.get(url) == Some(&id) {
                self.interned_live_ids.remove(url);
            }
        }
        let slot = &mut self.slots[slot_index as usize];
        slot.refcount -= 1;
        if slot.refcount == 0 {
            if slot.generation == GEN_MASK {
                slot.generation = RETIRED_GENERATION;
                self.retired_slot_count += 1;
            } else {
                self.free_list.push(slot_index);
            }
        }
        Ok(())
    }

    /// Return the URL bytes stored for `id`. Fails for an unknown or stale id.
    pub fn get(&self, id: IdPayload) -> Result<&[u8], LinkPoolError> {
        let (slot_index, generation) = unpack_id(id);
        if slot_index >= self.num_slots() {
            return Err(LinkPoolError::InvalidId);
        }
        let slot = &self.slots[slot_index as usize];
        if slot.generation != generation {
            return Err(LinkPoolError::WrongGeneration);
        }
        Ok(slot.data.as_slice())
    }

    /// Return the reference count of `id`. Fails for an unknown or stale id.
    pub fn get_refcount(&self, id: IdPayload) -> Result<u32, LinkPoolError> {
        let (slot_index, generation) = unpack_id(id);
        if slot_index >= self.num_slots() {
            return Err(LinkPoolError::InvalidId);
        }
        let slot = &self.slots[slot_index as usize];
        if slot.generation != generation {
            return Err(LinkPoolError::WrongGeneration);
        }
        Ok(slot.refcount)
    }

    /// Number of slots the pool has created, in use or not.
    pub fn total_slots(&self) -> u64 {
        u64::from(self.num_slots())
    }

    /// Number of slots ready for reuse.
    pub fn free_slot_count(&self) -> u64 {
        self.free_list.len() as u64
    }

    /// Number of slots that are neither free nor retired.
    pub fn live_slot_count(&self) -> u64 {
        u64::from(self.num_slots())
            - self.free_list.len() as u64
            - u64::from(self.retired_slot_count)
    }
}

// ---- LinkTracker ----

/// Per-buffer link usage with per-cell refcounting. Reference
/// `LinkTracker`. Pool errors inside bookkeeping are silently ignored,
/// matching the reference (garbage attribute bits must never panic).
#[derive(Debug)]
pub struct LinkTracker {
    pool: Rc<RefCell<LinkPool>>,
    used_ids: HashMap<IdPayload, u32>,
}

impl LinkTracker {
    /// Create an empty tracker that takes its references in `pool`.
    pub fn new(pool: Rc<RefCell<LinkPool>>) -> Self {
        LinkTracker {
            pool,
            used_ids: HashMap::new(),
        }
    }

    fn decref_all(&mut self) {
        // Draining keeps the map's capacity and copies no id list.
        for (id, _) in self.used_ids.drain() {
            let _ = self.pool.borrow_mut().decref(id);
        }
    }

    /// Forget every tracked id and release the tracker's pool references.
    pub fn clear(&mut self) {
        self.decref_all();
    }

    /// Make room to track `ids` distinct ids with no further allocation,
    /// whatever the order of adds and removes. The renderer reserves its
    /// current buffer's tracker for one id per cell, since `render` syncs
    /// cells into that buffer (RAS-003).
    pub fn reserve(&mut self, ids: usize) {
        // Twice the ids: the standard map reuses the slots removed entries
        // leave by rehashing in place, which it does only while at most
        // half full; past that it moves to a larger table.
        let wanted = ids.saturating_mul(2).saturating_add(2);
        self.used_ids
            .reserve(wanted.saturating_sub(self.used_ids.len()));
    }

    /// Count one more cell that uses `id`. The first cell for an id takes one
    /// pool reference; an id the pool rejects is tracked with a count of zero.
    pub fn add_cell_ref(&mut self, id: IdPayload) {
        use std::collections::hash_map::Entry;
        match self.used_ids.entry(id) {
            Entry::Vacant(slot) => {
                // First sighting: publish one pool ref. An invalid id
                // (garbage attribute bits) is silently ignored with a
                // defined zero count instead of the reference's garbage
                // value; map presence still matches.
                if self.pool.borrow_mut().incref(id).is_ok() {
                    slot.insert(1);
                } else {
                    slot.insert(0);
                }
            }
            Entry::Occupied(mut slot) => {
                *slot.get_mut() += 1;
            }
        }
    }

    /// Count one fewer cell that uses `id`. At zero the tracker drops the id
    /// and releases its pool reference. Unknown ids and zero counts are ignored.
    pub fn remove_cell_ref(&mut self, id: IdPayload) {
        let remove = match self.used_ids.get_mut(&id) {
            None => return,
            Some(count) => {
                if *count == 0 {
                    return;
                }
                *count -= 1;
                *count == 0
            }
        };
        if remove {
            self.used_ids.remove(&id);
            let _ = self.pool.borrow_mut().decref(id);
        }
    }

    /// Whether the tracker holds any id, counting ids the pool rejected.
    pub fn has_any(&self) -> bool {
        !self.used_ids.is_empty()
    }

    /// Number of distinct ids the tracker holds, counting ids the pool
    /// rejected, which it keeps with a cell count of zero.
    pub fn link_count(&self) -> u32 {
        self.used_ids.len() as u32
    }

    /// Tracked cell count for one id. Observability accessor for the
    /// ported per-cell counting vectors.
    pub fn cell_count(&self, id: IdPayload) -> Option<u32> {
        self.used_ids.get(&id).copied()
    }
}

impl Drop for LinkTracker {
    fn drop(&mut self) {
        self.decref_all();
    }
}
