//! Ported vectors for `grapheme.zig` pool + tracker (UNI-008, `uni-pool`).
//!
//! One `#[test]` per reference case in `tests/grapheme_test.zig`, named
//! `req_008_<slug>`. The three global-pool cases are NOT ported: UNI-008
//! forbids process-global pools, and isolation is covered by
//! `req_008_pool_isolation` plus the `uni::pool_isolation` unit test.
//! Packing vectors at the end exercise the same packed-char API the pool
//! ids feed into (UNI-009 behavior, covered there as well).

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::uni::pool::{
    GraphemePool, GraphemePoolError, GraphemeTracker, InitOptions, SLOT_BITS,
};
use suprtui::uni::segments::{
    CHAR_EXT_MASK, CHAR_EXT_RIGHT_SHIFT, CHAR_FLAG_CONTINUATION, CHAR_FLAG_GRAPHEME,
    char_left_extent, char_right_extent, encoded_char_width, grapheme_id_from_char,
    image_fallback_from_char, image_id_from_char, is_cluster_char, is_continuation_char,
    is_grapheme_char, is_image_char, pack_continuation, pack_grapheme_start, pack_image_cell,
};

fn shared_pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::new()))
}

#[test]
fn req_008_pool_init_and_cleanup() {
    // ported: GraphemePool - can initialize and cleanup
    let pool = GraphemePool::new();
    drop(pool);
}

#[test]
fn req_008_pool_alloc_get_small() {
    // ported: GraphemePool - alloc and get small grapheme
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"a").unwrap();
    pool.incref(id).unwrap();
    assert_eq!(b"a", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_alloc_get_emoji() {
    // ported: GraphemePool - alloc and get emoji
    let mut pool = GraphemePool::new();
    let id = pool.alloc("🌟".as_bytes()).unwrap();
    pool.incref(id).unwrap();
    assert_eq!("🌟".as_bytes(), pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_alloc_get_multibyte() {
    // ported: GraphemePool - alloc and get multi-byte grapheme
    let mut pool = GraphemePool::new();
    let id = pool.alloc("é".as_bytes()).unwrap();
    pool.incref(id).unwrap();
    assert_eq!("é".as_bytes(), pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_alloc_get_combining() {
    // ported: GraphemePool - alloc and get combining character grapheme
    let mut pool = GraphemePool::new();
    let id = pool.alloc("é".as_bytes()).unwrap();
    pool.incref(id).unwrap();
    assert_eq!("é".as_bytes(), pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_multiple_allocations() {
    // ported: GraphemePool - multiple allocations
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc(b"a").unwrap();
    let id2 = pool.alloc(b"b").unwrap();
    let id3 = pool.alloc("🌟".as_bytes()).unwrap();
    pool.incref(id1).unwrap();
    pool.incref(id2).unwrap();
    pool.incref(id3).unwrap();
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
    assert_eq!(b"a", pool.get(id1).unwrap());
    assert_eq!(b"b", pool.get(id2).unwrap());
    assert_eq!("🌟".as_bytes(), pool.get(id3).unwrap());
    pool.decref(id1).unwrap();
    pool.decref(id2).unwrap();
    pool.decref(id3).unwrap();
}

#[test]
fn req_008_pool_various_sizes() {
    // ported: GraphemePool - handles various size graphemes
    let mut pool = GraphemePool::new();
    let id_small = pool.alloc(b"a").unwrap();
    let id_medium = pool.alloc(b"0123456789").unwrap();
    let id_large = pool.alloc(b"012345678901234567890123456789").unwrap();
    pool.incref(id_small).unwrap();
    pool.incref(id_medium).unwrap();
    pool.incref(id_large).unwrap();
    assert_eq!(b"a", pool.get(id_small).unwrap());
    assert_eq!(b"0123456789", pool.get(id_medium).unwrap());
    assert_eq!(
        b"012345678901234567890123456789",
        pool.get(id_large).unwrap()
    );
    pool.decref(id_small).unwrap();
    pool.decref(id_medium).unwrap();
    pool.decref(id_large).unwrap();
}

#[test]
fn req_008_pool_large_allocation() {
    // ported: GraphemePool - large allocation (128 bytes)
    let mut pool = GraphemePool::new();
    let buffer = [b'X'; 128];
    let id = pool.alloc(&buffer).unwrap();
    pool.incref(id).unwrap();
    let retrieved = pool.get(id).unwrap();
    assert_eq!(128, retrieved.len());
    assert_eq!(&buffer, retrieved);
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_owned_too_long() {
    // ported: GraphemePool - owned grapheme exceeding storage bound returns error
    let mut pool = GraphemePool::new();
    let buffer = [b'X'; 129];
    assert_eq!(Err(GraphemePoolError::GraphemeTooLong), pool.alloc(&buffer));
}

#[test]
fn req_008_pool_incref() {
    // ported: GraphemePool - incref increases refcount
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"a").unwrap();
    pool.incref(id).unwrap();
    assert_eq!(b"a", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_decref_keeps_alive() {
    // ported: GraphemePool - decref once keeps data alive
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"a").unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    pool.decref(id).unwrap();
    assert_eq!(b"a", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_decref_zero_reuse() {
    // ported: GraphemePool - decref to zero allows slot reuse
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc(b"a").unwrap();
    pool.incref(id1).unwrap();
    pool.decref(id1).unwrap();
    let id2 = pool.alloc(b"b").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id1));
    pool.incref(id2).unwrap();
    assert_eq!(b"b", pool.get(id2).unwrap());
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_multiple_incref_decref() {
    // ported: GraphemePool - multiple incref and decref
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    pool.decref(id).unwrap();
    pool.decref(id).unwrap();
    assert_eq!(b"test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
    let _ = pool.alloc(b"x").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id));
}

#[test]
fn req_008_pool_freed_invalid_after_reuse() {
    // ported: GraphemePool - freed IDs become invalid after reuse
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc(b"a").unwrap();
    pool.incref(id1).unwrap();
    pool.decref(id1).unwrap();
    let id2 = pool.alloc(b"b").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id1));
    pool.incref(id2).unwrap();
    assert_eq!(b"b", pool.get(id2).unwrap());
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_stale_generation() {
    // ported: GraphemePool - stale ID with wrong generation fails
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    pool.incref(id).unwrap();
    let stale_id = id ^ (1 << SLOT_BITS);
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(stale_id));
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_decref_zero_refcount() {
    // ported: GraphemePool - decref on zero refcount fails
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"a").unwrap();
    assert_eq!(Err(GraphemePoolError::InvalidId), pool.decref(id));
}

#[test]
fn req_008_pool_many_allocations() {
    // ported: GraphemePool - many allocations
    let mut pool = GraphemePool::new();
    let mut ids = Vec::with_capacity(1000);
    for i in 0..1000u32 {
        let text = format!("{i}");
        let id = pool.alloc(text.as_bytes()).unwrap();
        pool.incref(id).unwrap();
        ids.push((id, text));
    }
    for (id, text) in &ids {
        assert_eq!(text.as_bytes(), pool.get(*id).unwrap());
    }
    for (id, _) in &ids {
        pool.decref(*id).unwrap();
    }
}

#[test]
fn req_008_pool_varying_sizes() {
    // ported: GraphemePool - allocations with varying sizes
    let mut pool = GraphemePool::new();
    let mut ids = Vec::new();
    for i in 0..50u8 {
        let size = (i % 5) as usize * 16 + 5;
        let buffer = vec![i; size];
        let id = pool.alloc(&buffer).unwrap();
        pool.incref(id).unwrap();
        ids.push((id, size, i));
    }
    for (id, size, byte) in &ids {
        let retrieved = pool.get(*id).unwrap();
        assert_eq!(*size, retrieved.len());
        assert!(retrieved.iter().all(|b| *b == *byte));
    }
    for (id, _, _) in &ids {
        pool.decref(*id).unwrap();
    }
}

#[test]
fn req_008_pool_reuse_many_slots() {
    // ported: GraphemePool - reuse many slots
    let mut pool = GraphemePool::new();
    for i in 0..100u32 {
        let text = format!("{i}");
        let id = pool.alloc(text.as_bytes()).unwrap();
        pool.incref(id).unwrap();
        assert_eq!(text.as_bytes(), pool.get(id).unwrap());
        pool.decref(id).unwrap();
    }
}

#[test]
fn req_008_pool_invalid_id_after_reuse() {
    // ported: GraphemePool - invalid ID returns error
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    pool.incref(id).unwrap();
    pool.decref(id).unwrap();
    let _ = pool.alloc(b"test2").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id));
}

#[test]
fn req_008_pool_isolation() {
    // UNI-008 falsifier at integration level (unit level lives at
    // `uni::pool_isolation`): independently created pools must not
    // observe each other's entries.
    let mut pool1 = GraphemePool::new();
    let mut pool2 = GraphemePool::new();
    let id1 = pool1.alloc(b"pool1_data").unwrap();
    let id2 = pool2.alloc(b"pool2_data").unwrap();
    pool1.incref(id1).unwrap();
    pool2.incref(id2).unwrap();
    assert_eq!(b"pool1_data", pool1.get(id1).unwrap());
    assert_eq!(b"pool2_data", pool2.get(id2).unwrap());
    match pool2.get(id1) {
        Err(GraphemePoolError::InvalidId) => {}
        Ok(bytes) => assert_ne!(b"pool1_data", bytes),
        Err(other) => panic!("unexpected error: {other}"),
    }
    pool1.decref(id1).unwrap();
    pool2.decref(id2).unwrap();
}

#[test]
fn req_008_pool_use_after_free() {
    // ported: GraphemePool - use-after-free returns error not garbage
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc(b"first").unwrap();
    pool.incref(id1).unwrap();
    pool.decref(id1).unwrap();
    let id2 = pool.alloc(b"second").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id1));
    pool.incref(id2).unwrap();
    assert_eq!(b"second", pool.get(id2).unwrap());
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_unique_ids() {
    // ported: GraphemePool - IDs remain unique across many allocations
    let mut pool = GraphemePool::new();
    let mut ids = Vec::with_capacity(100);
    for i in 0..100u32 {
        let text = format!("{i}");
        let id = pool.alloc(text.as_bytes()).unwrap();
        pool.incref(id).unwrap();
        ids.push(id);
    }
    for (i, id1) in ids.iter().enumerate() {
        for id2 in &ids[i + 1..] {
            assert_ne!(id1, id2);
        }
    }
    for id in &ids {
        pool.decref(*id).unwrap();
    }
}

#[test]
fn req_008_pool_concurrent_refcount() {
    // ported: GraphemePool - concurrent incref/decref maintains consistency
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    assert_eq!(b"test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
    assert_eq!(b"test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
    assert_eq!(b"test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_zero_length() {
    // ported: GraphemePool - zero-length grapheme
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"").unwrap();
    pool.incref(id).unwrap();
    assert_eq!(0, pool.get(id).unwrap().len());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_incref_stale() {
    // ported: GraphemePool - incref on stale ID fails
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    pool.incref(id).unwrap();
    pool.decref(id).unwrap();
    let _ = pool.alloc(b"new").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.incref(id));
}

#[test]
fn req_008_pool_decref_stale() {
    // ported: GraphemePool - decref on stale ID fails
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"test").unwrap();
    assert_eq!(Err(GraphemePoolError::InvalidId), pool.decref(id));
}

// ---- unowned allocations ----

#[test]
fn req_008_pool_unowned_basic() {
    // ported: GraphemePool - allocUnowned basic
    let mut pool = GraphemePool::new();
    let external_text = "external";
    let id = pool.alloc_unowned(external_text.as_bytes()).unwrap();
    pool.incref(id).unwrap();
    let retrieved = pool.get(id).unwrap();
    assert_eq!(external_text.as_bytes(), retrieved);
    // Same memory location: the pool aliases caller memory.
    assert_eq!(external_text.as_ptr(), retrieved.as_ptr());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_unowned_multiple() {
    // ported: GraphemePool - allocUnowned multiple references
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc_unowned(b"external1").unwrap();
    let id2 = pool.alloc_unowned(b"external2").unwrap();
    let id3 = pool.alloc_unowned(b"external3").unwrap();
    pool.incref(id1).unwrap();
    pool.incref(id2).unwrap();
    pool.incref(id3).unwrap();
    assert_eq!(b"external1", pool.get(id1).unwrap());
    assert_eq!(b"external2", pool.get(id2).unwrap());
    assert_eq!(b"external3", pool.get(id3).unwrap());
    assert_eq!(b"external1".as_ptr(), pool.get(id1).unwrap().as_ptr());
    assert_eq!(b"external2".as_ptr(), pool.get(id2).unwrap().as_ptr());
    assert_eq!(b"external3".as_ptr(), pool.get(id3).unwrap().as_ptr());
    pool.decref(id1).unwrap();
    pool.decref(id2).unwrap();
    pool.decref(id3).unwrap();
}

#[test]
fn req_008_pool_unowned_emoji() {
    // ported: GraphemePool - allocUnowned with emoji
    let mut pool = GraphemePool::new();
    let external_emoji = "🌟🎉🚀";
    let id = pool.alloc_unowned(external_emoji.as_bytes()).unwrap();
    pool.incref(id).unwrap();
    let retrieved = pool.get(id).unwrap();
    assert_eq!(external_emoji.as_bytes(), retrieved);
    assert_eq!(external_emoji.as_ptr(), retrieved.as_ptr());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_unowned_refcounting() {
    // ported: GraphemePool - allocUnowned refcounting
    let mut pool = GraphemePool::new();
    let id = pool.alloc_unowned(b"refcount_test").unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    pool.incref(id).unwrap();
    assert_eq!(b"refcount_test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
    assert_eq!(b"refcount_test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
    assert_eq!(b"refcount_test", pool.get(id).unwrap());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_mixed_owned_unowned() {
    // ported: GraphemePool - mix owned and unowned allocations
    let mut pool = GraphemePool::new();
    let owned_id = pool.alloc(b"owned").unwrap();
    let external_text = "unowned";
    let unowned_id = pool.alloc_unowned(external_text.as_bytes()).unwrap();
    pool.incref(owned_id).unwrap();
    pool.incref(unowned_id).unwrap();
    let retrieved_owned = pool.get(owned_id).unwrap();
    let retrieved_unowned = pool.get(unowned_id).unwrap();
    assert_eq!(b"owned", retrieved_owned);
    assert_eq!(external_text.as_bytes(), retrieved_unowned);
    // Owned is a copy; unowned aliases the source.
    assert_ne!(b"owned".as_ptr(), retrieved_owned.as_ptr());
    assert_eq!(external_text.as_ptr(), retrieved_unowned.as_ptr());
    pool.decref(owned_id).unwrap();
    pool.decref(unowned_id).unwrap();
}

#[test]
fn req_008_pool_unowned_slot_reuse() {
    // ported: GraphemePool - allocUnowned slot reuse
    let mut pool = GraphemePool::new();
    let id1 = pool.alloc_unowned(b"first").unwrap();
    pool.incref(id1).unwrap();
    pool.decref(id1).unwrap();
    let id2 = pool.alloc_unowned(b"second").unwrap();
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id1));
    pool.incref(id2).unwrap();
    let retrieved = pool.get(id2).unwrap();
    assert_eq!(b"second", retrieved);
    assert_eq!(b"second".as_ptr(), retrieved.as_ptr());
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_unowned_large() {
    // ported: GraphemePool - allocUnowned large text
    let mut pool = GraphemePool::new();
    let large_buffer = [b'X'; 1000];
    let id = pool.alloc_unowned(&large_buffer).unwrap();
    pool.incref(id).unwrap();
    let retrieved = pool.get(id).unwrap();
    assert_eq!(1000, retrieved.len());
    assert_eq!(&large_buffer, retrieved);
    assert_eq!(large_buffer.as_ptr(), retrieved.as_ptr());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_unowned_too_long() {
    // ported: GraphemePool - allocUnowned exceeding encoded length returns error
    let mut pool = GraphemePool::new();
    let buffer = vec![0u8; u16::MAX as usize + 1];
    assert_eq!(
        Err(GraphemePoolError::GraphemeTooLong),
        pool.alloc_unowned(&buffer)
    );
}

#[test]
fn req_008_pool_owned_skips_unowned() {
    // ported: GraphemePool - alloc does not reuse unowned IDs
    let mut pool = GraphemePool::new();
    let external_text = "shared";
    let unowned_id = pool.alloc_unowned(external_text.as_bytes()).unwrap();
    pool.incref(unowned_id).unwrap();
    let owned_id = pool.alloc(external_text.as_bytes()).unwrap();
    pool.incref(owned_id).unwrap();
    assert_ne!(owned_id, unowned_id);
    let owned_bytes = pool.get(owned_id).unwrap();
    assert_eq!(external_text.as_bytes(), owned_bytes);
    assert_ne!(external_text.as_ptr(), owned_bytes.as_ptr());
    pool.decref(unowned_id).unwrap();
    pool.decref(owned_id).unwrap();
}

#[test]
fn req_008_pool_unowned_stack() {
    // ported: GraphemePool - allocUnowned with stack memory
    let mut pool = GraphemePool::new();
    let mut stack_buffer = [0u8; 50];
    stack_buffer[0..11].copy_from_slice(b"stack_based");
    let stack_slice = &stack_buffer[0..11];
    let id = pool.alloc_unowned(stack_slice).unwrap();
    pool.incref(id).unwrap();
    let retrieved = pool.get(id).unwrap();
    assert_eq!(b"stack_based", retrieved);
    assert_eq!(stack_slice.as_ptr(), retrieved.as_ptr());
    pool.decref(id).unwrap();
}

#[test]
fn req_008_pool_unowned_empty() {
    // ported: GraphemePool - allocUnowned zero-length slice
    let mut pool = GraphemePool::new();
    let id = pool.alloc_unowned(b"").unwrap();
    pool.incref(id).unwrap();
    assert_eq!(0, pool.get(id).unwrap().len());
    pool.decref(id).unwrap();
}

// ---- pool options, interning, exhaustion ----

#[test]
fn req_008_pool_small_slots() {
    // ported: GraphemePool - initWithOptions with small slots_per_page
    let mut pool = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([2, 2, 2, 2, 2]),
    });
    let id1 = pool.alloc(b"abc").unwrap();
    let id2 = pool.alloc(b"def").unwrap();
    pool.incref(id1).unwrap();
    pool.incref(id2).unwrap();
    assert_eq!(b"abc", pool.get(id1).unwrap());
    assert_eq!(b"def", pool.get(id2).unwrap());
    pool.decref(id1).unwrap();
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_intern_live_id() {
    // ported: GraphemePool - alloc reuses live ID for same bytes
    let mut pool = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([1, 1, 1, 1, 1]),
    });
    let id1 = pool.alloc("👋".as_bytes()).unwrap();
    pool.incref(id1).unwrap();
    let id2 = pool.alloc("👋".as_bytes()).unwrap();
    assert_eq!(id1, id2);
    assert_eq!(1, pool.get_refcount(id1).unwrap());
    pool.decref(id1).unwrap();
    let id3 = pool.alloc("👋".as_bytes()).unwrap();
    pool.incref(id3).unwrap();
    assert_ne!(id3, id1);
    assert_eq!("👋".as_bytes(), pool.get(id3).unwrap());
    let id4 = pool.alloc("👋".as_bytes()).unwrap();
    assert_eq!(id3, id4);
    pool.decref(id3).unwrap();
}

#[test]
fn req_008_pool_growth() {
    // ported: GraphemePool - small pool exhaustion and growth
    let mut pool = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([1, 1, 1, 1, 1]),
    });
    let id1 = pool.alloc(b"a").unwrap();
    let id2 = pool.alloc(b"b").unwrap();
    pool.incref(id1).unwrap();
    pool.incref(id2).unwrap();
    assert_eq!(b"a", pool.get(id1).unwrap());
    assert_eq!(b"b", pool.get(id2).unwrap());
    pool.decref(id1).unwrap();
    pool.decref(id2).unwrap();
}

#[test]
fn req_008_pool_refcount_blocks_reuse() {
    // ported: GraphemePool - small pool with refcount prevents exhaustion
    let mut pool = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([2, 2, 2, 2, 2]),
    });
    let id1 = pool.alloc(b"aa").unwrap();
    let id2 = pool.alloc(b"bb").unwrap();
    pool.incref(id1).unwrap();
    pool.incref(id2).unwrap();
    pool.decref(id1).unwrap();
    let id3 = pool.alloc(b"cc").unwrap();
    pool.incref(id3).unwrap();
    assert_eq!(b"bb", pool.get(id2).unwrap());
    assert_eq!(b"cc", pool.get(id3).unwrap());
    assert_eq!(Err(GraphemePoolError::WrongGeneration), pool.get(id1));
    pool.decref(id2).unwrap();
    pool.decref(id3).unwrap();
}

#[test]
fn req_008_pool_size_classes() {
    // ported: GraphemePool - different size classes with small limits
    let mut pool = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([2, 2, 2, 2, 2]),
    });
    let id_small = pool.alloc(b"ab").unwrap();
    let id_medium = pool.alloc(b"0123456789abc").unwrap();
    let id_large = pool.alloc(b"012345678901234567890").unwrap();
    pool.incref(id_small).unwrap();
    pool.incref(id_medium).unwrap();
    pool.incref(id_large).unwrap();
    assert_eq!(b"ab", pool.get(id_small).unwrap());
    assert_eq!(b"0123456789abc", pool.get(id_medium).unwrap());
    assert_eq!(b"012345678901234567890", pool.get(id_large).unwrap());
    pool.decref(id_small).unwrap();
    pool.decref(id_medium).unwrap();
    pool.decref(id_large).unwrap();
}

// ---- tracker ----

#[test]
fn req_008_tracker_init() {
    // ported: GraphemeTracker - init and deinit
    let pool = shared_pool();
    let tracker = GraphemeTracker::new(Rc::clone(&pool));
    assert!(!tracker.has_any());
    assert_eq!(0, tracker.grapheme_count());
    drop(tracker);
}

#[test]
fn req_008_tracker_add_single() {
    // ported: GraphemeTracker - add single grapheme
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"a").unwrap();
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id);
    assert!(tracker.has_any());
    assert!(tracker.contains(id));
    assert_eq!(1, tracker.grapheme_count());
    drop(tracker);
}

#[test]
fn req_008_tracker_add_multiple() {
    // ported: GraphemeTracker - add multiple graphemes
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc(b"a").unwrap();
    let id2 = pool_ref.alloc(b"b").unwrap();
    let id3 = pool_ref.alloc("🌟".as_bytes()).unwrap();
    drop(pool_ref);
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id1);
    tracker.add(id2);
    tracker.add(id3);
    assert_eq!(3, tracker.grapheme_count());
    assert!(tracker.contains(id1));
    assert!(tracker.contains(id2));
    assert!(tracker.contains(id3));
    drop(tracker);
}

#[test]
fn req_008_tracker_add_twice_single_incref() {
    // ported: GraphemeTracker - add same grapheme twice increfs once
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"a").unwrap();
    {
        let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
        tracker.add(id);
        tracker.add(id);
        assert_eq!(1, tracker.grapheme_count());
        assert_eq!(2, tracker.cell_count());
        assert_eq!(2, tracker.total_bytes());
        tracker.remove(id);
        assert!(tracker.contains(id));
        assert_eq!(1, tracker.cell_count());
    }
    let _ = pool.borrow_mut().alloc(b"b").unwrap();
    assert_eq!(
        Err(GraphemePoolError::WrongGeneration),
        pool.borrow().get(id)
    );
}

#[test]
fn req_008_tracker_remove() {
    // ported: GraphemeTracker - remove grapheme
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"a").unwrap();
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id);
    assert!(tracker.contains(id));
    tracker.remove(id);
    assert!(!tracker.contains(id));
    assert_eq!(0, tracker.grapheme_count());
    drop(tracker);
}

#[test]
fn req_008_tracker_remove_missing_safe() {
    // ported: GraphemeTracker - remove non-existent grapheme is safe
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"a").unwrap();
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.remove(id);
    assert_eq!(0, tracker.grapheme_count());
    drop(tracker);
}

#[test]
fn req_008_tracker_clear() {
    // ported: GraphemeTracker - clear removes all graphemes
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc(b"a").unwrap();
    let id2 = pool_ref.alloc(b"b").unwrap();
    drop(pool_ref);
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id1);
    tracker.add(id2);
    assert_eq!(2, tracker.grapheme_count());
    tracker.clear();
    assert_eq!(0, tracker.grapheme_count());
    assert!(!tracker.contains(id1));
    assert!(!tracker.contains(id2));
    assert!(!tracker.has_any());
    drop(tracker);
}

#[test]
fn req_008_tracker_total_bytes() {
    // ported: GraphemeTracker - getTotalGraphemeBytes
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc(b"a").unwrap();
    let id2 = pool_ref.alloc("🌟".as_bytes()).unwrap();
    let id3 = pool_ref.alloc(b"test").unwrap();
    drop(pool_ref);
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id1);
    tracker.add(id2);
    tracker.add(id3);
    assert_eq!(1 + 4 + 4, tracker.total_bytes());
    drop(tracker);
}

#[test]
fn req_008_tracker_keeps_alive() {
    // ported: GraphemeTracker - tracker keeps graphemes alive
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"test").unwrap();
    {
        let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
        tracker.add(id);
        assert_eq!(b"test", pool.borrow().get(id).unwrap());
    }
    let _ = pool.borrow_mut().alloc(b"x").unwrap();
    assert_eq!(
        Err(GraphemePoolError::WrongGeneration),
        pool.borrow().get(id)
    );
}

#[test]
fn req_008_tracker_shared_grapheme() {
    // ported: GraphemeTracker - multiple trackers share same grapheme
    let pool = shared_pool();
    let id = pool.borrow_mut().alloc(b"shared").unwrap();
    {
        let mut tracker1 = GraphemeTracker::new(Rc::clone(&pool));
        {
            let mut tracker2 = GraphemeTracker::new(Rc::clone(&pool));
            tracker1.add(id);
            tracker2.add(id);
            assert!(tracker1.contains(id));
            assert!(tracker2.contains(id));
            assert_eq!(b"shared", pool.borrow().get(id).unwrap());
        }
        assert_eq!(b"shared", pool.borrow().get(id).unwrap());
    }
    let _ = pool.borrow_mut().alloc(b"y").unwrap();
    assert_eq!(
        Err(GraphemePoolError::WrongGeneration),
        pool.borrow().get(id)
    );
}

#[test]
fn req_008_tracker_stress() {
    // ported: GraphemeTracker - stress test many graphemes
    let pool = shared_pool();
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    let mut ids = Vec::with_capacity(500);
    for i in 0..500u32 {
        let text = format!("{i}");
        let id = pool.borrow_mut().alloc(text.as_bytes()).unwrap();
        tracker.add(id);
        ids.push(id);
    }
    assert_eq!(500, tracker.grapheme_count());
    for id in &ids {
        assert!(tracker.contains(*id));
    }
    tracker.clear();
    assert_eq!(0, tracker.grapheme_count());
    for id in &ids {
        assert!(!tracker.contains(*id));
    }
    drop(tracker);
}

#[test]
fn req_008_tracker_small_pool() {
    // ported: GraphemePool - tracker with small pool
    let pool = Rc::new(RefCell::new(GraphemePool::with_options(InitOptions {
        slots_per_page: Some([3, 3, 3, 3, 3]),
    })));
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc("🌟".as_bytes()).unwrap();
    let id2 = pool_ref.alloc("🎨".as_bytes()).unwrap();
    let id3 = pool_ref.alloc("🚀".as_bytes()).unwrap();
    drop(pool_ref);
    tracker.add(id1);
    tracker.add(id2);
    tracker.add(id3);
    assert_eq!(3, tracker.grapheme_count());
    tracker.clear();
    assert_eq!(0, tracker.grapheme_count());
    drop(tracker);
}

#[test]
fn req_008_tracker_unowned() {
    // ported: GraphemeTracker - with unowned allocations
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let id1 = pool_ref.alloc_unowned(b"external1").unwrap();
    let id2 = pool_ref.alloc_unowned(b"external2").unwrap();
    drop(pool_ref);
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(id1);
    tracker.add(id2);
    assert_eq!(2, tracker.grapheme_count());
    assert!(tracker.contains(id1));
    assert!(tracker.contains(id2));
    assert_eq!(b"external1", pool.borrow().get(id1).unwrap());
    assert_eq!(b"external2", pool.borrow().get(id2).unwrap());
    drop(tracker);
}

#[test]
fn req_008_tracker_mixed() {
    // ported: GraphemeTracker - mix owned and unowned
    let pool = shared_pool();
    let mut pool_ref = pool.borrow_mut();
    let owned_id = pool_ref.alloc(b"owned_data").unwrap();
    let unowned_id = pool_ref.alloc_unowned(b"external_data").unwrap();
    drop(pool_ref);
    let mut tracker = GraphemeTracker::new(Rc::clone(&pool));
    tracker.add(owned_id);
    tracker.add(unowned_id);
    assert_eq!(2, tracker.grapheme_count());
    assert_eq!(
        b"owned_data".len() + b"external_data".len(),
        tracker.total_bytes() as usize
    );
    assert_eq!(b"owned_data", pool.borrow().get(owned_id).unwrap());
    assert_eq!(b"external_data", pool.borrow().get(unowned_id).unwrap());
    drop(tracker);
}

// ---- packed-char vectors (same API the pool ids feed into) ----

#[test]
fn req_008_packing_image_cell() {
    // ported: image cell markers use the unused character tag
    let marker = pack_image_cell(12345, 9);
    assert!(is_image_char(marker));
    assert!(!is_grapheme_char(marker));
    assert!(!is_continuation_char(marker));
    assert_eq!(12345, image_id_from_char(marker));
    assert_eq!(9, image_fallback_from_char(marker));
}

#[test]
fn req_008_packing_bit_fns() {
    // ported: GraphemePool - bit manipulation functions
    let grapheme_char = CHAR_FLAG_GRAPHEME | 0x1234;
    assert!(is_grapheme_char(grapheme_char));
    assert!(!is_grapheme_char(0x41));
    let cont_char = CHAR_FLAG_CONTINUATION | 0x1234;
    assert!(is_continuation_char(cont_char));
    assert!(!is_continuation_char(0x41));
    assert!(is_cluster_char(grapheme_char));
    assert!(is_cluster_char(cont_char));
    assert!(!is_cluster_char(0x41));
    let packed_char = CHAR_FLAG_GRAPHEME | 0x12345;
    assert_eq!(0x12345, grapheme_id_from_char(packed_char));
}

#[test]
fn req_008_packing_extents() {
    // ported: GraphemePool - extent encoding and decoding
    let char_with_right = (2 << CHAR_EXT_RIGHT_SHIFT) | CHAR_FLAG_GRAPHEME;
    assert_eq!(2, char_right_extent(char_with_right));
    let char_with_left = (1 << 26) | CHAR_FLAG_GRAPHEME;
    assert_eq!(1, char_left_extent(char_with_left));
}

#[test]
fn req_008_packing_grapheme_start() {
    // ported: GraphemePool - packGraphemeStart
    let packed_char = pack_grapheme_start(0x1234, 2);
    assert!(is_grapheme_char(packed_char));
    assert_eq!(0x1234, grapheme_id_from_char(packed_char));
    assert_eq!(1, char_right_extent(packed_char));
    assert_eq!(0, char_left_extent(packed_char));
}

#[test]
fn req_008_packing_grapheme_start_saturates() {
    // ported: GraphemePool - packGraphemeStart saturates wider wcwidth clusters
    let packed_char = pack_grapheme_start(0x1234, 8);
    assert_eq!(CHAR_EXT_MASK, char_right_extent(packed_char));
    assert_eq!(CHAR_EXT_MASK + 1, encoded_char_width(packed_char));
}

#[test]
fn req_008_packing_continuation() {
    // ported: GraphemePool - packContinuation
    let packed_char = pack_continuation(1, 2, 0x1234);
    assert!(is_continuation_char(packed_char));
    assert_eq!(0x1234, grapheme_id_from_char(packed_char));
    assert_eq!(1, char_left_extent(packed_char));
    assert_eq!(2, char_right_extent(packed_char));
}

#[test]
fn req_008_packing_encoded_width() {
    // ported: GraphemePool - encodedCharWidth
    assert_eq!(1, encoded_char_width(u32::from(b'A')));
    assert_eq!(2, encoded_char_width(pack_grapheme_start(0x1234, 2)));
    assert_eq!(3, encoded_char_width(pack_continuation(1, 1, 0x1234)));
}

#[test]
fn req_008_pool_free_unreferenced() {
    // hand-ported: freeUnreferenced releases a never-incref'd slot.
    let mut pool = GraphemePool::new();
    let id = pool.alloc(b"transient").unwrap();
    pool.free_unreferenced(id).unwrap();
    assert_eq!(Err(GraphemePoolError::InvalidId), pool.get(id));
    // The freed slot is reusable with a fresh generation.
    let id2 = pool.alloc(b"reused").unwrap();
    pool.incref(id2).unwrap();
    assert_eq!(b"reused", pool.get(id2).unwrap());
    pool.decref(id2).unwrap();
}

#[test]
#[should_panic(expected = "slots_per_page must be non-zero")]
fn req_008_pool_zero_slots_panics() {
    // hand-ported: zero slots_per_page is a programming error (reference panics).
    let _ = GraphemePool::with_options(InitOptions {
        slots_per_page: Some([1, 0, 1, 1, 1]),
    });
}
