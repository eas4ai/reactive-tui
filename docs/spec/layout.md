# Layout

Prefix: LAY

An element's utility classes become a Taffy style (src/layout/css), Taffy
lays the tree out, and the painter reads each node's position and size in
cells (src/layout/paint_tree/suprtui.rs). Read and seen on 2026-09-29: the
width and height classes count in cells (`w-24` is 24 cells), but the
padding, margin, gap and space classes count in fours (`p-1` is 4 cells,
`gap-4` is 16), so the widget catalog writes `gap-0.25` for a gap of one
cell (src/layout/css/parsers.rs). In the catalog at 160 columns the first
and second card of a three-column grid touch and the third has a gap of two
cells, and a cell that spans all four columns of a grid ends one cell short
of it. `gap-x-2` sets the gap between rows to zero, `col-span-full` spans
twelve columns whatever the grid has, and `grid-cols-auto-fit-20` makes
twenty columns. The data table gives each of its panels half of its own
last height, so with two panels open its card grows on every layout
(src/widgets/display/data_table/live.rs). Widget sizes and colors are not
part of this file; the widget bar (quality-bar.md) holds them.

## Observed

(none yet)

## Draft

[LAY-001] A grid, or a flex container that packs its items together, whose class or style asks for a gap MUST paint exactly that many cells between every two neighbouring items, across and down, at every position of the container and at every size that holds its gaps and one cell for each item; grid tracks of equal weight MUST differ in size by at most one cell; and an item that spans several tracks MUST start on the first cell of its first track and end on the last cell of its last track.
Falsifier: In a grid of two to six equal columns with a gap of one or two cells, at any container width from 40 to 512 cells, at a whole or a fractional position, two neighbouring items are painted with a gap other than the one asked for, two tracks differ in width by more than one cell, or an item that spans tracks starts or ends on a cell other than its first track's first or its last track's last; or the same is true of the rows of a grid, or of a flex row or column with a gap.
Mechanism: layout
Rationale: The developer reported on 2026-09-28 that the space between boxes collapses in wide layouts; a gap of one cell is where an error of one cell shows.
Status: Agreed 2026-09-29

[LAY-002] The number in a padding, margin, gap or space class MUST be a count of cells, as it is in the width and height classes: `p-1` pads one cell on each side and `gap-2` leaves two cells; every whole number from 0 to 512 MUST be accepted; and a number with a fraction MUST count as the next whole number of cells.
Falsifier: A padding, margin, gap or space class with a whole number N from 0 to 512 gives a length other than N cells in a direction it names, or leaves the style unchanged; or one with a fraction, such as `gap-0.25` or `p-1.5`, gives a length other than the next whole number of cells.
Mechanism: layout
Rationale: Until this requirement these classes counted in fours, so `p-1` was four cells while `w-1` was one, and a number outside the table, such as `gap-13`, was dropped without a message.
Status: Agreed 2026-09-29

[LAY-003] `gap-x-N` and `gap-y-N` MUST each set one direction and keep the other; `col-span-full` and `row-span-full` MUST span every track the grid has and add none; and `grid-cols-auto-fit-N` and `grid-cols-auto-fill-N` MUST make as many columns of at least N cells as the container holds, and at least one.
Falsifier: `gap-2 gap-x-1` leaves a gap between rows other than two cells, or `gap-2 gap-y-1` a gap between columns other than two; `col-span-full` or `row-span-full` in a grid of four tracks adds a fifth; or `grid-cols-auto-fit-20` or `grid-cols-auto-fill-20`, in a container of 90 cells that holds eight items, makes a number of columns other than four.
Mechanism: layout
Status: Agreed 2026-09-29

[LAY-004] Presenting an unchanged element tree at an unchanged size MUST give every element the same position and size again once the layout has settled, which it MUST do within three presents; no widget MAY take its size from its own last measured size in a way that changes its next measure.
Falsifier: With no input and no change of data after the first present, an element of the widget catalog's Input, Layout or Data display page has another position or size at the tenth present than at the fourth; or the data table, with its filter panel, its column panel or both open inside a parent whose height follows its content, differs in height between its fourth and its tenth present.
Mechanism: layout
Rationale: The developer reported on 2026-09-29 that the data table kept growing while its filter was open.
Status: Agreed 2026-09-29
