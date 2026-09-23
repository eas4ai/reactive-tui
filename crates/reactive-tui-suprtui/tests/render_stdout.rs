//! Integration tests for the render-stdout commitment (REN-012).
//!
//! The parity vector renders the same scene through the memory and
//! stdout backends and demands identical streams; the rest exercise
//! the public backend through a `Cursor<Vec<u8>>` stand-in.

use std::cell::RefCell;
use std::io::Cursor;
use std::rc::Rc;
use suprtui::ansi::rgb_color;
use suprtui::render::{Backend, MemoryBackend, RenderStatus, Renderer, StdoutBackend, WriteStatus};
use suprtui::uni::pool::GraphemePool;

fn pool() -> Rc<RefCell<GraphemePool<'static>>> {
    Rc::new(RefCell::new(GraphemePool::new()))
}

fn draw<B: Backend>(renderer: &mut Renderer<'static, B>, text: &str, x: u32, y: u32) {
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

fn render_scene<B: Backend>(renderer: &mut Renderer<'static, B>) {
    // Immediate-mode frames redraw the whole scene every time; an
    // undrawn frame blanks the screen by design.
    draw(renderer, "HI", 2, 1);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    renderer.set_cursor(7, 2, true);
    draw(renderer, "HI", 2, 1);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    draw(renderer, "HI", 2, 1);
    assert_eq!(RenderStatus::Skipped, renderer.render(false));
}

#[test]
fn req_012_memory_stdout_parity() {
    let mut memory = Renderer::new(8, 3, pool(), MemoryBackend::new()).unwrap();
    render_scene(&mut memory);
    let expected: Vec<u8> = memory.into_backend().frames().concat();

    let cursor = Cursor::new(Vec::new());
    let mut stdout = Renderer::new(8, 3, pool(), StdoutBackend::new(cursor)).unwrap();
    render_scene(&mut stdout);
    let actual = stdout.into_backend().into_writer().into_inner();

    assert!(!expected.is_empty());
    assert_eq!(expected, actual);
}

#[test]
fn req_012_failed_frame_drops_bytes() {
    let cursor = Cursor::new(Vec::new());
    let mut backend = StdoutBackend::new(cursor);
    backend.begin_frame();
    backend.write_bytes(b"partial");
    backend.fail_frame();
    assert_eq!(WriteStatus::Failed, backend.end_frame());
    backend.begin_frame();
    backend.write_bytes(b"kept");
    assert_eq!(WriteStatus::Ok, backend.end_frame());
    assert_eq!(b"kept", backend.into_writer().into_inner().as_slice());
}

#[test]
fn req_012_empty_frame_writes_nothing() {
    let cursor = Cursor::new(Vec::new());
    let mut backend = StdoutBackend::new(cursor);
    backend.begin_frame();
    assert_eq!(WriteStatus::Ok, backend.end_frame());
    assert!(backend.into_writer().into_inner().is_empty());
}
