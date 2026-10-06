//! Part of tests/review_facade.rs: FFI-005, `renderSurfaceToTerminal` paints
//! through the caller's terminal and keeps its session.

use crate::terminal::on_terminal;
use reactive_tui::core::surface::{Attr, Cell, Rgba, Surface};
use reactive_tui::ffi::*;
use std::io::Write;
use std::ptr;

/// FFI-005: after `setupTerminal`, a `renderSurfaceToTerminal` call writes
/// the surface, graphemes whole, and leaves the caller's session as it was.
#[test]
fn ffi_005_render_surface_to_terminal_keeps_the_callers_session() {
    let written = on_terminal(
        "render::ffi_005_render_surface_to_terminal_keeps_the_callers_session",
        || unsafe {
            let terminal = createTerminal();
            assert!(!terminal.is_null(), "createTerminal gave null");
            setupTerminal(terminal, true);
            let buffer = createOptimizedBuffer(8, 1, false, 0, ptr::null(), 0);
            assert!(!buffer.is_null(), "createOptimizedBuffer gave null");
            let surface = &mut *(buffer as *mut Surface);
            let white = Rgba::white();
            let black = Rgba::black();
            let mut cell = Cell::default();
            cell.fg = white;
            cell.bg = black;
            surface.set_grapheme(0, 0, "e\u{301}", cell);
            surface.write_str(1, 0, "界A", white, black, Attr::empty());
            let mut stdout = std::io::stdout();
            stdout.write_all(b"MARK1").unwrap();
            stdout.flush().unwrap();
            let drawn = renderSurfaceToTerminal(buffer, terminal);
            stdout.write_all(b"MARK2").unwrap();
            stdout.flush().unwrap();
            destroyOptimizedBuffer(buffer);
            destroyTerminal(terminal);
            assert!(drawn, "renderSurfaceToTerminal returned false");
        },
    );
    if let Some(text) = written {
        let start = text.find("MARK1").expect("the first marker");
        let end = text.find("MARK2").expect("the second marker");
        let between = &text[start..end];
        assert!(
            !between.contains("\x1b[?1049l"),
            "FFI-005: the drawing call left the alternate screen under the caller's live handle:\n{between:?}"
        );
        assert!(
            between.contains("e\u{301}") && between.contains("界") && between.contains('A'),
            "FFI-005: the drawn frame lost a grapheme or a glyph:\n{between:?}"
        );
    }
}

/// FFI-005: the header says what `renderWithStats` does with its terminal
/// argument.
#[test]
fn ffi_005_the_header_says_what_render_with_stats_does_with_its_terminal() {
    let doc = crate::header::doc_of("renderWithStats");
    assert!(
        doc.contains("terminal") && (doc.contains("validated") || doc.contains("not written")),
        "FFI-005: the header's documentation of renderWithStats ({doc:?}) does not say what it does with its terminal argument"
    );
}
