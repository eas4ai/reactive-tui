//! Part of tests/review_facade.rs: FFI-002, the C text buffers paint whole
//! graphemes.

use reactive_tui::core::surface::Surface;
use reactive_tui::ffi::*;
use std::ptr;

/// Paints `text` through a C text buffer onto a one-row surface of `width`
/// cells and returns the cells' characters and the surface's graphemes.
fn painted(text: &str, width: u32) -> (Vec<char>, Vec<String>) {
    unsafe {
        let tb = createTextBuffer(16, 0);
        assert!(!tb.is_null(), "createTextBuffer gave null");
        let bytes = text.as_bytes();
        let written = textBufferWriteChunk(
            tb,
            bytes.as_ptr(),
            bytes.len() as u32,
            ptr::null(),
            ptr::null(),
            ptr::null(),
        );
        assert_eq!(
            written as usize,
            text.chars().count(),
            "textBufferWriteChunk took {written} scalars of {text:?}"
        );
        let buffer = createOptimizedBuffer(width, 1, false, 0, ptr::null(), 0);
        assert!(!buffer.is_null(), "createOptimizedBuffer gave null");
        renderTextBufferToSurface(tb, buffer, 0, 0, width);
        let chars = bufferGetCharPtr(buffer);
        let cells: Vec<char> = std::slice::from_raw_parts(chars, width as usize)
            .iter()
            .map(|code| char::from_u32(*code).unwrap_or('\u{fffd}'))
            .collect();
        bufferReleaseCharPtr(chars, width as usize);
        let surface = &*(buffer as *const Surface);
        let graphemes: Vec<String> = (0..width as usize)
            .map(|x| surface.grapheme(x, 0).into_owned())
            .collect();
        destroyOptimizedBuffer(buffer);
        destroyTextBuffer(tb);
        (cells, graphemes)
    }
}

/// FFI-002: the letter after a wide glyph takes the cell after the glyph's
/// continuation cell, not the continuation cell itself.
#[test]
fn ffi_002_a_letter_after_a_wide_glyph_takes_the_next_free_cell() {
    let (cells, graphemes) = painted("界A", 4);
    assert_eq!(
        cells[2],
        'A',
        "FFI-002: 界A painted as the cells {cells:?} (graphemes {graphemes:?}): A must take column 2, after the wide glyph's continuation cell"
    );
    assert_eq!(
        graphemes[0], "界",
        "FFI-002: the wide glyph at column 0 is {:?}",
        graphemes[0]
    );
}

/// FFI-002: a combining mark stays with its base, in one cell, and the next
/// letter follows in the next cell.
#[test]
fn ffi_002_a_combining_mark_stays_with_its_base() {
    let (cells, graphemes) = painted("e\u{301}A", 4);
    assert_eq!(
        graphemes[0], "e\u{301}",
        "FFI-002: e with a combining acute at column 0 is the grapheme {:?} (cells {cells:?})",
        graphemes[0]
    );
    assert_eq!(
        cells[1], 'A',
        "FFI-002: the letter after the combined e is at {cells:?}, not column 1"
    );
}

/// FFI-002: the header says what `width_method` is.
#[test]
fn ffi_002_the_header_describes_width_method() {
    let create_text_buffer = crate::header::doc_of("createTextBuffer");
    let create_optimized_buffer = crate::header::doc_of("createOptimizedBuffer");
    assert!(
        create_text_buffer.contains("width_method") && create_optimized_buffer.contains("width_method"),
        "FFI-002: the header's documentation of createTextBuffer ({create_text_buffer:?}) and createOptimizedBuffer ({create_optimized_buffer:?}) does not describe width_method"
    );
}
