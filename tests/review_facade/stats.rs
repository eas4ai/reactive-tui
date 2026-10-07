//! Part of tests/review_facade.rs: FFI-004, the C renderer's hit grid, render
//! offset, host statistics and buffer dump do what their names say. The
//! renderer takes raw mode, so each test runs on a pseudo-terminal.

use crate::terminal::{on_terminal, without_sgr};
use reactive_tui::ffi::*;
use std::ptr;

/// Writes `text` into the renderer's surface at (0, 0) through a C text
/// buffer.
unsafe fn write_row(renderer: *mut RTuiRenderer, text: &str) {
    unsafe {
        let tb = createTextBuffer(16, 0);
        let bytes = text.as_bytes();
        textBufferWriteChunk(
            tb,
            bytes.as_ptr(),
            bytes.len() as u32,
            ptr::null(),
            ptr::null(),
            ptr::null(),
        );
        renderTextBufferToRenderer(tb, renderer, 0, 0, 8);
        destroyTextBuffer(tb);
    }
}

/// FFI-004: `checkHit` returns what `addToHitGrid` registered, after a
/// completed render, and 0 elsewhere.
#[test]
fn ffi_004_check_hit_returns_what_was_registered() {
    on_terminal(
        "stats::ffi_004_check_hit_returns_what_was_registered",
        || unsafe {
            let renderer = createRenderer(4, 3);
            assert!(!renderer.is_null(), "createRenderer gave null");
            let before = checkHit(renderer, 1, 1);
            addToHitGrid(renderer, 1, 1, 1, 1, 42);
            render(renderer, true);
            let hit = checkHit(renderer, 1, 1);
            let miss = checkHit(renderer, 0, 0);
            let outside = checkHit(renderer, 4, 0);
            destroyRenderer(renderer, true, 0);
            assert_eq!(
                before, 0,
                "FFI-004: checkHit before any registration gave {before}"
            );
            assert_eq!(
                hit, 42,
                "FFI-004: checkHit on the registered cell gave {hit}, not the id 42"
            );
            assert_eq!(
                miss, 0,
                "FFI-004: checkHit beside the registered cell gave {miss}"
            );
            assert_eq!(
                outside, 0,
                "FFI-004: checkHit outside the renderer gave {outside}"
            );
        },
    );
}

/// FFI-004: the statistics a host reports reach the debug overlay.
#[test]
fn ffi_004_reported_statistics_reach_the_overlay() {
    let written = on_terminal(
        "stats::ffi_004_reported_statistics_reach_the_overlay",
        || unsafe {
            let renderer = createRenderer(60, 6);
            assert!(!renderer.is_null(), "createRenderer gave null");
            setDebugOverlay(renderer, true, 0);
            updateStats(renderer, 16.0, 4242, 1.0);
            updateMemoryStats(renderer, 123_456, 999_999, 7);
            render(renderer, true);
            destroyRenderer(renderer, true, 0);
        },
    );
    if let Some(text) = written {
        assert!(
            text.contains("4242"),
            "FFI-004: with the debug overlay on, a frame after updateStats reporting 4242 frames per second did not show it:\n{text}"
        );
    }
}

/// FFI-004: `setRenderOffset` shifts the rows the renderer writes.
#[test]
fn ffi_004_the_render_offset_shifts_the_rows_written() {
    let written = on_terminal(
        "stats::ffi_004_the_render_offset_shifts_the_rows_written",
        || unsafe {
            let renderer = createRenderer(8, 2);
            assert!(!renderer.is_null(), "createRenderer gave null");
            write_row(renderer, "ROW0");
            setRenderOffset(renderer, 2);
            render(renderer, true);
            destroyRenderer(renderer, true, 0);
        },
    );
    if let Some(text) = written {
        let plain = without_sgr(&text);
        assert!(
            plain.contains("\x1b[3;1HROW0"),
            "FFI-004: with a render offset of 2 the first row was not written at terminal row 3:\n{plain:?}"
        );
    }
}

/// FFI-004: `dumpBuffers` writes the surfaces to a file named with the
/// timestamp in the current directory.
#[test]
fn ffi_004_dump_buffers_writes_a_file() {
    on_terminal("stats::ffi_004_dump_buffers_writes_a_file", || unsafe {
        let dir = std::env::temp_dir().join(format!("rtui-facade-dump-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("a directory for the dump");
        std::env::set_current_dir(&dir).expect("the dump directory is current");
        let renderer = createRenderer(8, 2);
        assert!(!renderer.is_null(), "createRenderer gave null");
        write_row(renderer, "ROW0");
        render(renderer, true);
        dumpBuffers(renderer, 7);
        destroyRenderer(renderer, true, 0);
        let path = dir.join("rtui-buffers-7.txt");
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            content.contains("ROW0"),
            "FFI-004: dumpBuffers(7) left no rtui-buffers-7.txt holding the front surface's text (read {content:?})"
        );
    });
}
