//! Ported vectors for `link.zig` (`LinkPool`, `LinkTracker`).
//!
//! The link pool is delivered with `buffer-core` as its dependency; the
//! terminal link behavior it serves arrives with `sys-core`. Global-pool
//! cases are excluded by the same caller-ownership rule as UNI-008.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::link::{GEN_MASK, LinkPool, LinkPoolError, LinkTracker};

fn shared_pool() -> Rc<RefCell<LinkPool>> {
    Rc::new(RefCell::new(LinkPool::new()))
}

#[test]
fn link_pool_init() {
    // ported: LinkPool - can initialize and cleanup
    let pool = LinkPool::new();
    drop(pool);
}

#[test]
fn link_pool_alloc_get() {
    // ported: LinkPool - alloc and get URL
    let mut pool = LinkPool::new();
    let id = pool.alloc(b"https://example.com").unwrap();
    pool.incref(id).unwrap();
    assert_eq!(b"https://example.com", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn link_pool_reuse() {
    // ported: LinkPool - decref to zero allows slot reuse
    let mut pool = LinkPool::new();
    let id1 = pool.alloc(b"https://first.example").unwrap();
    pool.incref(id1).unwrap();
    pool.decref(id1).unwrap();
    let id2 = pool.alloc(b"https://second.example").unwrap();
    pool.incref(id2).unwrap();
    assert_eq!(Err(LinkPoolError::WrongGeneration), pool.get(id1));
    assert_eq!(Err(LinkPoolError::WrongGeneration), pool.incref(id1));
    assert_eq!(Err(LinkPoolError::WrongGeneration), pool.decref(id1));
    assert_eq!(b"https://second.example", pool.get(id2).unwrap());
    pool.decref(id2).unwrap();
}

#[test]
fn link_pool_decref_zero_refcount() {
    // ported: LinkPool - decref on zero refcount fails
    let mut pool = LinkPool::new();
    let id = pool.alloc(b"https://example.com").unwrap();
    assert_eq!(Err(LinkPoolError::InvalidId), pool.decref(id));
}

#[test]
fn link_pool_never_zero_id() {
    // ported: LinkPool - alloc never returns sentinel zero ID
    let mut pool = LinkPool::new();
    for _ in 0..300 {
        let id = pool.alloc(b"https://example.com/rotate").unwrap();
        assert_ne!(0, id);
        pool.incref(id).unwrap();
        pool.decref(id).unwrap();
    }
}

#[test]
fn link_pool_generation_exhaustion() {
    // ported: LinkPool - stale ID stays invalid after generation exhaustion
    let mut pool = LinkPool::new();
    let stale_id = pool.alloc(b"https://example.com/stale").unwrap();
    pool.incref(stale_id).unwrap();
    pool.decref(stale_id).unwrap();

    let mut exhausted_id = 0;
    let mut generation = 2u32;
    while generation <= GEN_MASK {
        let id = pool.alloc(b"https://example.com/rotate").unwrap();
        pool.incref(id).unwrap();
        pool.decref(id).unwrap();
        if generation == GEN_MASK {
            exhausted_id = id;
        }
        generation += 1;
    }

    assert_eq!(
        Err(LinkPoolError::WrongGeneration),
        pool.incref(exhausted_id)
    );
    assert_eq!(0, pool.live_slot_count());

    let live_id = pool.alloc(b"https://example.com/live").unwrap();
    pool.incref(live_id).unwrap();
    assert_ne!(stale_id, live_id);
    assert_eq!(Err(LinkPoolError::WrongGeneration), pool.get(stale_id));
    assert_eq!(b"https://example.com/live", pool.get(live_id).unwrap());
    assert_eq!(1, pool.live_slot_count());
    pool.decref(live_id).unwrap();
}

#[test]
fn link_tracker_add_remove() {
    // ported: LinkTracker - add/remove keeps one pool ref per ID
    let pool = shared_pool();
    let id = pool
        .borrow_mut()
        .alloc(b"https://example.com/same")
        .unwrap();
    let mut tracker = LinkTracker::new(Rc::clone(&pool));
    tracker.add_cell_ref(id);
    tracker.add_cell_ref(id);
    tracker.add_cell_ref(id);
    assert_eq!(1, tracker.link_count());
    assert_eq!(1, pool.borrow().get_refcount(id).unwrap());
    tracker.remove_cell_ref(id);
    assert_eq!(1, tracker.link_count());
    assert_eq!(1, pool.borrow().get_refcount(id).unwrap());
    tracker.remove_cell_ref(id);
    assert_eq!(1, tracker.link_count());
    assert_eq!(1, pool.borrow().get_refcount(id).unwrap());
    tracker.remove_cell_ref(id);
    assert_eq!(0, tracker.link_count());
    assert_eq!(0, pool.borrow().get_refcount(id).unwrap());
    drop(tracker);
}

#[test]
fn link_tracker_clear() {
    // ported: LinkTracker - clear releases tracked IDs
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc(b"https://example.com/1").unwrap();
    let id2 = pool_ref.alloc(b"https://example.com/2").unwrap();
    drop(pool_ref);
    let mut tracker = LinkTracker::new(Rc::clone(&pool));
    tracker.add_cell_ref(id1);
    tracker.add_cell_ref(id2);
    assert!(tracker.has_any());
    assert_eq!(
        2,
        pool.borrow().get_refcount(id1).unwrap() + pool.borrow().get_refcount(id2).unwrap()
    );
    tracker.clear();
    assert!(!tracker.has_any());
    assert_eq!(0, pool.borrow().get_refcount(id1).unwrap());
    assert_eq!(0, pool.borrow().get_refcount(id2).unwrap());
    drop(tracker);
}

#[test]
fn link_tracker_clear_once_per_id() {
    // ported: LinkTracker - clear only decrefs once per ID with multiple cell refs
    let pool = shared_pool();
    let id = pool
        .borrow_mut()
        .alloc(b"https://example.com/shared")
        .unwrap();
    let mut tracker_a = LinkTracker::new(Rc::clone(&pool));
    let mut tracker_b = LinkTracker::new(Rc::clone(&pool));
    tracker_a.add_cell_ref(id);
    tracker_a.add_cell_ref(id);
    tracker_a.add_cell_ref(id);
    tracker_b.add_cell_ref(id);
    assert_eq!(2, pool.borrow().get_refcount(id).unwrap());
    tracker_a.clear();
    assert_eq!(1, pool.borrow().get_refcount(id).unwrap());
    assert_eq!(
        b"https://example.com/shared",
        pool.borrow().get(id).unwrap()
    );
    drop(tracker_a);
    drop(tracker_b);
}

#[test]
fn link_pool_alloc_only_accumulates() {
    // ported: LinkPool - leak repro: alloc-only IDs accumulate live slots
    let mut pool = LinkPool::new();
    for i in 0..4096u32 {
        let url = format!("https://example.com/r{i}");
        let _ = pool.alloc(url.as_bytes()).unwrap();
    }
    assert!(pool.live_slot_count() > 0);
    assert!(pool.free_slot_count() < pool.total_slots());
}

#[test]
fn link_pool_intern_live() {
    // ported: LinkPool - alloc reuses live ID for same URL
    let mut pool = LinkPool::new();
    let id1 = pool.alloc(b"https://example.com/stable").unwrap();
    pool.incref(id1).unwrap();
    let id2 = pool.alloc(b"https://example.com/stable").unwrap();
    assert_eq!(id1, id2);
    assert_eq!(1, pool.get_refcount(id1).unwrap());
    pool.decref(id1).unwrap();
    let id3 = pool.alloc(b"https://example.com/stable").unwrap();
    pool.incref(id3).unwrap();
    assert_ne!(id3, id1);
    assert_eq!(b"https://example.com/stable", pool.get(id3).unwrap());
    let id4 = pool.alloc(b"https://example.com/stable").unwrap();
    assert_eq!(id3, id4);
    pool.decref(id3).unwrap();
}
