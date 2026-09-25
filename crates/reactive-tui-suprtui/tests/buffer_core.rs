//! Ported core vectors for `buffer.zig` (BUF-001 … BUF-007, `buffer-core`).
//!
//! One `#[test]` per portable reference case, named `req_00n_<slug>`.
//! Text-drawing cases use explicit `set` loops: `drawText` itself arrives
//! with the drawing commitment, and a per-cell `set` loop is exactly what
//! it lowers to for plain runs. Global pools are replaced with
//! caller-owned pools per UNI-008.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::ansi::{
    ColorIntent, TextAttributes, fallback_ansi256_color, indexed_color, intent, rgb_color,
    rgba_from_floats, slot,
};
use suprtui::buffer::{Cell, InitOptions, OptimizedBuffer, make_cell};
use suprtui::link::LinkPool;
use suprtui::uni::pool::{GraphemePool, InitOptions as PoolOptions};
use suprtui::uni::segments::{
    GRAPHEME_ID_MASK, grapheme_id_from_char, is_continuation_char, is_grapheme_char,
    pack_grapheme_start,
};

fn pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::new()))
}

fn buffer(pool: &Rc<RefCell<GraphemePool<'static>>>, w: u32, h: u32) -> OptimizedBuffer<'static> {
    OptimizedBuffer::new(w, h, InitOptions::new(Rc::clone(pool))).unwrap()
}

/// `drawText` for plain runs lowers to one `set` per cell; the drawing
/// commitment ports `drawText` itself.
fn set_text(
    buf: &mut OptimizedBuffer<'static>,
    text: &str,
    x: u32,
    y: u32,
    fg: suprtui::ansi::Rgba,
    bg: suprtui::ansi::Rgba,
    attributes: u32,
) {
    for (i, ch) in text.chars().enumerate() {
        buf.set(x + i as u32, y, make_cell(ch as u32, fg, bg, attributes));
    }
}

#[test]
fn req_001_color_intents() {
    // ported: packed RGBA stores metadata (ansi_test.zig)
    let c = indexed_color(9, 255, 0, 0);
    assert_eq!(ColorIntent::Indexed, intent(c));
    assert_eq!(9, slot(c));
    let rgb = rgb_color(10, 20, 30, 255);
    assert_eq!(ColorIntent::Rgb, intent(rgb));
}

#[test]
fn req_002_palette_regions() {
    // ported: fallbackAnsi256Color returns base, cube, and grayscale colors
    let base = fallback_ansi256_color(9);
    assert_eq!(
        (255, 0, 0),
        (
            suprtui::ansi::red(base),
            suprtui::ansi::green(base),
            suprtui::ansi::blue(base)
        )
    );
    let cube = fallback_ansi256_color(16 + 5 * 36 + 5 * 6 + 5);
    assert_eq!(
        (255, 255, 255),
        (
            suprtui::ansi::red(cube),
            suprtui::ansi::green(cube),
            suprtui::ansi::blue(cube)
        )
    );
    let gray = fallback_ansi256_color(232);
    assert_eq!(
        (8, 8, 8),
        (
            suprtui::ansi::red(gray),
            suprtui::ansi::green(gray),
            suprtui::ansi::blue(gray)
        )
    );
}

#[test]
fn req_003_buffer_init_dims() {
    // ported: OptimizedBuffer - init and deinit
    let p = pool();
    let buf = buffer(&p, 10, 10);
    assert_eq!(10, buf.width());
    assert_eq!(10, buf.height());
    drop(buf);
}

#[test]
fn req_003_set_get_roundtrip() {
    // hand-ported: scalar set/get across the grid keeps every field.
    let p = pool();
    let mut buf = buffer(&p, 6, 4);
    let fg = rgba_from_floats(1.0, 0.5, 0.0, 1.0);
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let attrs = u32::from(TextAttributes::BOLD | TextAttributes::UNDERLINE);
    for y in 0..4u32 {
        for x in 0..6u32 {
            buf.set(x, y, make_cell(0x41 + x + y, fg, bg, attrs));
        }
    }
    for y in 0..4u32 {
        for x in 0..6u32 {
            let cell: Cell = buf.get(x, y).unwrap();
            assert_eq!(0x41 + x + y, cell.char);
            assert_eq!(fg, cell.fg);
            assert_eq!(bg, cell.bg);
            assert_eq!(attrs, cell.attributes);
        }
    }
}

#[test]
fn req_003_link_id_preserves_flags() {
    // hand-ported: link-id writes preserve the style flags (BUF-003).
    let p = pool();
    let mut buf = buffer(&p, 4, 1);
    let flags = TextAttributes::DIM | TextAttributes::BLINK;
    let linked = TextAttributes::set_link_id(u32::from(flags), 0x123456);
    buf.set(
        0,
        0,
        make_cell(
            0x58,
            rgb_color(0, 0, 0, 255),
            rgb_color(0, 0, 0, 255),
            linked,
        ),
    );
    let back = buf.get(0, 0).unwrap();
    assert_eq!(flags, TextAttributes::base_attributes(back.attributes));
    assert_eq!(0x123456, TextAttributes::link_id(back.attributes));
}

#[test]
fn req_004_bounds_no_panic() {
    // hand-ported: OOB set/set_raw/get never panic; get is None outside.
    let p = pool();
    let mut buf = buffer(&p, 3, 2);
    let cell = make_cell(0x41, rgb_color(0, 0, 0, 255), rgb_color(0, 0, 0, 255), 0);
    for (x, y) in [(3, 0), (0, 2), (99, 99), (u32::MAX, 0), (0, u32::MAX)] {
        assert!(buf.get(x, y).is_none(), "{x},{y}");
        buf.set(x, y, cell);
        buf.set_raw(x, y, cell);
        buf.sync_cell(x, y, cell);
    }
    assert!(buf.get(2, 1).is_some());
}

#[test]
fn req_005_resize_zero_rejected() {
    // hand-ported: zero-size resizes fail without changing dims.
    let p = pool();
    let mut buf = buffer(&p, 5, 5);
    assert!(buf.resize(0, 5).is_err());
    assert!(buf.resize(5, 0).is_err());
    assert_eq!((5, 5), (buf.width(), buf.height()));
}

#[test]
fn req_006_resize_clears_all() {
    // hand-ported: shrink and grow both clear every cell.
    let p = pool();
    let mut buf = buffer(&p, 6, 6);
    let fg = rgb_color(7, 7, 7, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.set(0, 0, make_cell(0x41, fg, bg, 0xFF));
    buf.set(5, 5, make_cell(0x42, fg, bg, 0xFF));
    buf.resize(3, 3).unwrap();
    for y in 0..3 {
        for x in 0..3 {
            assert_eq!(0x20, buf.get(x, y).unwrap().char);
        }
    }
    buf.set(0, 0, make_cell(0x43, fg, bg, 0xFF));
    buf.resize(5, 5).unwrap();
    for y in 0..5 {
        for x in 0..5 {
            assert_eq!(0x20, buf.get(x, y).unwrap().char);
        }
    }
}

#[test]
fn req_007_clear_custom_char() {
    // hand-ported: clear with a caller char fills it everywhere.
    let p = pool();
    let mut buf = buffer(&p, 4, 3);
    let bg = rgb_color(1, 1, 1, 255);
    buf.clear(bg, Some(0x2E));
    for y in 0..3 {
        for x in 0..4 {
            let cell = buf.get(x, y).unwrap();
            assert_eq!(0x2E, cell.char);
            assert_eq!(bg, cell.bg);
            assert_eq!(rgb_color(255, 255, 255, 255), cell.fg);
        }
    }
}

#[test]
fn req_003_clear_fills_default() {
    // ported: OptimizedBuffer - clear fills with default char
    let p = pool();
    let mut buf = buffer(&p, 5, 5);
    buf.clear(rgba_from_floats(0.0, 0.0, 0.0, 1.0), None);
    for y in 0..5 {
        for x in 0..5 {
            assert_eq!(0x20, buf.get(x, y).unwrap().char);
        }
    }
}

#[test]
fn req_006_resize_grow_initialized() {
    // ported: OptimizedBuffer - cells are initialized after resize grow
    let p = pool();
    let mut buf = buffer(&p, 10, 10);
    buf.resize(20, 20).unwrap();
    let cell = buf.get(15, 15).unwrap();
    assert_eq!(0x20, cell.char);
}

#[test]
fn req_003_link_roundtrip() {
    // ported: OptimizedBuffer - link encoding round-trip (set-loop for drawText)
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut options = InitOptions::new(Rc::clone(&p));
    options.link_pool = Some(Rc::clone(&link_pool));
    let mut buf = OptimizedBuffer::new(20, 5, options).unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg, None);

    let link_id = link_pool
        .borrow_mut()
        .alloc(b"https://example.com")
        .unwrap();
    let attributes = TextAttributes::set_link_id(u32::from(TextAttributes::BOLD), link_id);
    set_text(&mut buf, "Click", 0, 0, fg, bg, attributes);

    let cell = buf.get(0, 0).unwrap();
    assert_eq!(u32::from(b'C'), cell.char);
    assert_eq!(
        TextAttributes::BOLD,
        TextAttributes::base_attributes(cell.attributes)
    );
    assert_eq!(link_id, TextAttributes::link_id(cell.attributes));
    assert!(buf.link_tracker.has_any());
    assert_eq!(1, buf.link_tracker.link_count());
}

#[test]
fn req_003_link_per_cell_counting() {
    // ported: OptimizedBuffer - link tracker per-cell counting
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut options = InitOptions::new(Rc::clone(&p));
    options.link_pool = Some(Rc::clone(&link_pool));
    let mut buf = OptimizedBuffer::new(20, 5, options).unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg, None);

    let link_id = link_pool
        .borrow_mut()
        .alloc(b"https://example.com")
        .unwrap();
    let attributes = TextAttributes::set_link_id(0, link_id);
    set_text(&mut buf, "ABC", 0, 0, fg, bg, attributes);

    assert_eq!(1, buf.link_tracker.link_count());
    assert_eq!(1, link_pool.borrow().get_refcount(link_id).unwrap());
    assert_eq!(Some(3), buf.link_tracker.cell_count(link_id));

    set_text(&mut buf, "X", 0, 0, fg, bg, 0);
    assert_eq!(Some(2), buf.link_tracker.cell_count(link_id));
    assert_eq!(1, link_pool.borrow().get_refcount(link_id).unwrap());
    // Release-on-clear is BUF-007's assertion (buffer::clear covers it);
    // this test owns counting only, so it must not depend on clear().
}

#[test]
fn req_007_set_clear_cycle_no_leak() {
    // ported: OptimizedBuffer - set and clear cycle should not leak
    // (set-loop for drawText)
    let tiny = Rc::new(RefCell::new(GraphemePool::with_options(PoolOptions {
        slots_per_page: Some([3, 3, 3, 3, 3]),
    })));
    let mut buf = OptimizedBuffer::new(10, 5, InitOptions::new(Rc::clone(&tiny))).unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    for _ in 0..200 {
        buf.set(0, 0, make_cell('•' as u32, fg, bg, 0));
        buf.clear(bg, None);
    }
    assert!(!buf.grapheme_tracker.has_any());
}

#[test]
fn req_003_adjacent_continuation_kept() {
    // ported: OptimizedBuffer - set should not clear newly written
    // adjacent grapheme continuation
    let tiny = Rc::new(RefCell::new(GraphemePool::with_options(
        PoolOptions::default(),
    )));
    let mut buf = OptimizedBuffer::new(
        8,
        1,
        InitOptions {
            pool: Rc::clone(&tiny),
            id: "set-adjacent-grapheme".to_string(),
            ..InitOptions::new(Rc::clone(&tiny))
        },
    )
    .unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg, None);

    let old_gid = tiny.borrow_mut().alloc("🌟".as_bytes()).unwrap();
    let old_start = pack_grapheme_start(old_gid & GRAPHEME_ID_MASK, 2);
    buf.set(3, 0, make_cell(old_start, fg, bg, 0));

    let new_gid = tiny.borrow_mut().alloc("🔥".as_bytes()).unwrap();
    let new_start = pack_grapheme_start(new_gid & GRAPHEME_ID_MASK, 2);

    buf.set(2, 0, make_cell(new_start, fg, bg, 0));
    buf.set(4, 0, make_cell(0x20, fg, bg, 0));

    let c2 = buf.get(2, 0).unwrap();
    let c3 = buf.get(3, 0).unwrap();
    let c4 = buf.get(4, 0).unwrap();
    assert!(is_grapheme_char(c2.char));
    assert_eq!(new_gid & GRAPHEME_ID_MASK, grapheme_id_from_char(c2.char));
    assert!(is_continuation_char(c3.char));
    assert_eq!(new_gid & GRAPHEME_ID_MASK, grapheme_id_from_char(c3.char));
    assert_eq!(0x20, c4.char);
}

#[test]
fn req_003_span_cleanup_link_consistency() {
    // ported: OptimizedBuffer - set span cleanup keeps shared link
    // refcounts consistent
    let tiny = Rc::new(RefCell::new(GraphemePool::with_options(
        PoolOptions::default(),
    )));
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut buf = OptimizedBuffer::new(
        10,
        1,
        InitOptions {
            pool: Rc::clone(&tiny),
            link_pool: Some(Rc::clone(&link_pool)),
            id: "set-span-link-refcount".to_string(),
            ..InitOptions::new(Rc::clone(&tiny))
        },
    )
    .unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg, None);

    let link_id = link_pool
        .borrow_mut()
        .alloc(b"https://example.com")
        .unwrap();
    let linked_attr = TextAttributes::set_link_id(0, link_id);
    let gid = tiny.borrow_mut().alloc("你".as_bytes()).unwrap();
    let start = pack_grapheme_start(gid & GRAPHEME_ID_MASK, 2);

    buf.set(2, 0, make_cell(start, fg, bg, linked_attr));
    buf.set(6, 0, make_cell(u32::from(b'X'), fg, bg, linked_attr));
    assert_eq!(Some(3), buf.link_tracker.cell_count(link_id));
    assert_eq!(1, link_pool.borrow().get_refcount(link_id).unwrap());

    buf.set(3, 0, make_cell(0x20, fg, bg, 0));
    assert_eq!(1, buf.link_tracker.link_count());
    assert_eq!(Some(1), buf.link_tracker.cell_count(link_id));
    assert_eq!(1, link_pool.borrow().get_refcount(link_id).unwrap());
}

#[test]
fn req_003_sync_cell_tracker_transitions() {
    // ported: OptimizedBuffer - syncCell updates grapheme tracker for
    // start transitions
    let tiny = Rc::new(RefCell::new(GraphemePool::with_options(
        PoolOptions::default(),
    )));
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut buf = OptimizedBuffer::new(
        10,
        1,
        InitOptions {
            pool: Rc::clone(&tiny),
            link_pool: Some(Rc::clone(&link_pool)),
            id: "sync-cell-grapheme-tracker".to_string(),
            ..InitOptions::new(Rc::clone(&tiny))
        },
    )
    .unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg, None);

    let gid_old = tiny.borrow_mut().alloc("你".as_bytes()).unwrap();
    let gid_new = tiny.borrow_mut().alloc("好".as_bytes()).unwrap();
    let old_id = gid_old & GRAPHEME_ID_MASK;
    let new_id = gid_new & GRAPHEME_ID_MASK;

    buf.sync_cell(1, 0, make_cell(pack_grapheme_start(old_id, 2), fg, bg, 0));
    assert_eq!(1, buf.grapheme_tracker.grapheme_count());
    assert!(buf.grapheme_tracker.contains(old_id));

    buf.sync_cell(1, 0, make_cell(pack_grapheme_start(new_id, 2), fg, bg, 0));
    assert_eq!(1, buf.grapheme_tracker.grapheme_count());
    assert!(!buf.grapheme_tracker.contains(old_id));
    assert!(buf.grapheme_tracker.contains(new_id));

    buf.sync_cell(1, 0, make_cell(0x20, fg, bg, 0));
    assert_eq!(0, buf.grapheme_tracker.grapheme_count());
    assert!(!buf.grapheme_tracker.contains(new_id));
}

#[test]
fn req_003_set_raw_skips_tracker() {
    // hand-ported: set_raw writes without grapheme tracking or cleanup.
    let p = pool();
    let mut buf = buffer(&p, 6, 1);
    let fg = rgb_color(0, 0, 0, 255);
    let bg = rgb_color(0, 0, 0, 255);
    let gid = p.borrow_mut().alloc("🌟".as_bytes()).unwrap();
    buf.set_raw(0, 0, make_cell(pack_grapheme_start(gid, 2), fg, bg, 0));
    assert_eq!(pack_grapheme_start(gid, 2), buf.get(0, 0).unwrap().char);
    assert!(!buf.grapheme_tracker.has_any());
    // No continuation propagation either.
    assert_eq!(0, buf.get(1, 0).unwrap().char);
}
