//! render-alloc mechanism: RAS-003 and RAS-007. A counting global allocator
//! observes the render path; the hit grid is allocated on its first write
//! and a rendered frame leaves the next buffer for the painter to clear
//! (docs/spec/rasterizer.md).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use suprtui::ansi::{self, TextAttributes};
use suprtui::render::{Backend, RenderStatus, Renderer, WriteStatus};
use suprtui::uni::pool::GraphemePool;

struct Counting;

// Counted per thread: the test harness runs these tests in parallel, and a
// process-wide count would charge one test's allocations to another's
// render. `render` with the `Sink` backend runs on the calling thread.
thread_local! {
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn count() {
    // `try_with` fails only while the thread is being torn down.
    let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        count();
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

/// Allocations made on this thread so far.
fn allocations() -> usize {
    ALLOCATIONS.with(Cell::get)
}

/// A backend whose frame buffer is reserved once and reused.
struct Sink {
    bytes: Vec<u8>,
}

impl Backend for Sink {
    fn prepare_frame(&mut self) -> WriteStatus {
        WriteStatus::Ok
    }
    fn begin_frame(&mut self) {
        self.bytes.clear();
    }
    fn write_bytes(&mut self, data: &[u8]) {
        self.bytes.extend_from_slice(data);
    }
    fn write_out(&mut self, _: &[u8]) {}
    fn fail_frame(&mut self) {}
    fn end_frame(&mut self) -> WriteStatus {
        WriteStatus::Ok
    }
}

fn renderer(width: u32, height: u32) -> Renderer<'static, Sink> {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    Renderer::new(
        width,
        height,
        pool,
        Sink {
            bytes: Vec::with_capacity(1 << 22),
        },
    )
    .unwrap()
}

fn fill(r: &mut Renderer<'static, Sink>, width: u32, height: u32, seed: u8) {
    let line: String = (0..width)
        .map(|x| char::from(b'a' + ((x as u8).wrapping_add(seed)) % 26))
        .collect();
    for y in 0..height {
        r.next_buffer()
            .draw_text(
                &line,
                0,
                y,
                ansi::rgb_color(200, 200, 200, 255),
                Some(ansi::rgb_color(0, 0, 40, 255)),
                if y % 2 == 0 { 0 } else { 1 },
            )
            .unwrap();
    }
}

/// RAS-003: once the frame buffer has reached capacity, rendering a
/// 10,000-cell frame allocates nothing.
#[test]
fn ras_003_render_allocates_nothing_once_the_frame_buffer_has_capacity() {
    let mut r = renderer(100, 100);
    fill(&mut r, 100, 100, 0);
    assert_eq!(RenderStatus::Rendered, r.render(true));
    fill(&mut r, 100, 100, 1);
    assert_eq!(RenderStatus::Rendered, r.render(true));

    fill(&mut r, 100, 100, 2);
    let before = allocations();
    let status = r.render(true);
    let during = allocations() - before;
    assert_eq!(RenderStatus::Rendered, status);
    assert_eq!(
        0, during,
        "render of a 10,000-cell frame allocated {during} times"
    );

    fill(&mut r, 100, 100, 3);
    let before = allocations();
    let status = r.render(false);
    let during = allocations() - before;
    assert_eq!(RenderStatus::Rendered, status);
    assert_eq!(0, during, "diffed render allocated {during} times");
}

/// Draw a new foreground and background color in each of 10,000 cells.
/// The frame costs more bytes per cell than any frame drawn after it.
fn fill_colors(r: &mut Renderer<'static, Sink>) {
    for y in 0..100u32 {
        for x in 0..100u32 {
            let (a, b) = (x as u8, y as u8);
            r.next_buffer()
                .draw_text(
                    "c",
                    x,
                    y,
                    ansi::rgb_color(a, b, 200, 255),
                    Some(ansi::rgb_color(b, a, 40, 255)),
                    0,
                )
                .unwrap();
        }
    }
}

/// Draw 10,000 grapheme clusters, a Latin letter plus a combining mark,
/// every one distinct: each lives in the grapheme pool, not in the cell.
/// Each `set` uses its own 30 marks, so no two sets share a cluster.
fn fill_clusters(r: &mut Renderer<'static, Sink>, set: u32) {
    for y in 0..100u32 {
        let line: String = (0..100u32)
            .flat_map(|x| {
                let i = y * 100 + x;
                [
                    char::from_u32(0x0100 + i % 336).unwrap(),
                    char::from_u32(0x0300 + set * 30 + i / 336).unwrap(),
                ]
            })
            .collect();
        r.next_buffer()
            .draw_text(
                &line,
                0,
                y,
                ansi::rgb_color(200, 200, 200, 255),
                Some(ansi::rgb_color(0, 0, 40, 255)),
                0,
            )
            .unwrap();
    }
}

/// Link each of 10,000 cells to its own URL in the renderer's link pool.
fn fill_links(r: &mut Renderer<'static, Sink>, set: u32) {
    for y in 0..100u32 {
        for x in 0..100u32 {
            let url = format!("https://example.com/{set}/{y}/{x}");
            let buffer = r.next_buffer();
            let id = buffer.link_pool.borrow_mut().alloc(url.as_bytes()).unwrap();
            buffer
                .draw_text(
                    "l",
                    x,
                    y,
                    ansi::rgb_color(200, 200, 200, 255),
                    Some(ansi::rgb_color(0, 0, 40, 255)),
                    TextAttributes::set_link_id(0, id),
                )
                .unwrap();
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Frame {
    Text,
    Clusters(u32),
    Links(u32),
}

/// RAS-003 for text held in the grapheme and link pools. Only the frame
/// buffer is grown first; after that no render allocates: not the first
/// frame of 10,000 distinct clusters or links, not the frame that releases
/// them, and not any later frame that swaps one set for another. Releasing
/// a last reference must not copy its bytes, and the trackers `render`
/// updates must not grow.
#[test]
fn ras_003_frames_of_clusters_and_links_allocate_nothing() {
    let mut r = renderer(100, 100);
    fill_colors(&mut r);
    assert_eq!(RenderStatus::Rendered, r.render(true));

    let cycle = [
        Frame::Clusters(0),
        Frame::Text,
        Frame::Clusters(1),
        Frame::Clusters(2),
        Frame::Links(0),
        Frame::Clusters(0),
        Frame::Links(1),
        Frame::Links(2),
        Frame::Text,
    ];
    for round in 0..6 {
        for (k, frame) in cycle.into_iter().enumerate() {
            r.next_buffer().clear(ansi::rgb_color(0, 0, 0, 255), None);
            match frame {
                Frame::Text => fill(&mut r, 100, 100, 0),
                Frame::Clusters(set) => fill_clusters(&mut r, set),
                Frame::Links(set) => fill_links(&mut r, set + 3 * round),
            }
            let before = allocations();
            let status = r.render(false);
            let during = allocations() - before;
            assert_eq!(RenderStatus::Rendered, status, "frame {k}, {frame:?}");
            assert_eq!(
                0, during,
                "frame {k}, {frame:?}, in round {round} allocated {during} times"
            );
        }
    }
}

/// RAS-007: a renderer that never received a hit write holds no grid; the
/// first write allocates it; a rendered frame does not clear the next
/// buffer, which the painter clears once before the next frame.
#[test]
fn ras_007_hit_grid_is_lazy_and_the_next_buffer_is_cleared_once() {
    let mut r = renderer(40, 10);
    fill(&mut r, 40, 10, 0);
    assert_eq!(RenderStatus::Rendered, r.render(true));
    assert!(!r.hit_grid_allocated(), "no hit write, yet a grid exists");
    assert_eq!(0, r.check_hit(1, 1));

    // The renderer never clears the next buffer; the painter does, once
    // per frame, before it paints.
    let clears = r.next_buffer().clear_count();
    fill(&mut r, 40, 10, 2);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    assert_eq!(
        clears,
        r.next_buffer().clear_count(),
        "render cleared the next buffer"
    );

    let before = allocations();
    r.add_to_hit_grid(0, 0, 4, 2, 7);
    assert!(
        allocations() > before,
        "the first hit write must allocate the grid"
    );
    assert!(r.hit_grid_allocated());
    fill(&mut r, 40, 10, 1);
    assert_eq!(RenderStatus::Rendered, r.render(false));
    assert_eq!(7, r.check_hit(1, 1));

    // The rendered frame stays in the next buffer: the renderer did not
    // clear it, so an identical redraw is a skipped frame either way and a
    // partial redraw over it leaves the old cells in place.
    let kept = r.next_buffer().get(3, 3).unwrap();
    assert_ne!(
        u32::from(b' '),
        kept.char,
        "next buffer was cleared after the render"
    );
    fill(&mut r, 40, 10, 1);
    assert_eq!(RenderStatus::Skipped, r.render(false));
}
