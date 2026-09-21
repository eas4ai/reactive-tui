# Share platform image decoding and apply draw options before protocol output

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014
Would be wrong if: Existing callers require encoded file bytes to be interpreted as pixels or rely on ignored drawing options rather than the documented source clipping and scaling behavior.
History: The existing image ownership decision already requires repairing the platform and Surface adapters against shared decoded pixels. Earlier API reversals require preserving public data shapes and explicit limits; this choice does not remove a format or protocol claim.

## Decision

Keep the public platform Image and DrawOptions data shapes and signatures. Decode files and encoded memory with the shared bounded decoder; validate raw pixel extents before writing output. Crop the source first. Treat an explicit cell size as the drawing box; otherwise scaling modes use the terminal size. None keeps native pixels and clips to an explicit box, Fill stretches to the box, Fit preserves aspect ratio and may enlarge, and Contain preserves aspect ratio without enlarging. Use physical cell dimensions when available and the documented 8x16 fallback otherwise. Preserve Kitty pixel offsets and z-order in its placement command. Other protocols use transparent padding for pixel offsets and report nonzero Kitty-only z-order as unsupported before writing bytes. Reuse valid serializers and complete the advertised WebP and Sixel decoding paths. Validate and encode before writing so malformed sources do not leave a partial graphics command. These choices do not replace the required host evidence.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/platform/image`, `src/core/surface`, `src/backend/suprtui/graphics`.

Behavior checks: `tests/api_widget_behavior/image.rs`, `tests/api_widget_behavior/image_host_capture.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
