//! Multi-parser replay: the same byte stream through every terminal parser
//! model, asserting no byte leaks into visible cells.
//!
//! Background: a single parser model only proves the stream parses under
//! that model. The Konsole CAN incident (a 0x18 frame boundary printed as a
//! visible glyph on Konsole-lineage terminals) passed a single-model check,
//! because the model consumed CAN while the real terminal printed it. The
//! fix (ESC ST boundary, see `src/backend/suprtui/output.rs`) is guarded
//! here at the byte level so no future boundary regresses on any model.
//!
//! What this harness covers: byte/sequence bugs — control codes, escape and
//! APC sequences leaking into cells, U+FFFD decoding failures, and model
//! disagreement on plain text.
//!
//! What it explicitly does NOT cover: font glyph coverage. Whether U+2800
//! renders blank or as dotted tofu is a font property, invisible to every
//! parser. That risk was tracked per terminal in the retired
//! terminal-compatibility contract, not in this test.
//!
//! Parser models: `vt100` (always) and Ghostty's real VT parser
//! (`embedded-terminal` feature, via `crates/libghostty-vt`).

const ROWS: u16 = 24;
const COLS: u16 = 80;

/// Feed bytes through the vt100 model and return the visible screen text.
fn vt100_text(bytes: &[u8]) -> String {
    let mut parser = vt100::Parser::new(ROWS, COLS, 0);
    parser.process(bytes);
    parser.screen().contents()
}

/// No decoding failure may ever reach a cell.
fn assert_clean(text: &str) {
    assert!(
        !text.contains('\u{FFFD}'),
        "U+FFFD replacement char leaked into cells: {text:?}"
    );
}

/// No raw C0 control (other than newline) may survive as cell content.
fn assert_no_controls(text: &str) {
    for ch in text.chars() {
        assert!(
            !ch.is_control() || ch == '\n',
            "control char {ch:?} leaked into cells: {text:?}"
        );
    }
}

#[test]
fn synchronized_update_frame_boundary_is_silent() {
    // The exact boundary emitted by the SuprTUI renderer.
    let bytes = b"\x1b\\\x1b[?2026l\x1b[0mcatalog";
    let text = vt100_text(bytes);
    assert_clean(&text);
    assert_no_controls(&text);
    assert!(text.contains("catalog"), "frame body lost: {text:?}");
}

#[test]
fn can_byte_never_reaches_cells() {
    // 0x18 must abort, never print. This is the Konsole regression: a CAN
    // frame boundary rendered as a visible glyph on Konsole-lineage
    // terminals. ST replaced it; this test pins the byte-level rule.
    let bytes = b"A\x18B";
    let text = vt100_text(bytes);
    assert_clean(&text);
    assert!(
        !text.contains('\x18'),
        "CAN byte leaked into cells: {text:?}"
    );
    assert!(
        text.contains('A') && text.contains('B'),
        "text lost: {text:?}"
    );
}

#[test]
fn braille_codepoints_round_trip() {
    // Blank (U+2800) and full (U+28FF) Braille must survive the byte stream
    // byte-identical: parsers see codepoints, fonts decide glyphs.
    let bytes = "donut \u{28ff} blank \u{2800} end".as_bytes();
    let text = vt100_text(bytes);
    assert_clean(&text);
    assert!(text.contains('\u{28ff}'), "braille lost: {text:?}");
    assert!(text.contains('\u{2800}'), "blank braille lost: {text:?}");
}

#[test]
fn sync_envelope_and_apc_payloads_do_not_leak() {
    // Synchronized-update envelope plus a Kitty-graphics APC with an ASCII
    // payload that must be swallowed whole.
    let bytes = b"\x1b[?2026hBODY\x1b_Ga=T,f=32,s=1,v=1;PAYLOAD\x1b\\TAIL\x1b[?2026l";
    let text = vt100_text(bytes);
    assert_clean(&text);
    assert_no_controls(&text);
    assert!(text.contains("BODY"), "body lost: {text:?}");
    assert!(text.contains("TAIL"), "tail lost: {text:?}");
    assert!(
        !text.contains("PAYLOAD"),
        "APC payload leaked into cells: {text:?}"
    );
    assert!(
        !text.contains("2026"),
        "sync envelope leaked into cells: {text:?}"
    );
}

/// Ghostty's real VT parser as the second, independent model. Only built
/// with the `embedded-terminal` feature (same gate as the crate itself).
#[cfg(feature = "embedded-terminal")]
mod ghostty {
    use libghostty_vt::render::{CellIterator, RowIterator};
    use libghostty_vt::{RenderState, Terminal};

    use super::{COLS, ROWS};

    /// Feed bytes through Ghostty's parser and dump visible rows as text.
    fn ghostty_rows(bytes: &[u8]) -> Vec<String> {
        let mut terminal = Terminal::new(COLS, ROWS).expect("terminal init");
        terminal.vt_write(bytes);
        let mut render_state = RenderState::new().expect("render state");
        let snapshot = render_state.update(&terminal).expect("snapshot");
        let mut rows = RowIterator::new().expect("row iterator");
        let mut cells = CellIterator::new().expect("cell iterator");
        let mut row_iter = rows.update(&snapshot).expect("row update");
        let mut out = Vec::new();
        while let Some(row) = row_iter.next() {
            let mut line = String::new();
            let mut cell_iter = cells.update(&row).expect("cell update");
            while let Some(cell) = cell_iter.next() {
                let graphemes = cell.graphemes().expect("graphemes");
                if graphemes.is_empty() {
                    line.push(' ');
                } else {
                    for g in graphemes {
                        line.push(g);
                    }
                }
            }
            out.push(line);
        }
        out
    }

    fn ghostty_text(bytes: &[u8]) -> String {
        ghostty_rows(bytes).join("\n")
    }

    /// Both models must agree character-for-character on plain text.
    /// Agreement is asserted only where VT semantics are unambiguous;
    /// control/edge cases get per-model no-leak assertions instead.
    /// Normalised screen text from the vt100 model and the ghostty model.
    fn model_texts(bytes: &[u8]) -> (String, String) {
        let vt: Vec<String> = super::vt100_text(bytes)
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect();
        let ghost: Vec<String> = ghostty_rows(bytes)
            .iter()
            .map(|l| l.trim_end().to_string())
            .collect();
        (
            vt.join("\n").trim().to_string(),
            ghost.join("\n").trim().to_string(),
        )
    }

    #[test]
    fn models_agree_on_styled_text() {
        let bytes = b"Hello, \x1b[1;32mworld\x1b[0m!";
        let (vt, ghost) = model_texts(bytes);
        assert_eq!(vt, ghost, "parser models disagree on {bytes:?}");
        assert_eq!(vt, "Hello, world!");
    }

    #[test]
    fn models_agree_on_braille() {
        let bytes = "donut \u{28ff} blank \u{2800} end".as_bytes();
        let (vt, ghost) = model_texts(bytes);
        assert_eq!(vt, ghost, "parser models disagree on {bytes:?}");
        assert_eq!(vt, "donut \u{28ff} blank \u{2800} end");
    }

    #[test]
    fn ghostty_boundary_is_silent() {
        let text = ghostty_text(b"\x1b\\\x1b[?2026l\x1b[0mcatalog");
        assert!(!text.contains('\u{FFFD}'), "FFFD leaked: {text:?}");
        assert!(text.contains("catalog"), "frame body lost: {text:?}");
    }

    #[test]
    fn ghostty_surfaces_bare_can_so_the_framework_must_not_emit_it() {
        // Ghostty's parser surfaces a bare CAN as cell content while vt100
        // drops it. The models disagree, and that disagreement is exactly
        // why the renderer emits ESC ST frame boundaries instead of CAN
        // (see src/backend/suprtui/output.rs): any CAN byte is
        // terminal-roulette. If this assertion ever fails, Ghostty changed
        // CAN handling — re-check model agreement before touching the
        // boundary.
        let text = ghostty_text(b"A\x18B");
        assert!(
            text.contains('\x18'),
            "Ghostty stopped surfacing CAN; re-check model agreement: {text:?}"
        );
    }

    #[test]
    fn ghostty_apc_payload_does_not_leak() {
        let text = ghostty_text(b"OK\x1b_Ga=T,f=32,s=1,v=1;PAYLOAD\x1b\\");
        assert!(text.contains("OK"), "text lost: {text:?}");
        assert!(
            !text.contains("PAYLOAD"),
            "APC payload leaked into cells: {text:?}"
        );
    }
}
