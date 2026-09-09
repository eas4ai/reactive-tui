# Resolve node state before painting and lay out text in terminal cells

Level: Judged
Decided by: Codex
Rests on: API-009
Would be wrong if: State styles use unacknowledged event geometry, Unicode text is split inside a grapheme, or measurement disagrees with painted wrapping.
History: The clipboard reversal corrected a native Windows deadline after platform evidence. This decision stays Judged because it preserves prior ownership and presentation contracts; independent captured frames will test terminal semantics before acceptance.

## Decision

Resolve focus, focus-within and hover variants from the existing acknowledged event tree before painting. Keep event registration after successful presentation and request another frame if registration changes styling state. Store disabled state in the owned Element metadata introduced under API-005; disabled nodes cannot focus or activate. Evaluate compound variants before the class cache. Retain the public NodeSpec shape. Add inherited text layout settings to the private StyleBuilder payload and use one grapheme-aware cell layout helper for measurement and painting. Implement case conversion, whitespace handling, wrapping, alignment and ellipsis with explicit resets. Keep host-controlled font family and physical size aliases; document cell rounding for line height and letter spacing and test the resulting output. Preserve callback ownership, focus traps, Rust signatures and the C ABI.

## Realized by

- 4b2369d3ed1fbe86829491b25978ecafac0e2ef7 Render state-dependent styles and Unicode terminal typography
