# Layout widgets

Crate modules: `widgets`

Widget module: `layout`

## Purpose

Layout widgets arrange other content and move through it: the tabs, the
accordion, the breadcrumb, the scroll view, the stack, the resizable split,
the sidebar and the log view. Each shows a set of places, panels or rows,
moves the current one on a key or a click, and reports the move through a
callback. Their colors come from the theme's roles
(docs/spec/layout-widgets.md, NAV-001 and NAV-007): a tab's label, a
breadcrumb's segment and an accordion's glyph are `text-muted`, the
selected tab, the current segment and a title `foreground`, the item that
holds the keyboard focus `selection` while the widget has the focus, the
item under the pointer `hover`, and a disabled item `text-muted`. They fill
the width their parent allots, except the sidebar, which is 28 cells wide;
the scroll view, the split, the sidebar and the log view fill its height
too (NAV-002, NAV-008).

## Tabs

`builder::tabs()` takes `.tab(title, content)` for each tab, `.active(index)`
for the selected one, `.closable(true)` to let a tab close, `.class(..)` for
the container, `.aria_label(..)` for the screen reader's name of the tab
list and `.on_change(f)` for the new tab's index (CMP-009). `widgets::layout::TabsBuilder` adds `.variant(..)` (`Line`, the
default, underlines the selected label; `Enclosed` puts it on `surface`,
`Soft` on `secondary`, `Solid` on `primary`), `.size(..)` (`Small`, `Medium`
and `Large` give a label none, one or two cells of padding at each side),
`.position(..)` for a bar at the top, bottom, left or right,
`.keyboard_activation(..)`, `.lazy_loading(..)` and `.aria_label(..)`.

Left, Right, Up and Down move between tabs and wrap, Home and End reach the
ends, Enter and Space select the focused tab (arrows select at once unless
activation is `Manual`), Delete closes a closable tab, and the digits
1 to 9 select by position. A bar wider than its parent scrolls so the tab
with the focus, or the selected tab, is whole in view (NAV-003). Each tab
tells the screen reader its label, whether it is selected, its position and
the count of tabs; its panel is named by the tab's label (NAV-004).

When the tabs do not fit, the bar scrolls to keep the focused tab in view and paints an overflow mark at its far end, `»` on a horizontal bar and `⌄` on a vertical one; Shift+F10 while the bar holds the focus, or a click on the mark, opens a menu of every tab with the selected one checked, and choosing one selects it and brings it into view (NAV-006).

A tab can carry a badge: `TabBadge::new(text)` with `.with_variant(..)`
(`Default`, `Success`, `Warning`, `Error` or `Info`), added to the tab with
`Tab::with_badge(..)`. The badge piece draws it in its outline form, the
kind's mark from the icon catalog before the text, in the kind's text color:
`●` for `Default`, `•` for `Info`, `✓` for `Success`, `⚠` for `Warning` and
`×` for `Error`. The screen reader hears the badge's text as part of the
tab's description. The piece is described under
[Display widgets](display-widgets.md#badge).

## Accordion

`builder::accordion()` takes `.section(AccordionSection)` for each section
(`AccordionSection::new(id, title).content(element).expanded(bool)
.disabled(bool)`), `.mode(..)` for one or several open sections,
`.animated(..)`, `.icons(expand, collapse)` for the glyphs after a title,
`.keyboard_navigation(..)`, `.aria_label(..)` and `.on_change(f)`, called
with a section's index and whether it is open (CMP-009). `simple_accordion(vec![(id,
title, text)])` builds one from strings. Up and Down move between headers
and wrap, Home and End reach the ends, Enter and Space toggle the focused
section; a click on a header toggles it too. A header tells the screen
reader its label, whether it is expanded, its position and the count of
sections. The body opens with an animation that keeps the App under the
frame budget (BAR-005).

## Breadcrumb

`builder::breadcrumb()` takes `.segment(BreadcrumbSegment)` for each
segment (`BreadcrumbSegment::new(id, label, path).icon(..).current(true)`),
`.separator(..)`, `.show_icons(..)`, `.overflow_strategy(..)`, `.max_width(..)`
and `.aria_label(..)`; `path_breadcrumb("/a/b/c")` builds one from a path
with a home icon. A segment has one cell of padding at each side. Left and
Right move the focus between clickable segments, Home and End reach the
ends, Enter and Space follow the focused segment, as a click does. When
the trail does not fit, the default strategy keeps the first and the last
segment whole and replaces middle ones with an ellipsis before it cuts a
label (NAV-003). The root is a navigation landmark named by `aria_label`
only; each segment is a link with its label, its position and the count
of segments, and the current one is marked as the current page.

When the trail does not fit, the middle segments give way to an ellipsis that is a stop of its own: Left and Right reach it in its place, and Enter, Space or a click on it opens a menu of the hidden segments in trail order; choosing one acts as a click on that segment (NAV-006).

## Scroll view

`builder::scroll_view()` takes `.content(element)` or `.contents(vec)`,
`.vertical_scroll(bool)`, `.horizontal_scroll(bool)` (off by default, from
the props and either builder alike, so content wraps to the view's width),
`.show_scrollbars(bool)`,
`.viewport_size(width, height)` (or `.viewport_width(..)` and
`.viewport_height(..)`; 0, the default, takes the parent's size), `.class(..)`
and `.aria_label(..)`. The bar takes its column or row only while the
content overflows in that direction; its track is `border` and its thumb
`text-muted`. Arrows scroll by one cell, Page Up and Page Down by the
visible height, Home and End to the ends; the wheel scrolls (Shift for
sideways), a click on the track scrolls one page toward the click, and a
drag of the thumb scrolls with it. The screen reader is told the offsets
and their ranges.

## Stack

`builder::stack()` arranges its `.child(..)` or `.children(..)` along one
axis (`.direction(..)`) with `.spacing(..)` cells between them,
`.alignment(..)`, `.justify(..)`, `.padding(..)`, `.class(..)` and
`.aria_label(..)` for the screen reader's name. `widgets::layout::StackBuilder`
adds `StackBuilder::horizontal()` and `StackBuilder::vertical()` as starting
points, `.wrap(..)`, `.reverse(..)` and the `.padding_all(..)`,
`.padding_symmetric(..)`, `.padding_horizontal(..)` and `.padding_vertical(..)`
shorthands. A stack has no colors or state of its own, fills the width its
parent allots, and is named only by `aria_label`.

## Resizable split

`builder::resizable()` lays its panes side by side with `.horizontal()`,
the default, or stacks them with `.vertical()`, with a divider between
each pair. `.pane(element)` adds a pane that takes an equal share;
`.pane_with(Pane::new(element).size(40))` adds one of a fixed number of
cells, `.weight(2.0)` in place of `.size(..)` one with a share by weight of
the cells the fixed panes leave, and `.min(6)` sets the fewest cells a pane
can be moved down to (4 unless set). The builder also takes
`.aria_label(..)`, `.disabled(..)`, `.class(..)` and `.on_resize(f)`, called
with every pane's size after each move;
`Resizable::new(props).with_on_resize(f)` builds the same split from
`ResizableProps`. The split fills what its parent allots unless a `w-N` or
`h-N` class sizes it. Its length less one cell per divider is shared in
whole cells: fixed sizes first, then weights, then equal shares. When the
split's length changes, each pane keeps its share of it.

A divider is a `│` between panes side by side and a `─` between stacked
panes, drawn by the separator piece in `border`, on `hover` under the
pointer, and in `selection` with `selection-foreground` while it holds the
focus or is dragged. Each divider is a focus stop, and `.auto_focus()` on
the split's built element gives its first divider the focus when the page
opens. Left and Right, or Up
and Down between stacked panes, move it one cell, ten with Shift; Home and
End move it as far as it goes. A press on a divider takes the pointer, and
the drag moves the divider to the cell under it. A move takes cells from
the pane beside the divider down to its minimum, then from the next pane
beyond it, and gives them to the pane on the other side; `on_resize` is
called once per move. The screen reader hears each divider as a splitter
named by `aria_label`, with its orientation, the size of the pane before it
as its value, and the least and greatest size it can take; Increment,
Decrement and SetValue move it as the arrow keys do. After each move the
divider also announces the new size of the pane before it, the number
alone, as a polite live announcement, because Orca 50.1 speaks a named
splitter's role when its value changes but not the value.

Where the terminal takes pixels (PIX-009), a divider draws a picture in
place of `│` or `─`: a one-pixel `border` line along the middle of its
cells, with a rounded `text-muted` grip one cell long at its centre; while
it holds the focus or is dragged, the line is two pixels wide and the line
and the grip are in `ring`. There the sidebar's filled rows and the log
view's `↓ N new` bar also draw their fills with round ends.

## Sidebar

`builder::sidebar()` takes `.item(SidebarItem)` for each top-level item, or
`.items(vec)`, `.active(id)` for the active item, `.collapsed(bool)`,
`.header(element)` and `.footer(element)`, `.aria_label(..)`, `.class(..)`,
`.on_select(f)`, called with the id of the item the user makes active, and
`.on_collapse(f)`, called with the collapsed state after the user flips it.
`SidebarItem::new(id, label)` takes `.icon(Icon::Folder)`, `.badge("3")`,
`.disabled(true)`, `.open(true)` for a group that starts open, and
`.child(item)`; an item with children is a group.
`Sidebar::new(props).with_on_select(f).with_on_collapse(f)` builds the same
bar from `SidebarProps`.

The bar is 28 cells wide and fills the height its parent allots. It shows
the header, then one row per visible item, then the footer, and its toggle
on the last row, `‹` while the bar is expanded and `›` while it is
collapsed. An item is indented two cells per level, with the expander of a
group (`›` closed, `▼` open), its icon, its label cut with
`…` when it is too long, and its badge at the row's end. The bar is on
`surface` with a `│` in `border` along the edge that faces the content; a
label is `foreground`, an icon and an expander `text-muted`, the badge as
the badge piece paints it, the active item `accent` with
`accent-foreground` text, the item under the keyboard cursor `selection`
with `selection-foreground` text while the bar holds the focus, the item
under the pointer `hover`, and a disabled item and the toggle `text-muted`.

The item tree and the toggle are two focus stops. In the tree, Up, Down,
Home and End move the cursor over the visible items and pass disabled
ones; Right opens a closed group, or moves into an open one; Left closes an
open group, or moves from a child to its group; Enter and Space make the
item active, call `on_select`, and open or close a group. A click on an
item does what Enter does. On the toggle, Enter, Space or a click collapse
or expand the bar and call `on_collapse`. A collapsed bar is 3 cells wide
and shows each visible item's icon, or its label's first character; while
the tree holds the focus, the cursor item's label shows beside the bar as a
tooltip. The screen reader hears the bar as a navigation landmark named by
`aria_label`, holding a tree; each item is a tree item with its label, its
level, its position and the count of its siblings, a group's expanded
state, and the active item marked selected. The toggle is a button named
by `aria_label`, marked expanded while the bar is.

## Log view

`builder::log_view()` shows the rows of a `LogHandle` given with
`.handle(handle)`, and takes `.new_word(..)`, `.aria_label(..)` and
`.class(..)`. `LogHandle::new()` holds the rows: `.append(row)` and
`.append_all(rows)` add rows at the end, `.prepend(rows)` adds older rows
above, `.clear()` removes them, and `.len()` counts them. A clone of a
handle holds the same rows, so a clone on another thread can add rows;
each change wakes the App that shows the view, which redraws it.
`LogRow::new(text)` is a row in `foreground`; `.kind(LogKind::Error)` paints
it in `text-error`, and the kinds `Info`, `Success` and `Warning` in
`text-info`, `text-success` and `text-warning`.

The view fills the width and the height its parent allots, and a row wider
than the view continues on the next lines. It follows its last row while
rows arrive. Up, Page Up, Home, the wheel up, a click on the track above
the thumb or a drag of the thumb up stop the following; End, a click on the
`↓ N new` bar or scrolling down to the last row resume it. While the view
does not follow and rows arrived since it stopped, its last line is a bar
that reads `↓ N new` with the count of those rows, on `primary` with
`primary-foreground` text; `.new_word(..)` replaces the word `new`.
Prepended rows, and rows appended while the view does not follow, leave the
rows in view on their lines. The scroll bar and the keys are a scroll
view's. The view is one focus stop. The screen reader hears a log named by
`aria_label` that announces new rows politely, with the scroll offsets and
their ranges; the `↓ N new` bar is a button named by its text, and its
Click resumes the following.

## Source map

- Layout widget exports: [`src/widgets/layout/mod.rs`](../src/widgets/layout/mod.rs)
- The shared look: [`src/widgets/layout/look.rs`](../src/widgets/layout/look.rs)
- Tabs: [`src/widgets/layout/tabs.rs`](../src/widgets/layout/tabs.rs)
- Accordion: [`src/widgets/layout/accordion.rs`](../src/widgets/layout/accordion.rs), [`src/widgets/layout/accordion/live.rs`](../src/widgets/layout/accordion/live.rs)
- Breadcrumb: [`src/widgets/layout/breadcrumb.rs`](../src/widgets/layout/breadcrumb.rs), [`src/widgets/layout/breadcrumb/live.rs`](../src/widgets/layout/breadcrumb/live.rs)
- Scroll view: [`src/widgets/layout/scroll_view.rs`](../src/widgets/layout/scroll_view.rs)
- Stack: [`src/widgets/layout/stack.rs`](../src/widgets/layout/stack.rs)
- Resizable split: [`src/widgets/layout/resizable.rs`](../src/widgets/layout/resizable.rs), [`src/widgets/layout/resizable/share.rs`](../src/widgets/layout/resizable/share.rs)
- Sidebar: [`src/widgets/layout/sidebar.rs`](../src/widgets/layout/sidebar.rs), [`src/widgets/layout/sidebar/paint.rs`](../src/widgets/layout/sidebar/paint.rs), [`src/widgets/layout/sidebar/rows.rs`](../src/widgets/layout/sidebar/rows.rs)
- Log view: [`src/widgets/layout/log_view.rs`](../src/widgets/layout/log_view.rs), [`src/widgets/layout/log_view/handle.rs`](../src/widgets/layout/log_view/handle.rs), [`src/widgets/layout/log_view/wrap.rs`](../src/widgets/layout/log_view/wrap.rs)
- Builders: [`src/builder/widgets/layout.rs`](../src/builder/widgets/layout.rs), [`src/builder/widgets/accordion.rs`](../src/builder/widgets/accordion.rs), [`src/builder/widgets/breadcrumb.rs`](../src/builder/widgets/breadcrumb.rs), [`src/builder/specialized.rs`](../src/builder/specialized.rs), [`src/builder/widgets/resizable.rs`](../src/builder/widgets/resizable.rs), [`src/builder/widgets/sidebar.rs`](../src/builder/widgets/sidebar.rs), [`src/builder/widgets/log_view.rs`](../src/builder/widgets/log_view.rs)
- Contract tests: [`tests/layout_widgets_contract.rs`](../tests/layout_widgets_contract.rs); the split, the sidebar and the log view in [`tests/medium_widgets_contract.rs`](../tests/medium_widgets_contract.rs)
- Goldens: [`tests/layout_widgets_goldens.rs`](../tests/layout_widgets_goldens.rs), [`tests/snapshots/layout_widgets`](../tests/snapshots/layout_widgets)
- Behavior tests: [`tests/api_widget_behavior/accordion.rs`](../tests/api_widget_behavior/accordion.rs), [`tests/api_widget_behavior/breadcrumb.rs`](../tests/api_widget_behavior/breadcrumb.rs), [`tests/api_widget_behavior/scroll.rs`](../tests/api_widget_behavior/scroll.rs), [`tests/api_widget_behavior/stack.rs`](../tests/api_widget_behavior/stack.rs), [`tests/api_widget_behavior/tabs.rs`](../tests/api_widget_behavior/tabs.rs)
- Screen-reader probe page of the split, the sidebar, the list and the log view (NAV-011): [`tests/api_widget_behavior/medium_probe.rs`](../tests/api_widget_behavior/medium_probe.rs), with what Orca must say in [`tests/api_widget_behavior/orca_medium.py`](../tests/api_widget_behavior/orca_medium.py)

## Related chapters

- [Layout, style, and themes](layout-style-and-themes.md)
- [Animation and screens](animation-and-screens.md)
- [Events, focus, and input](events-focus-and-input.md)
- [Display pieces](display-widgets.md#display-pieces), for the badge piece on each tab

[Back to the manual](README.md)
