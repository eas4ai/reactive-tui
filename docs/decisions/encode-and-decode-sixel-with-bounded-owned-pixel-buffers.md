# Encode and decode Sixel with bounded owned pixel buffers

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014
Would be wrong if: The Rust Sixel codec cannot preserve palette colors and declared pixel coordinates under protocol captures and real-host rendering, or duplicates a suitable bounded codec already available in the repository.
History: The image decisions require decoded-pixel parity and valid output while preserving protocol claims. The current sixel-rs capture for a two-pixel red and blue image emits one red palette entry and advances a sixel row before drawing; new pixel assertions fail. No readiness claim can rest on that output.

## Decision

Use a small owned Rust Sixel codec with explicit encoded-size, decoded-size, work and workspace limits. Decode one DCS image with palette definitions, RGB/HLS colors, repetition, raster extents, row controls and transparent background into checked RGBA pixels; reject malformed controls before allocation. Encode small palettes exactly and quantize larger palettes with bounded scratch memory. Retain Fast, Balanced and High quality choices, using no diffusion, Atkinson and Stucki respectively when quantization is required. Emit explicit palette and raster data in the correct six-pixel bands, preserve fully transparent regions where supported, and bound the output. Keep public APIs and Sixel support. Verify hand-authored pixel expectations, round trips, hostile sizes, and actual host screenshots; do not count serializer prefixes as sufficient evidence.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/display/image/sixel_encode.rs`, `src/platform/image`.

Behavior checks: `tests/api_widget_behavior/image.rs`, `tests/api_widget_behavior/image_host_capture.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.


The unused sixel-rs dependency and its orphaned build dependencies were removed
when checking the developer-provided Windows/MSVC and MacBook toolchains. No Rust
source still imports that crate. The owned encoder and decoder retain the image
API and protocol contract; 87 image-filtered tests passed after dependency removal.
