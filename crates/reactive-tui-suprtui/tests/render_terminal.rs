//! Integration tests for the render-terminal commitment (REN-006 …
//! REN-010). These exercise the public renderer API; the spec-literal
//! unit tests live in `src/render.rs`.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::ansi::rgb_color;
use suprtui::buffer::{ClipRect, ImagePlacement, ImageProtocol, make_cell};
use suprtui::render::{Backend, MemoryBackend, RenderStatus, Renderer, ThreadedBackend};
use suprtui::uni::pool::GraphemePool;
use suprtui::uni::segments::pack_image_cell;

fn make_renderer(width: u32, height: u32) -> Renderer<'static, MemoryBackend> {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    Renderer::new(width, height, pool, MemoryBackend::new()).unwrap()
}

fn draw<B: Backend>(renderer: &mut Renderer<'_, B>, text: &str, x: u32, y: u32) {
    renderer
        .next_buffer()
        .draw_text(
            text,
            x,
            y,
            rgb_color(255, 255, 255, 255),
            Some(rgb_color(0, 0, 0, 255)),
            0,
        )
        .unwrap();
}

fn output(renderer: &Renderer<'static, MemoryBackend>) -> Vec<u8> {
    renderer.backend().frames().concat()
}

#[test]
fn req_006_thread_parity() {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    let mut direct = Renderer::new(8, 3, Rc::clone(&pool), MemoryBackend::new()).unwrap();
    let mut threaded =
        Renderer::new(8, 3, pool, ThreadedBackend::new(MemoryBackend::new())).unwrap();
    for text in ["one", "two", "three"] {
        draw(&mut direct, text, 0, 0);
        draw(&mut threaded, text, 0, 0);
        assert_eq!(direct.render(false), threaded.render(false));
    }
    draw(&mut direct, "three", 0, 0);
    draw(&mut threaded, "three", 0, 0);
    assert_eq!(RenderStatus::Skipped, direct.render(false));
    assert_eq!(RenderStatus::Skipped, threaded.render(false));
    let threaded_backend = std::mem::replace(
        threaded.backend_mut(),
        ThreadedBackend::new(MemoryBackend::new()),
    );
    assert_eq!(
        direct.backend().frames(),
        threaded_backend.shutdown().frames()
    );
}

#[test]
fn req_007_lifecycle_sequences() {
    let mut renderer = make_renderer(8, 3);
    renderer.setup_terminal(true);
    renderer.shutdown();
    let out = renderer.backend().direct_output().to_vec();
    assert!(out.windows(8).any(|w| w == b"\x1b[?1049h"));
    assert!(out.windows(8).any(|w| w == b"\x1b[?1049l"));
    // Cursor state is restored visibly: style reset and shown.
    assert!(out.windows(5).any(|w| w == b"\x1b[0 q"));
    assert!(out.windows(6).any(|w| w == b"\x1b[?25h"));

    // No setup, no sequences: suspend and shutdown stay silent.
    let mut bare = make_renderer(8, 3);
    bare.suspend();
    bare.shutdown();
    assert!(bare.backend().direct_output().is_empty());
}

#[test]
fn req_008_hit_grid() {
    let mut renderer = make_renderer(8, 4);
    renderer.push_hit_scissor(ClipRect {
        x: 1,
        y: 1,
        width: 6,
        height: 2,
    });
    renderer.add_to_hit_grid(0, 0, 8, 4, 4);
    draw(&mut renderer, "X", 0, 0);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    // Inside the scissor the id lands; outside it stays empty even
    // though the rect covered the whole grid.
    assert_eq!(4, renderer.check_hit(2, 2));
    assert_eq!(0, renderer.check_hit(0, 0));
    assert_eq!(0, renderer.check_hit(7, 3));
    renderer.clear_hit_scissors();
    renderer.add_to_hit_grid(7, 3, 4, 4, 6);
    draw(&mut renderer, "Y", 0, 1);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    // Grid-clipped: only the in-bounds cell registers.
    assert_eq!(6, renderer.check_hit(7, 3));
    assert_eq!(0, renderer.check_hit(0, 0));
}

#[test]
fn req_009_split_offset() {
    let mut renderer = make_renderer(8, 4);
    renderer.set_render_offset(1);
    assert_eq!(1, renderer.render_offset());
    draw(&mut renderer, "O", 4, 0);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert!(output(&renderer).windows(6).any(|w| w == b"\x1b[2;5H"));
    renderer.set_render_offset(0);
    assert_eq!(0, renderer.render_offset());
}

#[test]
fn req_010_image_fallback() {
    let mut renderer = make_renderer(6, 2);
    renderer.next_buffer().push_placement(ImagePlacement {
        placement_id: 3,
        image_handle: 9,
        x: 1,
        y: 0,
        width: 2,
        height: 1,
        pixel_width: 16,
        pixel_height: 8,
        source_x: 0,
        source_y: 0,
        source_width: 16,
        source_height: 8,
        opacity: 200,
        protocol: ImageProtocol::Auto,
    });
    let cell = make_cell(
        pack_image_cell(3, 9),
        rgb_color(255, 255, 255, 255),
        rgb_color(0, 0, 0, 255),
        0,
    );
    renderer.next_buffer().set(1, 0, cell);
    renderer.stage_image(3);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert_eq!(&[3], renderer.committed_images());
    assert_eq!(1, renderer.stats().cells_updated);
    let quadrant = char::from_u32(suprtui::buffer::draw::QUADRANT_CHARS[9])
        .unwrap()
        .to_string();
    let out = output(&renderer);
    assert!(
        out.windows(quadrant.len())
            .any(|w| w == quadrant.as_bytes())
    );
    assert!(!out.windows(4).any(|w| w == b"\x1b_G"));
}
