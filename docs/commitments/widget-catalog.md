# Commitment: widget-catalog

Status: Agreed 2026-09-15
Requirements: CAT-001, CAT-002, CAT-003

## Pause

Shawn paused the catalog visual repairs on 2026-09-16 to implement
`wgpu-graphics` first. The open findings in `.cairn/reviews/widget-catalog.md`
remain unresolved. This pause is not acceptance or completion; resume the
full-width, spacing, colored column-span, and remaining cube-quality work
after the graphics commitment.

## Deliverable

Build one responsive `widget_catalog` example for screenshots and short video
clips. It presents the public widget families in focused pages, renders the
project logo with the image widget, demonstrates animation with a spinning
wireframe cube, and has verified navigation and quit behavior.

## Boundaries

This commitment may add the catalog example, focused catalog tests and check
programs, its Cairn mechanisms, and the documentation needed to run it. It may
fix a framework defect only when the catalog exposes that defect and the fix
is required for a named catalog requirement. It does not repair the known
Kitty embedded-terminal crash or add production application infrastructure.

Done-when: CAT-001 through CAT-003 pass and final review finds that the catalog
is responsive, capture-ready, locally self-contained, and honest about the
terminal-widget limitation.
