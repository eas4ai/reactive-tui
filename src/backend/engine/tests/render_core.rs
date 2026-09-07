//! Integration tests for the render-core commitment (REN-001 … REN-005,
//! REN-011). These exercise the public renderer API; the spec-literal
//! unit tests live in `src/render.rs`.

use std::cell::RefCell;
use std::rc::Rc;
use suprtui::ansi::rgb_color;
use suprtui::render::{CursorStyle, MemoryBackend, RenderStatus, Renderer};
use suprtui::uni::pool::GraphemePool;

fn renderer(width: u32, height: u32) -> Renderer<'static, MemoryBackend> {
    let pool = Rc::new(RefCell::new(GraphemePool::new()));
    Renderer::new(width, height, pool, MemoryBackend::new()).unwrap()
}

fn draw(renderer: &mut Renderer<'static, MemoryBackend>, text: &str, x: u32, y: u32) {
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
fn req_001_frame_publish() {
    let mut renderer = renderer(8, 3);
    draw(&mut renderer, "HI", 2, 1);
    assert!(renderer.backend().frames().is_empty());
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    let out = output(&renderer);
    assert!(out.contains(&b'H'));
    assert!(out.contains(&b'I'));
    // Cell (2,1) positions the cursor at row 2, column 3 (1-based).
    assert!(out.windows(6).any(|w| w == b"\x1b[2;3H"));
}

#[test]
fn req_002_unchanged_skips() {
    let mut renderer = renderer(8, 3);
    draw(&mut renderer, "HI", 2, 1);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    let bytes = output(&renderer).len();
    draw(&mut renderer, "HI", 2, 1);
    assert_eq!(RenderStatus::Skipped, renderer.render(false));
    assert_eq!(bytes, output(&renderer).len());
}

#[test]
fn req_003_cursor_tracking() {
    let mut renderer = renderer(8, 3);
    renderer.set_cursor(7, 2, true);
    renderer.set_cursor_style(CursorStyle::Underline, true);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    let out = output(&renderer);
    assert!(out.windows(6).any(|w| w == b"\x1b[3;8H"));
    assert!(out.windows(5).any(|w| w == b"\x1b[3 q"));
    let bytes = out.len();
    assert_eq!(RenderStatus::Skipped, renderer.render(false));
    assert_eq!(bytes, output(&renderer).len());
}

#[test]
fn req_004_failed_frame_rolls_back() {
    let mut renderer = renderer(8, 3);
    draw(&mut renderer, "OK", 0, 0);
    renderer.set_next_hit(4, 2, 11);
    renderer.stage_image(7);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert_eq!(11, renderer.committed_hit(4, 2));
    assert_eq!(&[7], renderer.committed_images());

    draw(&mut renderer, "XX", 0, 2);
    renderer.set_next_hit(0, 0, 13);
    renderer.stage_image(9);
    renderer.backend_mut().set_fail_next(true);
    assert_eq!(RenderStatus::Failed, renderer.render(false));
    assert_eq!(0, renderer.committed_hit(0, 0));
    assert_eq!(11, renderer.committed_hit(4, 2));
    assert_eq!(&[7], renderer.committed_images());

    // Recovery repaints everything the caller redraws; no stale cell
    // from the failed frame survives.
    draw(&mut renderer, "OK", 0, 0);
    draw(&mut renderer, "XX", 0, 2);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    let out = output(&renderer);
    assert!(out.contains(&b'O'));
    assert!(out.contains(&b'X'));
    assert_eq!((8 * 3) as u32, renderer.stats().cells_updated);
}

#[test]
fn req_005_memory_backend_exact() {
    let mut renderer = renderer(1, 1);
    draw(&mut renderer, "Q", 0, 0);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert_eq!(1, renderer.backend().frames().len());
    let hand =
        "\x1b[?2026h\x1b[?25l\x1b[1;1H\x1b[38;2;255;255;255m\x1b[48;2;0;0;0mQ\x1b[0m\x1b[?2026l";
    assert_eq!(hand.as_bytes(), renderer.backend().frames()[0].as_slice());
    // A second identical frame skips and appends nothing.
    draw(&mut renderer, "Q", 0, 0);
    assert_eq!(RenderStatus::Skipped, renderer.render(false));
    assert_eq!(1, renderer.backend().frames().len());
}

#[test]
fn req_011_stats_count() {
    let mut renderer = renderer(8, 3);
    draw(&mut renderer, "S", 0, 0);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert_eq!(1, renderer.stats().frame_count);
    draw(&mut renderer, "S", 0, 0);
    assert_eq!(RenderStatus::Skipped, renderer.render(false));
    assert_eq!(1, renderer.stats().frame_count);
    draw(&mut renderer, "T", 1, 0);
    assert_eq!(RenderStatus::Rendered, renderer.render(false));
    assert_eq!(2, renderer.stats().frame_count);
}
