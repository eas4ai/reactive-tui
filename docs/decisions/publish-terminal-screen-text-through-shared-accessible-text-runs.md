# Publish terminal screen text through shared accessible text runs

Level: Judged
Decided by: Codex
Rests on: API-011,API-018
Would be wrong if: Orca cannot read actual child output or cursor positions, text runs disclose hidden terminal cells, or the shared conversion changes editor password and Unicode selection behavior.
History: TextInput already publishes grapheme-aware AccessKit text runs and resolves their stable child IDs in App. The retained terminal currently paints styled fragments with no terminal role or readable document.

## Decision

Reuse the existing private text-run conversion for TextInput and TerminalWidget. Publish the terminal role and current displayed screen as bounded text runs, with a caret only when the child cursor is in that view. Generate the text from parsed visible cells, preserving graphemes and replacing invisible cells with spaces; never expose raw child bytes or hidden text. Keep visual cells separate from the readable document and retain the existing PTY, focus and input ownership. Verify a real child input/output Orca workflow, Unicode and hidden-cell regressions, and the existing editor text and selection checks.

## Realized by

Implementation: `src/widgets/terminal/paint.rs`, `src/accessibility/text.rs`.

Behavior checks: `tests/api_widget_behavior/orca_display.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
