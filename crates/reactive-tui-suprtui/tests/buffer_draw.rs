//! Ported drawing vectors for `buffer.zig` (BUF-008 … BUF-013, `buffer-draw`).
//!
//! `drawTextBuffer`, `drawImage`, and renderer-driven cases stay with the
//! text-view, media-image, and render commitments that own their inputs;
//! everything here runs on `set`, `draw_text`, fills, and compositing.
//! Failing-allocator halves of the no-allocate copies have no stable-Rust
//! equivalent, so those ports keep the copy assertions only.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::ansi::{
    ColorIntent, TextAttributes, alpha, blue, default_color, green, indexed_color, intent,
    pack_meta, pack_rgba8, red, rgb_color, rgba_from_floats, slot,
};
use suprtui::buffer::{ImagePlacement, ImageProtocol, InitOptions, OptimizedBuffer, make_cell};
use suprtui::link::LinkPool;
use suprtui::uni::pool::{GraphemePool, InitOptions as PoolOptions};
use suprtui::uni::segments::{
    GRAPHEME_ID_MASK, grapheme_id_from_char, is_continuation_char, is_grapheme_char,
    pack_grapheme_start,
};

fn pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::new()))
}

fn tiny_pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::with_options(PoolOptions {
        slots_per_page: Some([3, 3, 3, 3, 3]),
    })))
}

fn buffer(pool: &Rc<RefCell<GraphemePool<'static>>>, w: u32, h: u32) -> OptimizedBuffer<'static> {
    OptimizedBuffer::new(w, h, InitOptions::new(Rc::clone(pool))).unwrap()
}

fn buffer_with_link(
    pool: &Rc<RefCell<GraphemePool<'static>>>,
    link_pool: &Rc<RefCell<LinkPool>>,
    w: u32,
    h: u32,
    id: &str,
) -> OptimizedBuffer<'static> {
    OptimizedBuffer::new(
        w,
        h,
        InitOptions {
            pool: Rc::clone(pool),
            link_pool: Some(Rc::clone(link_pool)),
            id: id.to_string(),
            ..InitOptions::new(Rc::clone(pool))
        },
    )
    .unwrap()
}

// ---- BUF-008 blending ----

#[test]
fn req_008_blend_transparent_destination() {
    // ported: OptimizedBuffer - blendColors with transparent destination
    let p = pool();
    let mut buf = buffer(&p, 2, 2);
    let transparent_bg = rgba_from_floats(0.0, 0.0, 0.0, 0.0);
    buf.clear(transparent_bg, None);

    let semi_white = rgba_from_floats(1.0, 1.0, 1.0, 0.5);
    let transparent_fg = rgba_from_floats(0.0, 0.0, 0.0, 0.0);
    buf.set_cell_with_alpha_blending(0, 0, u32::from(b'X'), semi_white, transparent_fg, 0);

    let cell = buf.get(0, 0).unwrap();
    assert_eq!(255, red(cell.fg));
    assert_eq!(255, green(cell.fg));
    assert_eq!(255, blue(cell.fg));
    assert_eq!(128, alpha(cell.fg));
}

#[test]
fn req_008_blend_backdrop_flattens() {
    // ported: OptimizedBuffer - blend backdrop flattens transparent destination
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut options = InitOptions::new(Rc::clone(&p));
    options.blend_backdrop = Some(rgba_from_floats(1.0, 1.0, 1.0, 1.0));
    let mut buf = OptimizedBuffer::new(2, 2, options).unwrap();

    let transparent_bg = rgba_from_floats(0.0, 0.0, 0.0, 0.0);
    buf.clear(transparent_bg, None);
    drop(link_pool);

    let opaque_fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    let semi_black_bg = rgba_from_floats(0.0, 0.0, 0.0, 0.5);
    buf.set_cell_with_alpha_blending(0, 0, 0x20, opaque_fg, semi_black_bg, 0);

    let cell = buf.get(0, 0).unwrap();
    assert_eq!(127, red(cell.bg));
    assert_eq!(127, green(cell.bg));
    assert_eq!(127, blue(cell.bg));
    assert_eq!(255, alpha(cell.bg));
}

#[test]
fn req_008_blend_preserves_overlay_link() {
    // ported: OptimizedBuffer - alpha blending preserves overlay link not dest link
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut buf = buffer_with_link(&p, &link_pool, 20, 5, "test-buffer");
    let bg_opaque = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let bg_alpha = rgba_from_floats(0.5, 0.5, 0.5, 0.5);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg_opaque, None);

    let link_id_a = link_pool
        .borrow_mut()
        .alloc(b"https://underlying.com")
        .unwrap();
    let attr_a = TextAttributes::set_link_id(u32::from(TextAttributes::BOLD), link_id_a);
    buf.draw_text("X", 5, 0, fg, Some(bg_opaque), attr_a)
        .unwrap();

    let dest_cell = buf.get(5, 0).unwrap();
    assert_eq!(link_id_a, TextAttributes::link_id(dest_cell.attributes));
    assert_eq!(u32::from(b'X'), dest_cell.char);

    let link_id_b = link_pool
        .borrow_mut()
        .alloc(b"https://overlay.com")
        .unwrap();
    let attr_b = TextAttributes::set_link_id(0, link_id_b);
    buf.draw_text(" ", 5, 0, fg, Some(bg_alpha), attr_b)
        .unwrap();

    let result_cell = buf.get(5, 0).unwrap();
    assert_eq!(u32::from(b'X'), result_cell.char);
    assert_eq!(link_id_b, TextAttributes::link_id(result_cell.attributes));
    assert_ne!(link_id_a, TextAttributes::link_id(result_cell.attributes));
}

#[test]
fn req_008_blend_no_link_clears() {
    // ported: OptimizedBuffer - alpha blending with no link clears underlying link
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut buf = buffer_with_link(&p, &link_pool, 20, 5, "test-buffer");
    let bg_opaque = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let bg_alpha = rgba_from_floats(0.5, 0.5, 0.5, 0.5);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.clear(bg_opaque, None);

    let link_id = link_pool
        .borrow_mut()
        .alloc(b"https://underlying.com")
        .unwrap();
    let attr_link = TextAttributes::set_link_id(u32::from(TextAttributes::BOLD), link_id);
    buf.draw_text("X", 5, 0, fg, Some(bg_opaque), attr_link)
        .unwrap();

    let dest_cell = buf.get(5, 0).unwrap();
    assert_eq!(link_id, TextAttributes::link_id(dest_cell.attributes));

    buf.draw_text(" ", 5, 0, fg, Some(bg_alpha), 0).unwrap();

    let result_cell = buf.get(5, 0).unwrap();
    assert_eq!(u32::from(b'X'), result_cell.char);
    assert_eq!(0, TextAttributes::link_id(result_cell.attributes));
    assert!(!TextAttributes::has_link(result_cell.attributes));
}

#[test]
fn req_008_blend_downgrades_metadata() {
    // ported: OptimizedBuffer - alpha blending downgrades blended metadata to rgb
    let p = pool();
    let mut buf = buffer(&p, 4, 1);
    let base_bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    buf.clear(base_bg, None);

    buf.set_cell_with_alpha_blending(
        0,
        0,
        u32::from(b'B'),
        pack_rgba8(255, 0, 0, 128, pack_meta(ColorIntent::Indexed, 3)),
        base_bg,
        0,
    );

    let fg_blended_cell = buf.get(0, 0).unwrap();
    assert_eq!(ColorIntent::Rgb, intent(fg_blended_cell.fg));
    assert_eq!(ColorIntent::Rgb, intent(fg_blended_cell.bg));

    buf.set(
        0,
        0,
        make_cell(
            u32::from(b'A'),
            indexed_color(1, 255, 255, 255),
            indexed_color(2, 0, 0, 0),
            0,
        ),
    );

    buf.set_cell_with_alpha_blending(
        0,
        0,
        u32::from(b'C'),
        indexed_color(5, 255, 0, 0),
        pack_rgba8(0, 255, 0, 128, pack_meta(ColorIntent::Indexed, 6)),
        0,
    );

    let bg_blended_cell = buf.get(0, 0).unwrap();
    assert_eq!(ColorIntent::Indexed, intent(bg_blended_cell.fg));
    assert_eq!(5, slot(bg_blended_cell.fg));
    assert_eq!(ColorIntent::Rgb, intent(bg_blended_cell.bg));
}

// ---- BUF-009 scissor ----

#[test]
fn req_009_draw_text_alpha_scissor() {
    // ported: OptimizedBuffer - drawText with alpha blending and scissor
    let tiny = tiny_pool();
    let mut buf = OptimizedBuffer::new(
        80,
        25,
        InitOptions {
            pool: Rc::clone(&tiny),
            id: "test-buffer".to_string(),
            ..InitOptions::new(Rc::clone(&tiny))
        },
    )
    .unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    let bg_alpha = rgba_from_floats(0.0, 0.0, 0.0, 0.5);
    buf.clear(bg, None);
    buf.push_scissor_rect(0, 0, 10, 10);
    for _ in 0..200 {
        buf.draw_text("• • • •", 50, 0, fg, Some(bg_alpha), 0)
            .unwrap();
    }
    // Fully clipped: the grid keeps only cleared cells.
    assert_eq!(0x20, buf.get(50, 0).unwrap().char);
    assert_eq!(0x20, buf.get(0, 0).unwrap().char);
}

#[test]
fn req_009_fill_rect_clipped() {
    // hand-ported: negative-origin fills clip to the grid.
    let p = pool();
    let mut buf = buffer(&p, 6, 4);
    let bg = rgb_color(0, 0, 0, 255);
    buf.clear(bg, None);
    let fill = rgb_color(9, 9, 9, 255);
    buf.fill_rect_clipped(-2, -1, 5, 3, fill);
    assert_eq!(fill, buf.get(0, 0).unwrap().bg);
    assert_eq!(fill, buf.get(2, 1).unwrap().bg);
    assert_eq!(bg, buf.get(3, 0).unwrap().bg);
    assert_eq!(bg, buf.get(0, 2).unwrap().bg);
}

// ---- BUF-010 wide cells ----

#[test]
fn req_010_same_id_extents_keep_slot() {
    // ported: buffer - set same grapheme ID with different extents keeps slot alive
    let tiny = Rc::new(RefCell::new(GraphemePool::with_options(PoolOptions {
        slots_per_page: Some([1, 1, 1, 1, 1]),
    })));
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut buf = buffer_with_link(&tiny, &link_pool, 10, 2, "extents");
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);

    let emoji = "👋";
    let gid = tiny.borrow_mut().alloc(emoji.as_bytes()).unwrap();
    let packed_w2 = pack_grapheme_start(gid & GRAPHEME_ID_MASK, 2);
    buf.set(0, 0, make_cell(packed_w2, fg, bg, 0));

    let id_from_char = grapheme_id_from_char(packed_w2);
    assert!(buf.grapheme_tracker.contains(id_from_char));

    let packed_w1 = pack_grapheme_start(gid & GRAPHEME_ID_MASK, 1);
    buf.set(0, 0, make_cell(packed_w1, fg, bg, 0));

    assert!(buf.grapheme_tracker.contains(id_from_char));
    assert_eq!(emoji.as_bytes(), tiny.borrow().get(gid).unwrap());
}

#[test]
fn req_010_draw_text_stress_no_exhaustion() {
    // ported: repeated mixed drawText frames with a tiny pool (clear releases refs)
    let tiny = tiny_pool();
    let mut buf = OptimizedBuffer::new(
        40,
        5,
        InitOptions {
            pool: Rc::clone(&tiny),
            id: "test-buffer".to_string(),
            ..InitOptions::new(Rc::clone(&tiny))
        },
    )
    .unwrap();
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    for _ in 0..500 {
        buf.clear(bg, None);
        buf.draw_text("A🌟B🎨C🚀D", 0, 0, fg, Some(bg), 0).unwrap();
        buf.draw_text("测试文字处理", 0, 1, fg, Some(bg), 0)
            .unwrap();
        buf.draw_text("Hello World!", 0, 2, fg, Some(bg), 0)
            .unwrap();
    }
    assert_eq!(u32::from(b'A'), buf.get(0, 0).unwrap().char);
}

#[test]
fn req_010_overwrite_graphemes_repeatedly() {
    // ported: overwriting graphemes repeatedly (same slot churn)
    let tiny = tiny_pool();
    let mut buf = buffer(&tiny, 20, 5);
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    for _ in 0..1000 {
        buf.draw_text("🌟", 0, 0, fg, Some(bg), 0).unwrap();
        buf.draw_text("🎨", 0, 0, fg, Some(bg), 0).unwrap();
        buf.draw_text("🚀", 0, 0, fg, Some(bg), 0).unwrap();
    }
    let cell = buf.get(0, 0).unwrap();
    assert!(is_grapheme_char(cell.char));
    assert_eq!(
        "🚀".as_bytes(),
        tiny.borrow().get(grapheme_id_from_char(cell.char)).unwrap()
    );
}

#[test]
fn req_010_draw_positions_mixed_runs() {
    // ported: drawText at scattered positions with mixed runs
    let p = pool();
    let mut buf = buffer(&p, 20, 3);
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    let fg = rgba_from_floats(1.0, 1.0, 1.0, 1.0);
    buf.draw_text("🌟🎨🚀", 0, 0, fg, Some(bg), 0).unwrap();
    buf.draw_text("🍕🍔🍟", 5, 1, fg, Some(bg), 0).unwrap();
    let c0 = buf.get(0, 0).unwrap();
    assert!(is_grapheme_char(c0.char));
    assert_eq!(
        "🌟".as_bytes(),
        p.borrow().get(grapheme_id_from_char(c0.char)).unwrap()
    );
    let c1 = buf.get(1, 0).unwrap();
    assert!(is_continuation_char(c1.char));
    assert_eq!(
        grapheme_id_from_char(c0.char),
        grapheme_id_from_char(c1.char)
    );
}

// ---- BUF-011 ANSI ----

#[test]
fn req_011_grapheme_and_image_emission() {
    // hand-ported: grapheme cells resolve bytes, image cells fall back,
    // continuations emit nothing.
    let p = pool();
    let mut buf = buffer(&p, 4, 1);
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    buf.clear(bg, None);
    buf.draw_text("é", 0, 0, fg, Some(bg), 0).unwrap();
    let start = buf.get(0, 0).unwrap();
    assert!(is_grapheme_char(start.char));
    let seq = buf.cell_ansi(0, 0, true, true).unwrap();
    assert!(seq.ends_with("é"), "{seq:?}");

    // Image marker with fallback nibble 0xF renders the full block.
    buf.set(
        2,
        0,
        make_cell(suprtui::uni::segments::pack_image_cell(3, 0xF), fg, bg, 0),
    );
    let img = buf.cell_ansi(2, 0, true, true).unwrap();
    assert!(img.ends_with('█'), "{img:?}");
}

// ---- BUF-012 compositing ----

#[test]
fn req_012_image_free_copy() {
    // ported: image-free frame buffer copy (allocation half has no
    // stable-Rust equivalent; the copy assertions remain)
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut source = buffer_with_link(&p, &link_pool, 1, 1, "source");
    let mut target = buffer_with_link(&p, &link_pool, 1, 1, "target");
    source.set(
        0,
        0,
        make_cell(
            u32::from(b'X'),
            rgb_color(1, 2, 3, 255),
            rgb_color(4, 5, 6, 255),
            7,
        ),
    );
    target.draw_frame_buffer(0, 0, &source, None, None, None, None);
    assert_eq!(u32::from(b'X'), target.get(0, 0).unwrap().char);
}

#[test]
fn req_012_image_free_alpha_copy() {
    // ported: image-free alpha frame buffer copy (respectAlpha source)
    let p = pool();
    let link_pool = Rc::new(RefCell::new(LinkPool::new()));
    let mut src_options = InitOptions::new(Rc::clone(&p));
    src_options.respect_alpha = true;
    let mut source = OptimizedBuffer::new(1, 1, src_options).unwrap();
    let mut target = buffer_with_link(&p, &link_pool, 1, 1, "target");
    source.set(
        0,
        0,
        make_cell(
            u32::from(b'X'),
            rgb_color(1, 2, 3, 255),
            rgb_color(4, 5, 6, 255),
            7,
        ),
    );
    target.draw_frame_buffer(0, 0, &source, None, None, None, None);
    assert_eq!(u32::from(b'X'), target.get(0, 0).unwrap().char);
}

#[test]
fn req_012_opaque_copy_preserves_metadata() {
    // ported: drawFrameBuffer preserves packed metadata on opaque copy
    let p = pool();
    let mut src = buffer(&p, 2, 1);
    let mut dst = buffer(&p, 2, 1);
    let bg = rgba_from_floats(0.0, 0.0, 0.0, 1.0);
    src.clear(bg, None);
    dst.clear(bg, None);
    src.set(
        0,
        0,
        make_cell(
            u32::from(b'X'),
            default_color(255, 255, 255, 255),
            indexed_color(6, 0, 128, 128),
            0,
        ),
    );
    dst.draw_frame_buffer(0, 0, &src, None, None, None, None);
    let copied = dst.get(0, 0).unwrap();
    assert_eq!(ColorIntent::Default, intent(copied.fg));
    assert_eq!(ColorIntent::Indexed, intent(copied.bg));
    assert_eq!(6, slot(copied.bg));
}

#[test]
fn req_012_placement_geometry_merge() {
    // hand-ported: placement geometries clip and remap on composite
    // (pixel transfer stays with media-image).
    let p = pool();
    let mut src = buffer(&p, 6, 4);
    let mut dst = buffer(&p, 4, 4);
    let bg = rgb_color(0, 0, 0, 255);
    src.clear(bg, None);
    dst.clear(bg, None);
    src.push_placement(ImagePlacement {
        placement_id: 41,
        image_handle: 7,
        x: 1,
        y: 1,
        width: 4,
        height: 2,
        pixel_width: 40,
        pixel_height: 20,
        source_x: 0,
        source_y: 0,
        source_width: 40,
        source_height: 20,
        opacity: 255,
        protocol: ImageProtocol::Kitty,
    });
    dst.draw_frame_buffer(2, 0, &src, None, None, None, None);
    assert_eq!(1, dst.placements().len());
    let merged = dst.placements()[0];
    assert_eq!(1, merged.placement_id);
    assert_eq!(7, merged.image_handle);
    assert_eq!((3, 1), (merged.x, merged.y));
    assert_eq!((1, 2), (merged.width, merged.height));
    // Source placements are untouched.
    assert_eq!(1, src.placements().len());
}

#[test]
fn req_012_subregion_copy() {
    // hand-ported: explicit source windows composite exactly.
    let p = pool();
    let mut src = buffer(&p, 6, 4);
    let mut dst = buffer(&p, 6, 4);
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    src.clear(bg, None);
    dst.clear(bg, None);
    for (i, ch) in ["A", "B", "C", "D"].iter().enumerate() {
        src.set(
            i as u32,
            1,
            make_cell(ch.chars().next().unwrap() as u32, fg, bg, 0),
        );
    }
    dst.draw_frame_buffer(0, 0, &src, Some(1), Some(1), Some(2), Some(1));
    assert_eq!(u32::from(b'B'), dst.get(0, 0).unwrap().char);
    assert_eq!(u32::from(b'C'), dst.get(1, 0).unwrap().char);
    assert_eq!(0x20, dst.get(2, 0).unwrap().char);
}

// ---- BUF-013 second scenario ----

#[test]
fn req_013_same_pool_keeps_ids() {
    // hand-ported: same-pool composites copy ids raw (reference behavior).
    let p = pool();
    let mut src = buffer(&p, 4, 1);
    let mut dst = buffer(&p, 4, 1);
    let fg = rgb_color(255, 255, 255, 255);
    let bg = rgb_color(0, 0, 0, 255);
    src.clear(bg, None);
    dst.clear(bg, None);
    let gid = p.borrow_mut().alloc("🌟".as_bytes()).unwrap() & GRAPHEME_ID_MASK;
    src.set(0, 0, make_cell(pack_grapheme_start(gid, 1), fg, bg, 0));
    let src_char = src.get(0, 0).unwrap().char;
    dst.draw_frame_buffer(0, 0, &src, None, None, None, None);
    assert_eq!(src_char, dst.get(0, 0).unwrap().char);
    assert_eq!("🌟".as_bytes(), p.borrow().get(gid).unwrap());
}
