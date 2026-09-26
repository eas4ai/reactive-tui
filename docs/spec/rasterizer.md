# Rasterizer

Prefix: RAS

The rasterizer in crates/reactive-tui-suprtui/src/render.rs turns the
painted cell frame into terminal bytes. Measured at 33bc6700 (release,
in-memory backend, text-filled screens): every changed cell emits an
absolute cursor move, an SGR reset, a full foreground, a full background
and the glyph, 37 to 43 bytes; a full 200 by 50 repaint is 378 KB and
1.8 ms; the 262,144-cell maximum is 10.2 MB and 48 ms; each cell's bytes
are built with a per-cell `String`; the unchanged-row check reads cells one
at a time through `get`. Presentation is in presentation.md, the painter in
painter.md and image fallback in blitters.md. Input is a separate contract.

## Observed

(none yet)

## Draft

[RAS-001] The rasterizer MUST track the terminal cursor across the glyphs it emits within a frame, advancing by each glyph's cell width; a changed cell at the tracked position MUST emit no cursor move; a changed cell elsewhere on the tracked row MUST move with CHA (`ESC [ n G`); every other move MUST use CUP.
Falsifier: Two horizontally adjacent changed cells produce two cursor-move sequences, or a same-row jump emits CUP.
Mechanism: render-bytes
Status: Agreed 2026-09-22

[RAS-002] Foreground, background, base attributes and decoration MUST be emitted only when they differ from the last cell emitted in the frame; an attribute that turns off MUST use its SGR off code (22, 23, 24, 25, 27, 28, 29, 55); `ESC [0m` MUST appear at most once after the frame's sync-set and once before its sync-reset.
Falsifier: Consecutive changed cells of one style contain more than one foreground sequence, or a per-cell `ESC [0m` appears.
Mechanism: render-bytes
Status: Agreed 2026-09-22

[RAS-003] Cell emission MUST append to the frame's byte buffer directly with integer formatting; no `String` or `Vec` MUST be created per cell on the render path.
Falsifier: A counting allocator reports any allocation during `render` of a 10,000-cell frame once the frame buffer has reached capacity.
Mechanism: render-alloc
Status: Agreed 2026-09-22

[RAS-004] For every renderer test frame and every charts golden, the byte stream replayed through the vt100 and ghostty parser models MUST produce the screen text and colors recorded in tests/snapshots/renderer before the rasterizer changed.
Falsifier: A replay differs from its recording in a cell's text or color, or a frame has no recording.
Mechanism: render-replay
Status: Agreed 2026-09-22

[RAS-005] In a release build on the development host, best of three runs, a full 200 by 50 text repaint MUST emit at most 12 bytes per cell, an unchanged 262,144-cell frame MUST cost at most 1 millisecond and a full repaint of it at most 15 milliseconds.
Falsifier: The bench exceeds any bound on all three runs.
Mechanism: render-bytes
Rationale: The unchanged bound was 300 microseconds; the diff reads both 8 MB buffers and measures 530 to 700 microseconds after the painter's repaint, so the developer ruled 1 millisecond on 2026-09-22 (escalation a648dc31).
Status: Agreed 2026-09-22

[RAS-006] `RenderStats` MUST report bytes emitted, cursor moves emitted and elided, foreground, background and attribute emissions and elisions, and layout, diff, emit and write times per frame; the debug overlay MUST show bytes, elisions and the four times.
Falsifier: A rendered frame reports zero bytes, or a listed statistic is absent from the overlay.
Mechanism: render-bytes
Status: Agreed 2026-09-22

[RAS-007] The hit grid MUST be allocated on its first write and a frame that never wrote it MUST not fill it; the next buffer MUST be cleared once per frame.
Falsifier: A renderer that received no hit write allocates the grid, or a frame clears the next buffer twice.
Mechanism: render-alloc
Status: Agreed 2026-09-22

[RAS-008] The frame diff MUST compare rows on the buffer's column arrays in one pass that finds the first differing column; it MUST not construct cell values on the compare path.
Falsifier: The compare path constructs cell values, or the unchanged-frame bound in RAS-005 is exceeded.
Mechanism: render-bytes
Status: Agreed 2026-09-22
