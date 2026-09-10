# Keep DataTable query state above its measured row view

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Sorting only affects one page, row callbacks expose page-local indices, hidden-column or filter settings are discarded, controls lose edits on redraw, or virtual scrolling changes values without changing the rendered row window.
History: The original DataTable painted descriptive text and a named Table, had no input controls, omitted filters from prop equality, and its builder discarded filters and hidden columns. The repaired Table now supplies measured input and clipping.

## Decision

Keep DataTable search, typed filters, sort priorities, page, hidden columns and selected row IDs in an App-owned retained model. Compose controls from the repaired buttons and TextInput. Apply search, filters and stable sort before paging, and translate child row callbacks to source indices. A header click replaces sort priorities; Shift-click adds or toggles a secondary priority. The existing pixel-based virtual-scroll configuration remains a row-count request computed from viewport_height divided by row_height; App caps that request to its measured terminal area and renders each data row in one terminal line. Overscan controls retained rows outside the clipped viewport. Zero row height or page size produces a visible configuration error instead of a fabricated dimension. Table size values continue to use terminal cells in App; explicit column minima remain honored.

## Realized by

Implementation: `src/widgets/display/data_table`.

Behavior checks: `tests/api_widget_behavior/data_table.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
