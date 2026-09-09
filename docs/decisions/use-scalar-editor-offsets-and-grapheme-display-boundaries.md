# Use scalar editor offsets and grapheme display boundaries

Level: Judged
Decided by: Codex
Rests on: API-007
Would be wrong if: Edits split a grapheme, line indices become stale, selection paints outside its range, or display columns disagree between editor output paths.

## Decision

Retain GapBuffer scalar offsets and the existing public Cursor integer fields. Document scalar positions separately from terminal columns. Centralize grapheme boundary and display-column conversion, use it for both editors, and update newline indices on every mutation. Preserve preferred display columns for vertical movement. Build selection and cursor overlays on complete graphemes before clipping styled lines. Retain complete grapheme text in private Surface metadata for the existing editor render signatures, with matching diff output and invalidation on overwrite, without changing public Cell or C ABI layouts. Keep SuprTUI unchanged. Verify both editors, multiline changes, clipping, and actual terminal cell output.

## Realized by

(none yet: recorded, not built)
