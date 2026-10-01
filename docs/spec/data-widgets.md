# Data widgets

Prefix: DAT

The data family is the widgets that show a set of records: the table, the
data table, the tree, the file explorer and the progress bar
(src/widgets/display/table.rs, data_table.rs, tree.rs, file_explorer.rs,
progress_bar.rs and their directories; src/builder/widgets/table.rs,
display.rs, file_explorer.rs, src/builder/specialized.rs). Each shows rows
of a set, or one value's share of a range, moves the current row on a key
or a click, and reports a choice through a callback. Read on 2026-09-30,
in the widget catalog under the dark and the light preset at 240 and 100
columns: the table, the data table and the tree sit in a grey box with a
white border under every preset, since the `border` class paints a fixed
grey fill (src/layout/css/effects.rs:51-60; src/widgets/display/table/live.rs:557-558),
so under the light preset their text is dark on dark; the table shows one
of its two columns in a card of 85 cells, since a column's minimum is 50
cells (src/widgets/display/table.rs:661) and 100 through
`builder::data_table()` (src/builder/widgets/table.rs:61); no row shows
focus, selection or hover: the selected row's style is `bg-blue fg-white`
(src/widgets/display/table.rs:166; src/widgets/display/tree.rs:431, 527)
and the tree's lines `fg-gray` (tree.rs:434, 530), and `fg-` is no class
the parser knows, while `hover_row` and `hover_node` are kept and never
painted (table.rs:231, 251; src/widgets/display/tree/live.rs:818, 835);
the data table shows at most thirteen rows, its `row_height` of 32 and
`viewport_height` of 400 read as cells (src/widgets/display/data_table.rs:352-358;
src/widgets/display/data_table/live.rs:528-532); the file explorer paints
its selection `bg-blue-600 text-white` (src/widgets/display/file_explorer/live/paint.rs:340)
and an error `bg-black text-red-400` (358), and names itself "File
explorer" (448); the progress bar is `bg-blue` on `bg-gray-200`
(src/widgets/display/progress_bar.rs:233-234, 308-309), its error line
`text-red-500` (src/widgets/display/progress_bar/live.rs:65), and it is
named "Progress" when it has no label (188); error lines are `text-red-500`
in the table, the data table and the tree (table/live.rs:372;
data_table/live.rs:688, 704; data_table/filters.rs:144; tree/live/paint.rs:259).
The table sorts every column as text, so "10" sorts before "9"
(table.rs:356; data_table/live.rs:143-159); a tree node selected under a
collapsed parent stays hidden and the cursor falls back to the first row
(tree/live.rs:228-232). The keyboard already reaches most actions: arrows,
Home, End, Page Up and Page Down move the row cursor, Enter and Space
choose, Ctrl+A selects all, Left, Right, `+`, `-` and `*` fold a tree, and
the file explorer's toolbar words each have a key; a click on a table's
header sorts by that column and no key does. What the widget bar asks of
every reworked widget is in quality-bar.md and is not repeated here.
gpui-kit 0.7.0 paints a table's head in `table.head.foreground` on a
`table.head.background` fill, a hovered row in `table.hover`, the active
row in `list.active` with a `selection` border, a list item's selection
in `list.active` or `accent`, and a progress bar in `progress_bar`; its
table's columns default to 100 px wide with a minimum of 20 px.

## Observed

(none yet)

## Draft

[DAT-001] A data widget whose style the application did not set MUST look the same whether it is built from its props or through any of its builders, and MUST paint: a table's, a data table's, a tree's or a file explorer's box in its parent's background with no fill of its own and its border, when it has one, in `border`; a header's title in `foreground` and the sorted column's mark in `primary`; a row's text in `foreground`; a selected row in `accent` with `accent-foreground` text; the row that holds the keyboard cursor in `selection` with `selection-foreground` text while the widget holds the focus; the row under the pointer in `hover`; a tree's lines, expanders and checkbox frames, a file explorer's hints and sizes, and a data table's page count in `text-muted`; a disabled row in `text-muted`; an error line in `text-error`; a progress bar's filled part in `primary`, its track in `border`, its label and value in `foreground`. A data widget MUST show focus, hover and selection by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, a default table, data table, tree, file explorer or progress bar built through a builder paints a cell in another color than the same widget built from its props; a box, border, title, sort mark, row, selected row, cursor row, hovered row, line, expander, hint, page count, disabled row, error line, filled part, track, label or value is painted in a color other than its role's; a box paints a fill of its own; a widget paints a cell in a color no role of the theme has; or a widget with the focus, under the pointer or with a selection paints a glyph outside its rows, or paints a row with other characters than the row's own.
Mechanism: data-widgets
Rationale: A row's state must be visible under every preset, and color is the cue a terminal has; `bg-blue fg-white` was a palette color beside a class the parser drops.
Status: Agreed 2026-09-30

[DAT-002] A table, a data table, a tree and a file explorer MUST fill the width and the height their parent allots, and a progress bar the width, when the application set no size; `width` and `height` on the props and a `w-N`, `h-N`, `w-full` or `h-full` class on any of the builders MUST set the size instead. A table's columns MUST share its width by their weights, each at least as wide as its title and its sort mark, so a default table of two or three columns fits a box of 100 cells; a data table MUST show as many rows as its height holds, with no default cap on their number; a tree and a file explorer MUST show as many rows as their height holds.
Falsifier: In a viewport of 240 by 60 cells, a default table of two columns inside a box of 100 by 10 cells paints fewer than two column titles or a row that ends before the box's last cell; a default data table of 40 rows inside a box of 100 by 30 cells paints fewer than 20 rows or a row that ends before the box's last cell; a default tree of 20 nodes or a file explorer of 20 entries inside a box of 100 by 12 cells paints fewer than 8 rows or a row narrower than the box; a default progress bar inside a box of 100 cells paints a track that ends before the box's last cell; or `.class("w-40")` on a builder paints a widget of another width than 40 cells.
Mechanism: data-widgets
Rationale: BAR-003 asks a widget to fill the rectangle its parent allots; 50 and 100 cells and 32 by 400 were pixel numbers read as cells.
Status: Agreed 2026-09-30

[DAT-003] A table and a data table MUST sort a column whose every cell is a number by its numbers, and other columns by their text; a table MUST scroll sideways only when its columns' minimums exceed its width. A tree MUST reveal a node that the application or the keyboard selects, expanding the ancestors that hide it, and when an ancestor of the cursor's node is collapsed the cursor MUST move to that ancestor.
Falsifier: A column of "9", "10" and "2" sorted ascending reads 10, 2, 9 or 2, 9, 10 in another order than 2, 9, 10; a table whose columns fit its width paints a cell cut by a sideways scroll; a tree whose props select a node under a collapsed parent paints no row for that node; or collapsing the parent of the cursor's node leaves the cursor on the first row instead of that parent.
Mechanism: data-widgets
Rationale: Three defects of docs/widget-study.md (sorting as text, column minimums, the hidden selection) are fixed with the family, as the roadmap of 2026-09-26 ordered.
Status: Agreed 2026-09-30

[DAT-004] The screen reader MUST be told each data widget's name from the `aria_label` its props or its builder set, or a progress bar's label, with no fixed English name; a table's and a data table's row and column counts, each row's index, each cell's column index and the sorted column's direction; a tree row's label, level, position and count among its siblings, whether it is expanded and whether it is selected; a file explorer's rows with their labels, positions and count; a progress bar's value, minimum and maximum; and disabled for any of them. Every action a data widget takes from the pointer MUST have a key: a click on a column header sorts by it, so Left and Right MUST move the column cursor and `s` MUST sort by it; a click on a tree's expander is Left, Right, `+` or `-`, on its checkbox Space; a click on a row is Enter or Space; a click on a file explorer's toolbar word is that word's key.
Falsifier: A widget's accessibility node has a label although its props set no `aria_label` (or, for a progress bar, no label), or has none although they set one; a table's node lacks its row or column count, a row's node its index, a cell's node its column index, or the sorted header its direction; a tree row's node lacks its level, position, count, expanded or selected state; a file explorer row's node lacks its position or count; a progress bar's node lacks its value, minimum or maximum; a disabled row's node is not marked disabled; or an action of a data widget can be taken by the pointer and by no key.
Mechanism: data-widgets
Status: Agreed 2026-09-30
