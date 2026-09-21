# Compose shaped embedded cursors through the owned host renderer

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 RND-001 RND-005 EMB-001
Would be wrong if: Cursor shapes overwrite child text, escape clipping or covering content, survive removal, or bypass acknowledged renderer output and restoration.
History: The retained terminal decision preserves both embedded APIs and the SuprTUI output owner. Cursor parsing and owned blink tests now pass, but painting still turns underline and bar into a solid block.

## Decision

Keep solid block cursors in cell composition. Carry a private cursor request with the text element for underline and bar modes. The layout painter places it using the painted grapheme coordinates and clipping, and later opaque cell content hides it. SuprTUI emits the native host shape in its existing acknowledged frame operation, preserving the text beneath it. The widget retains blink scheduling and focus ownership; the host receives a steady shape while each visible phase is active. Reset native cursor style and color on host restoration and prevent shape state leaking into plain CellFrame output. Verify real PTY child output through App, captured host shape sequences, wide text, clipping, covering content, blinking, removal and output failure. No public API signature changes.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/layout/paint_tree/suprtui/cursor.rs`, `src/backend/suprtui/cursor_tests.rs`.

Behavior checks: `tests/api_widget_behavior/terminal.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
