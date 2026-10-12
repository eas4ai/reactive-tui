# Data widgets

Crate modules: `widgets`

Widget module: `display`

## Purpose

Data widgets show a set of records: the table, the data table, the tree,
the file explorer, the progress bar and the list. Each shows rows of a set,
or one value's share of a range, moves the current row on a key or a click,
and reports a choice through a callback. Their colors come from the theme's
roles (docs/spec/data-widgets.md, DAT-001 and DAT-006): a box sits on its parent's
background with no fill of its own and its border in `border`; a header's
title is `foreground` and the sorted column's mark `primary`; a row's text
is `foreground`; the row that holds the keyboard cursor is `selection`
while the widget has the focus, a selected row `accent`, the row under the
pointer `hover`, a disabled row `text-muted`; a tree's lines and expanders,
a file explorer's hints and sizes and a data table's page count are
`text-muted`; an error line is `text-error`; a progress bar's filled part
is `primary` on a `border` track. Focus, hover and selection show by color
alone. The table, the data table, the tree, the file explorer and the list
fill the width and the height their parent allots, and the progress bar the
width; `width` and `height` on the props, or a `w-N` or `h-N` class on a
builder, set a size instead (DAT-002, DAT-007). Every
widget is named for the screen reader by `aria_label` on its props or its
builder, and has no fixed English name without one (DAT-004).

## Table

`Table::with_props(TableProps { .. })` takes `TableColumn::new(title, key)`
for each column (`.with_width(..)`, `.with_alignment(..)`, `.sortable(..)`,
`.resizable(..)`) and `TableRow::new(id)` for each row (`.with_cell(key,
text)`, `.with_styled_cell(..)`, `.with_data(..)`, `.with_style(..)`,
`.selectable(..)`); `TableCell::new(text)` adds `.with_style(..)`,
`.with_alignment(..)` and `.clickable(action)`. The props carry
`sortable`, `selectable`, `multi_select`, `border`, `show_header`,
`zebra_striping`, `aria_label`, `width`, `height` and `max_height`, and the
`on_select`, `on_multi_select`, `on_sort` and `on_row_action` callbacks. A
column is `DisplaySize::Flex(1.0)` unless told otherwise: the columns share
the table's width by their weights, each at least as wide as its title and
its sort mark, so two or three default columns fit a box of 100 cells;
`Fixed`, `Percent` and `Auto` (the content's width) are the other sizes.
The table scrolls sideways only when the columns' minimums exceed its
width (DAT-003).

Up and Down move the cursor row and select it, Home and End reach the
ends, Page Up and Page Down move by a page; Enter reports the cursor row
through `on_row_action` with `select`, or with a clickable cell's own
action when the column cursor is on one (`.clickable(action)`), as a click
on the cell does; Space selects the row (in `multi_select` it toggles the
row, and Ctrl+A selects every row). Left and Right move the column cursor
and bring its column into view; `s` sorts by that column, as a click on
its header does, and a second `s` reverses the order; with
`resizable_columns` set, Shift with Left or Right narrows or widens the
cursor's column by a cell, as a drag of its header's edge does. A column
whose every cell is a number sorts by its numbers, so 2, 9 and 10 keep
that order, and integers compare exactly however long they are; other
columns sort by their text (DAT-003). The cursor starts
on the first row when the table takes the focus with nothing selected. The
screen reader is told the table's name, its row and column counts, each
row's index, each cell's column index and the sorted column's direction
(DAT-004).

## Data table

`builder::data_table()` wraps a table in a search field, a toolbar and
page controls: `.column(title, key)` or `.column_with_config(..)` for each
column, `.simple_row(vec![(key, text)])`, `.row(..)` or `.rows(..)` for the
rows, `.pagination(enabled, page_size)` (pages are off unless asked for,
and a viewport height of 0, the default of `.virtual_scroll(enabled,
row_height, viewport_height)`, is the table's own height, so no default
caps the rows it shows),
`.features(searchable, filterable, exportable)`, `.hide_columns(..)`,
`.filter(key, filter_type)`, `.class(..)` and `.aria_label(..)`. Its table
sorts and colors as the table does; Shift with a click on a header, or
with `s`, adds a secondary sort, and each mark then carries its priority;
when pages are on, the row under the table is the pagination bar, `‹ 1 2 3 ›`,
with the current page on `primary`, followed by the filtered row count in
`text-muted`, such as ` (25)`. The arrows `‹` and `›` step one page; the
bar has no separate Prev and Next buttons. Left and Right move a page, Home
and End go to the first and the last page, and Confirm or a click chooses a
page. An ellipsis `…` opens a popup menu of the pages it hides, and a choice
there moves the table to that page. The filter and column panels and the
export buttons keep their behavior. `.on_page_change(f)` is called with
the 0-based page each time the page changes.

## Tree

`builder::tree()` takes `.root(node)`, `.selectable(..)`,
`.multi_select(..)`, `.show_icons(..)`, `.show_lines(..)`,
`.checkable(..)`, `.class(..)` and `.aria_label(..)`; `TreeNode::new(id,
label)` nests `.add_child(..)` or `.children(vec)` and starts
`.expanded(true)`, `.selected(true)` or `.checked(bool)`.
`widgets::display::TreeBuilder` adds `.indent_size(..)`,
`.lazy_loading(..)` with `.on_load_children(..)`, `.drag_drop(..)`,
`.search(..)` and `.filter_visible(..)`, `.border(..)`, `.scrollable(..)`,
`.max_height(..)`, `.virtual_scrolling(..)`, the `.node_style(..)`,
`.selected_style(..)`, `.expanded_style(..)`, `.leaf_style(..)` and
`.line_style(..)` overrides, and the `.on_select(..)`, `.on_expand(..)`,
`.on_multi_select(..)`, `.on_check(..)` and `.on_node_action(..)`
callbacks. Lines, expanders and checkbox frames are `text-muted` and a
checked mark `primary`.

Up and Down move the cursor and select, Home and End reach the ends, Page
Up and Page Down move by a page; Right expands the cursor's node or moves
to its first child, Left collapses it or moves to its parent, `+` and `-`
expand and collapse, `*` expands every node at the cursor's level, Enter
toggles and reports the node, Space checks a checkable node. A node the
application selects, at first or later, is revealed: the ancestors that
hide it open, it takes the cursor and scrolls into view; collapsing the
parent of the cursor's node moves the cursor to that parent (DAT-003). Each row tells the screen reader its
label, its level, its position among its siblings and their count, whether
it is expanded and whether it is selected (DAT-004).

## File explorer

`builder::file_explorer()` takes `.root_path(..)` and `.current_path(..)`,
`.show_hidden(..)`, `.file_filters(vec)`, `.selection_mode(..)`,
`.view_mode(..)` (a list, a tree or a grid), `.sort_by(..)` and
`.sort_order(..)`, `.show_preview(..)`, `.show_breadcrumb(..)`,
`.show_details(..)`, `.search(..)`, `.keyboard_navigation(..)`,
`.max_visible_items(..)`, `.class(..)`, `.aria_label(..)` and the
`.on_select(..)`, `.on_activate(..)` and `.on_navigate(..)` event names;
`simple_file_browser(path)`, `code_project_explorer(path)` and the other
presets start from common settings. A row shows an entry's mark and name
in `foreground` and, with details on, its size and date in `text-muted`.
The mark comes from the icon catalog and is one cell wide: `▪` for a
folder, `▾` for a folder that the tree view shows expanded, and `▫` for a
file, a link or an entry of unknown kind. Where the terminal reports no
Unicode the marks are `+`, `-` and `.`. The toolbar's words, the status
line and the earlier directories of the breadcrumb are `text-muted` too,
and the status line is `text-error` while it carries an error.

Up and Down move the cursor and select, Home, End, Page Up and Page Down
as in the tree, Enter opens the entry, Space toggles it in the selection,
Backspace goes up a directory; in the tree view Right expands a directory
and Left collapses it. Each toolbar word has its key: `v` changes the view,
`s` the sort key, `o` the sort order, `.` shows hidden files, `/` starts a
search, `p` toggles the preview, F5 refreshes, F2 renames, F6 moves, F7
copies and Delete deletes, after a confirmation. The screen reader is told
the explorer's name and each row's name, position and count (DAT-004).

## Progress bar

`builder::progress_bar()` takes `.value(..)` and `.max_value(..)`,
`.label(..)`, `.show_percentage(..)`, `.animated(..)`, `.color(..)`,
`.width(..)`, `.class(..)` and `.aria_label(..)`.
`widgets::display::ProgressBarBuilder` adds `.min_value(..)`,
`.range(..)`, `.show_value(..)`, `.indeterminate(..)`,
`.background_color(..)`, `.height(..)`, `.orientation(..)` or
`.vertical()`, `.segments(..)`, `.striped(..)`, `.pulse(..)`,
`.bar_style(..)`, `.text_style(..)`, `.custom_formatter(..)` and
`.on_complete(..)`. Without a color the filled part is `primary` and the
track `border`; the label and the value are `foreground`. The bar fills
the width its parent allots unless `.width(..)` sets one. The screen
reader is told its name (`aria_label`, else the label), its value, its
minimum and its maximum; an indeterminate bar keeps its range and has no
value (DAT-004). An indeterminate bar's marker moves
every frame within the frame budget (BAR-005).

Where the terminal takes pixels (see [Input widgets](input-widgets.md#pixel-looks)),
a horizontal bar without a color of its own, stripes, segments or a pulse
is one picture over its rows: a track with a radius of half its height in
`border` and its filled part in `primary` to the exact pixel of its value
(`docs/spec/pixel-looks.md`, PIX-005). An indeterminate bar's segment, a
quarter of the track long, glides from end to end and back over two
seconds at the App's frame rate, one picture a frame; with
`.style("reduced-motion")` it steps a quarter of the track once a second.
Its label and value stay cell text, and a bar whose configuration fails
validation shows its error line in cells and no track. A bar with its own
colors, stripes, segments or a pulse, and a vertical bar, keep their
cells.

## List

`builder::list()` takes `.item(ListItem)` for each item, or `.items(vec)`,
`.selected(id)` for the selected item, `.filterable(true)` for a filter row
on top, `.has_more(true)` when the application has more items to give,
`.loading(true)` for a loading row at the end, `.empty_text(..)` for the
message shown when no item shows, `.aria_label(..)`, `.width(..)`,
`.height(..)` and `.class(..)`, and the callbacks `.on_select(f)`, called
with the id of the item the cursor moves to, `.on_confirm(f)`, called with
the id of the item Enter or a double click confirms, `.on_cancel(f)` and
`.on_load_more(f)`. `ListItem::new(id, label)` takes `.description(..)`,
shown after the label, `.section(..)`, the title of the section the item
sits under (consecutive items with one section share one title row), and
`.disabled(true)`. `List::new(props)` with `.with_on_select(f)`,
`.with_on_confirm(f)`, `.with_on_cancel(f)` and `.with_on_load_more(f)`
builds the same list from `ListProps`.

The list fills the width and the height its parent allots, one row per
item or section title, and builds elements only for the rows in view and
at most ten beyond them at each side, so a list of 100,000 items costs no
more to draw than one of a hundred. Its box has no fill of its own; a
label is `foreground`, a description and a section title `text-muted`, the
cursor's item `selection` with `selection-foreground` text while the list
holds the focus, the selected item `accent` with `accent-foreground` text,
the item under the pointer `hover`, a disabled item `text-muted`, and the
characters of a label that match the filter `primary`. The filter row is
painted as a text input's field, the loading row is the spinner piece, and
the empty view is the empty piece with `empty_text`. Where the terminal
takes pixels (PIX-009), the list's filled rows draw their fills with round
ends.

The list is one focus stop. Up and Down move the cursor one item, Page Up
and Page Down by the items its height holds, Home and End to the first and
the last item, passing over section titles and disabled items and keeping
the cursor's item in view; each move calls `on_select`. A click on an item
moves the cursor there and gives the list the focus; Enter, or a double
click, selects the cursor's item and calls `on_confirm`; the wheel scrolls.
With a filter, a typed character goes into the filter and Backspace takes
the last one out; the list shows the items whose label holds the filter's
text, letters compared without case, under the titles of the sections that
keep an item, with the cursor on the first of them. Escape empties a
filter that holds text, and otherwise calls `on_cancel`. While `has_more`
is set, the list calls `on_load_more` once, until its items change, when
the cursor or the view's last row comes within twenty items of the last
item. The screen reader hears a list box named by `aria_label`; each item in view is
an option with its label, its position among the items the list shows and
their count, and whether it is selected; a section is a group named by its
title, the filter a search input named by the list, and a disabled item is
marked disabled.

## Source map

- The shared look: [`src/widgets/display/look.rs`](../src/widgets/display/look.rs)
- Table: [`src/widgets/display/table.rs`](../src/widgets/display/table.rs), [`src/widgets/display/table/live.rs`](../src/widgets/display/table/live.rs), [`src/widgets/display/table/border.rs`](../src/widgets/display/table/border.rs)
- Data table: [`src/widgets/display/data_table.rs`](../src/widgets/display/data_table.rs), [`src/widgets/display/data_table/live.rs`](../src/widgets/display/data_table/live.rs), [`src/widgets/display/data_table/filters.rs`](../src/widgets/display/data_table/filters.rs)
- Tree: [`src/widgets/display/tree.rs`](../src/widgets/display/tree.rs), [`src/widgets/display/tree/live.rs`](../src/widgets/display/tree/live.rs), [`src/widgets/display/tree/live/paint.rs`](../src/widgets/display/tree/live/paint.rs)
- File explorer: [`src/widgets/display/file_explorer.rs`](../src/widgets/display/file_explorer.rs), [`src/widgets/display/file_explorer/live.rs`](../src/widgets/display/file_explorer/live.rs), [`src/widgets/display/file_explorer/live/paint.rs`](../src/widgets/display/file_explorer/live/paint.rs)
- Progress bar: [`src/widgets/display/progress_bar.rs`](../src/widgets/display/progress_bar.rs), [`src/widgets/display/progress_bar/live.rs`](../src/widgets/display/progress_bar/live.rs)
- List: [`src/widgets/display/list.rs`](../src/widgets/display/list.rs), [`src/widgets/display/list/lines.rs`](../src/widgets/display/list/lines.rs), [`src/widgets/display/list/paint.rs`](../src/widgets/display/list/paint.rs)
- Builders: [`src/builder/widgets/table.rs`](../src/builder/widgets/table.rs), [`src/builder/widgets/display.rs`](../src/builder/widgets/display.rs), [`src/builder/widgets/file_explorer.rs`](../src/builder/widgets/file_explorer.rs), [`src/builder/specialized.rs`](../src/builder/specialized.rs), [`src/builder/widgets/list.rs`](../src/builder/widgets/list.rs)
- Contract tests: [`tests/data_widgets_contract.rs`](../tests/data_widgets_contract.rs); the list in [`tests/medium_widgets_contract.rs`](../tests/medium_widgets_contract.rs)
- Goldens: [`tests/data_widgets_goldens.rs`](../tests/data_widgets_goldens.rs), [`tests/snapshots/data_widgets`](../tests/snapshots/data_widgets)
- Behavior tests: [`tests/api_widget_behavior/table.rs`](../tests/api_widget_behavior/table.rs), [`tests/api_widget_behavior/data_table.rs`](../tests/api_widget_behavior/data_table.rs), [`tests/api_widget_behavior/tree.rs`](../tests/api_widget_behavior/tree.rs), [`tests/api_widget_behavior/file_explorer.rs`](../tests/api_widget_behavior/file_explorer.rs), [`tests/api_widget_behavior/progress.rs`](../tests/api_widget_behavior/progress.rs)

## Related chapters

- [Display widgets](display-widgets.md)
- [Display pieces](display-widgets.md#display-pieces), for the pagination bar and the icon catalog
- [Layout, style, and themes](layout-style-and-themes.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
