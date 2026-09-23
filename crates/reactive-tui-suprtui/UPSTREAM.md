# Renderer origin

Imported from https://github.com/eas4ai/suprtui at
7793deb80c5bceecc5d8ed9fc6bc6d2530531774 on 2026-09-07.

This is the renderer implementation published for Reactive TUI. The original
renderer was adapted from OpenTUI; its MIT notice is retained in
LICENSE-OpenTUI. The package declares MIT. The imported source and tests are
retained together so renderer changes can be checked independently.

Changes after import are maintained in the Reactive-TUI Git history.

The image fallback blitters in src/blit.rs follow the notcurses blitters
(https://github.com/dankamongmen/notcurses, Apache-2.0) as ideas, not code:
the half-block, quadrant, sextant, octant and braille tiers, the exhaustive
search for the two-color split of each block, and a transparent background
for any block with a transparent pixel. No notcurses source was copied.
