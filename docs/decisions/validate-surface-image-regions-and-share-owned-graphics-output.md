# Validate Surface image regions and share owned graphics output

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014 API-016
Would be wrong if: The public Surface cell image contract cannot be preserved through shared decoded pixels and protocol serializers without changing existing signatures.
History: The retained-image decision already requires repairing Surface adapters and preserving advertised behavior; no feature or protocol is removed.

## Decision

Preserve existing Surface constructors and placement signatures. Add checked alternatives for malformed raw images and unrepresentable regions; legacy methods reject invalid input without panics or malformed output. Compute proportional source boundaries so remainder pixels and images smaller than the cell region are preserved, and clip destination loops before iteration. Remove dangling placements when images are removed. Replace DiffWriter placeholder shading with decoded cell fallback and shared owned graphics serializers, with explicit host options and checked output preparation. Keep protocol cleanup tied to the output owner and test source changes, clipping, layering, removal and failures. Implementation proceeds through validation and placement first, then output integration; neither alone establishes image-family acceptance.

## Realized by

Implementation: `src/platform/image`, `src/core/surface`, `src/backend/suprtui/graphics`.

Behavior checks: `tests/api_widget_behavior/image.rs`, `tests/api_widget_behavior/image_host_capture.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.

## Realized by

Surface now has checked RGBA registration and proportional/clipped region placement,
clears removed image references, and preserves zero as the error ID. DiffWriter uses
decoded fallback cells or the same owned Graphics serializer as App, with explicit
acknowledgment after delivery. Renderer propagates output failures and retries
without advancing its previous surface. It removes graphics before restoration.
Known-pixel and output lifecycle tests, a safe placeholder failure, and real Kitty,
Xterm/Sixel, WezTerm/inline and GNOME fallback captures are recorded in the current
commitment review. Kitty supports native behind-text images; the legacy protocols
preserve glyphs over sampled decoded image backgrounds in text cells. The behind-text
host checks pass for Kitty and Xterm. An isolated /dev/full fixture verifies a failed
buffered frame followed by successful redraw. Full editing regression passed with
899 library tests (three ignored), 340 App tests, strict Clippy, format and diff checks.
Implementation is not yet committed; Cairn evidence remains a later action.
