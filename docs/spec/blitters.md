# Image fallback blitters

Prefix: BLT

When the host has no pixel protocol, an image is drawn as block glyphs.
Today crates/reactive-tui-suprtui/src/buffer/draw.rs maps a 2 by 2 nibble
per cell to one of sixteen quadrant glyphs with one foreground and one
background, and src/widgets/display/image chooses that fallback. The
capability report (src/core/capabilities.rs) carries one `unicode` flag and
the terminal's identity from the environment; it cannot ask a host which
block glyphs its font covers. The algorithms below are borrowed from
notcurses as ideas, not code, and credited in the crate's UPSTREAM.md.

## Observed

(none yet)

## Draft

[BLT-001] Image fallback MUST offer half-block, quadrant, sextant (U+1FB00 to U+1FB3B), octant (U+1CD00 to U+1CDE5) and braille (U+2800 to U+28FF) blitters; for each pixel block the glyph and its two colors MUST be the two-partition of the block's pixels with the minimum total color error, with any transparent pixel forcing a transparent background.
Falsifier: An exhaustive check over a fixed block finds a partition with lower total error than the one chosen, or a block with a transparent pixel renders an opaque background.
Mechanism: blitters
Status: Agreed 2026-09-22

[BLT-002] The blitter MUST be chosen by tier, braille, octant, sextant, quadrant, half-block, ASCII: ASCII when the capability report says no unicode, otherwise the tier a per-terminal table gives the host's identity, sextant when the host is not in the table; an application or environment override MUST replace the choice.
Falsifier: A host reporting no unicode receives a block glyph above ASCII, a host in the table receives a tier other than its entry without an override, or an override is ignored.
Mechanism: blitters
Rationale: The item's text said "by the host's reported glyph support"; no terminal reports font coverage, so the rule names what the report can say and puts the rest in a table and an override.
Status: Agreed 2026-09-22
