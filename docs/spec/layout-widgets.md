# Layout widgets

Prefix: NAV

The layout family is the widgets that arrange other content and move
through it: the tabs, the accordion, the breadcrumb, the scroll view and
the stack (src/widgets/layout, src/builder/widgets/layout.rs,
src/builder/widgets/accordion.rs, src/builder/widgets/breadcrumb.rs,
src/builder/specialized.rs). Each shows a set of places, panels or rows,
moves the current one on a key or a click, and reports the move through a
callback. Read on 2026-09-30, in the widget catalog under the dark and the
light preset at 240 and 100 columns: the tab with the focus carries a `▶`
and the one under the pointer a `→` (src/widgets/layout/tabs.rs:541-547),
a disabled tab is written `~label~` (564-566), the selected tab of the
`Soft` and `Solid` variants is `bg-gray-700` and `bg-blue-600 text-white`
(667-673) and a tooltip `bg-gray-800 text-white` (776); the accordion's
focused header carries a `▶ ` and a disabled one is `text-gray-500`
(src/widgets/layout/accordion/live.rs:113-116); the breadcrumb gives each
segment four cells of padding at each side
(src/widgets/layout/breadcrumb/live.rs:156), `/catalog/layout/widgets` in
a card of 85 cells shows as `Root / … / widgets` and in one of 36 cells
cuts `Root` to `Ro`, its earlier segments are underlined and its current
one bold (204-211), and its tooltip is `bg-gray-800 text-white` (404); the
scroll view is 80 by 24 cells unless the application sizes it
(src/widgets/layout/scroll_view.rs:146-147, 346-352; the builder has no
size setter, src/builder/specialized.rs:520-540), gives up a column to
its bar even when nothing overflows (185-193), paints the bar as text in
the page's color (455-470) and takes no click or drag on it; and a tab bar
wider than its parent is cut at the edge (781-783) while the arrow keys
still reach the tabs outside it. The keyboard already reaches every
action: arrows, Home and End move between tabs, sections and segments,
Enter and Space choose or toggle, Delete closes a tab, arrows, Page Up,
Page Down, Home and End scroll a scroll view. What the widget bar asks of
every reworked widget is in quality-bar.md and is not repeated here.
gpui-kit 0.7.0 paints a tab's label in `tab_foreground` and the active
one in `foreground` over a `tab_bar` fill, a breadcrumb in
`muted_foreground` with the last item in `foreground`, an accordion's
chevron in `muted_foreground`, and a scroll bar's thumb in
`scrollbar.thumb` over a transparent track that takes no layout space;
its tab bar scrolls when its tabs do not fit.

Read on 2026-10-12 for the resizable split, the sidebar and the log view,
three of the medium widgets of docs/widget-study.md (its entries
Resizable, Sidebar and Message scroller): no split and no log view exists;
`builder::sidebar()` is a styled box, `aside().class("w-64 bg-gray-800
text-white")` (src/builder/layout.rs:27-30), with palette colors and no
keys, state or role; the catalog builds its own navigation column and
stage (examples/widget_catalog/catalog.rs). In gpui-kit 0.7.0 (G7/ and B7/
are its component and base crates' sources) a resizable keeps each panel
between a minimum and a maximum, gives a drag's excess to the next panels
and keeps the panels' proportions when the container resizes
(B7/resizable/mod.rs:278-352), and paints its handle `border` at rest and
`ring` while active with no role and no key (B7/resizable/resize_handle.rs:460-468);
a sidebar is 255 pixels wide and 48 collapsed, its items tree items with a
selected state and no key handling (G7/sidebar/mod.rs:27-28,
G7/sidebar/menu.rs:294-296); a message scroller follows its last row,
offers a button to the latest rows while scrolled up, keeps the rows in
view still when rows are added above them, and is a Log
(G7/message_scroller.rs:31-38, 76-117, 209-210, 365-368).

## Observed

(none yet)

## Draft

[NAV-001] A layout widget whose style the application did not set MUST look the same whether it is built from its props or through any of its builders, and MUST paint: a tab's label in `text-muted` and the selected tab's in `foreground`, underlined in the `Line` variant, on `surface` in the `Enclosed` variant, on `secondary` with `secondary-foreground` text in the `Soft` variant and on `primary` with `primary-foreground` text in the `Solid` variant; a tab's badge in `text-success`, `text-warning`, `text-error` or `text-info` by its kind and a default badge in `text-muted`; an accordion's title in `foreground` and its glyph in `text-muted`; a breadcrumb's segments and separators in `text-muted` and its current segment in `foreground`, a clickable segment underlined; a scroll bar's track in `border` and its thumb in `text-muted`; a tooltip in `surface` with `foreground` text; the tab, section header or segment that holds the keyboard focus in `selection` with `selection-foreground` text while the widget holds the focus; the one under the pointer in `hover`; and a disabled tab, section or segment in `text-muted`. A layout widget MUST show focus, hover and disabled by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, default tabs, an accordion, a breadcrumb or a scroll view built through a builder paints a cell in another color than the same widget built from its props; a label, selected tab, badge, title, glyph, segment, separator, current segment, track, thumb, tooltip, focused item, hovered item or disabled item is painted in a color other than its role's; a widget paints a cell in a color no role of the theme has; or a widget with the focus, under the pointer or disabled paints a glyph outside its labels, or paints a label with other characters than the label's own.
Mechanism: layout-widgets
Rationale: A widget's state must be visible under every preset, and color is the cue a terminal has; the `▶`, `→` and tildes were stand-ins for it.
Status: Agreed 2026-09-30

[NAV-002] Tabs, an accordion, a breadcrumb and a stack MUST fill the width their parent allots, and a scroll view MUST fill the width and the height its parent allots when the application set no size; `viewport_width` and `viewport_height` on its props, and a `w-N`, `h-N`, `w-full` or `h-full` class on any of the builders, MUST set the size instead. A tab's label MUST have one cell of padding at each side in the `Medium` size, none in `Small` and two in `Large`; a breadcrumb's segment one cell at each side, none when compact. A scroll view MUST give a column or a row to its bar only while its content overflows in that direction.
Falsifier: In a viewport of 240 by 60 cells, default tabs, an accordion, a breadcrumb or a stack inside a box of 100 cells paints a bar, a header row, a trail or a child row that ends before the box's last cell; a default scroll view inside a box of 100 by 20 cells paints fewer than 100 columns or 20 rows, or `.class("w-40 h-10")` on its builder paints one of another size than 40 by 10 cells; a `Medium` tab paints other than one cell between its label and the next tab's padding, or a segment other than one cell between its label and the separator; or a scroll view whose content fits paints a bar or lays its content out in a viewport a column or a row smaller than its box.
Mechanism: layout-widgets
Rationale: BAR-003 asks a widget to fill the rectangle its parent allots; 80 by 24 was the terminal of 1985, and four cells of padding a pixel number read as cells.
Status: Agreed 2026-09-30

[NAV-003] A tab bar wider or taller than its parent MUST scroll so that the tab with the keyboard focus, or the selected tab when none has the focus, is whole in view, and a tab out of view MUST come into view when the keys or a click reach it. A breadcrumb whose trail does not fit MUST keep its first and its last segment whole and replace middle segments with an ellipsis before it cuts a label, unless its strategy says otherwise. A scroll view MUST scroll on the wheel, on a click on its bar's track by one page toward the click, and on a drag of its thumb by the thumb's travel, as it scrolls on its keys.
Falsifier: In a viewport of 240 by 60 cells, ten tabs of 20 cells inside a box of 100 cells have their focused or selected tab cut or out of view, or Right from the last visible tab leaves the next tab out of view; a breadcrumb of `/catalog/layout/widgets` with its icons inside a box of 50 cells paints a cut label or drops a segment, or one of five 12-cell segments inside a box of 40 cells cuts its first or its last label or shows no ellipsis; or a default scroll view of 60 rows of content in a box of 20 rows does not move on the wheel, on a click on its track below the thumb, or on a drag of its thumb.
Mechanism: layout-widgets
Rationale: gpui-kit's tab bar scrolls to the active tab (G7/tab/tab_bar.rs:545-546); ours cut the bar and let the keys move to tabs nobody could see.
Status: Agreed 2026-09-30

[NAV-004] The screen reader MUST be told each layout widget's name from the `aria_label` its props or its builder set, with no fixed English name; a tab's label, whether it is selected, its position and the count of tabs, and the panel's name from its tab's label; an accordion header's label, whether it is expanded, its position and the count of sections; a breadcrumb's segments as links with their labels, the current one marked as the current page, their position and count; a scroll view's offsets and their ranges; and disabled for any of them. Every action a layout widget takes from the pointer MUST have a key: a click on a tab, a header or a segment is Enter or Space, a click on a tab's close mark is Delete, and the wheel, a click on a scroll bar's track and a drag of its thumb are an arrow, Page Up, Page Down, Home or End.
Falsifier: A widget's accessibility node has a label although its props set no `aria_label`, or has none although they set one; a tab's node lacks its label, its selected state, its position or its set size, or its panel's node has no label; an accordion header's node lacks its label, its expanded state, its position or its set size; a segment's node lacks its label, its position or its set size, or the current segment is not marked current; a scroll view's node lacks its offsets or their ranges; a disabled item's node is not marked disabled; or an action of a layout widget can be taken by the pointer and by no key.
Mechanism: layout-widgets
Status: Agreed 2026-09-30

[NAV-005] A scroll view MUST decide its two bars together against the viewport each bar leaves: a bar whose row or column makes the content overflow in the other direction MUST bring that direction's bar; and the viewport size, the scroll limits, the painted bars and the pointer's track and thumb MUST all follow that one decision.
Falsifier: Fixed content of 10 columns by 20 rows in a scroll view of 10 by 5 cells with both directions enabled and bars shown paints a vertical bar and no horizontal bar, exposes a horizontal scroll limit while painting no horizontal bar, or paints a bar whose track takes no pointer.
Mechanism: review-widgets
Rationale: The developer's code review of 2026-10-04, W06: each bar was decided against the full box, so the column a vertical bar took could clip the content sideways with no horizontal bar to scroll it.
Status: Agreed 2026-10-08

[NAV-006] A breadcrumb's ellipsis MUST be a focusable stop that Left and Right reach in its place in the trail, and Enter, Space or a click on it MUST open a popup menu under it that lists the hidden segments in trail order; choosing one MUST act as a click on that segment. A tab bar with a tab out of view MUST paint an overflow mark at the far end of the bar, `»` on a horizontal bar and `⌄` on a vertical one, and Shift+F10 while the bar holds the focus, or a click on the mark, MUST open a popup menu that lists every tab in order with the selected tab checked; choosing one MUST select that tab and bring it whole into view. The mark MUST NOT be painted when every tab is in view, and both menus MUST follow MNU-001 to MNU-004.
Falsifier: Five 12-cell segments inside a box of 40 cells leave no focusable ellipsis, or Right from the first segment does not reach it, or Enter on it opens no menu, or the menu lacks a hidden segment or lists them out of order, or choosing one does not call that segment's click; ten tabs of 20 cells inside a box of 100 cells paint no `»` at the bar's end, or Shift+F10 on the focused bar opens no menu, or the menu lacks a tab or does not check the selected one, or choosing the tenth tab does not select it and bring it whole into view; or a bar whose tabs all fit paints the mark.
Mechanism: widget-behavior
Rationale: The keyboard reaches only the visible segments (src/widgets/layout/breadcrumb.rs:250-265) and the ellipsis is hidden text (src/widgets/layout/breadcrumb/live.rs:325-334); the tab bar scrolls (NAV-003) but offers no list of what is out of view, where gpui-kit's tab bar lists every tab in a menu (docs/widget-study.md, change 5).
Status: Agreed 2026-10-09

[NAV-007] A resizable split, a sidebar and a log view whose style the application did not set MUST look the same whether they are built from their props or through any of their builders, and MUST paint: a split's divider, a `│` between panes side by side or a `─` between stacked panes as the separator piece draws them, in `border`, on `hover` while it is under the pointer, and in `selection` with `selection-foreground` glyphs while it holds the keyboard focus or is dragged; a sidebar on `surface` with a `│` in `border` along the edge that faces the content, an item's label in `foreground`, its icon and a group's expander in `text-muted`, its badge as the badge piece paints it, the active item in `accent` with `accent-foreground` text, the item that holds the keyboard cursor in `selection` with `selection-foreground` text while the sidebar holds the focus, the item under the pointer in `hover`, a disabled item and the toggle in `text-muted`, and a collapsed sidebar's label beside the bar as NAV-001 paints a tooltip; a log view's rows in `foreground`, or in `text-info`, `text-success`, `text-warning` or `text-error` by their kind, its scroll bar as NAV-001 paints a scroll view's, and its `↓ N new` bar in `primary` with `primary-foreground` text. These widgets MUST show focus, hover, the active item and disabled by those colors alone, adding no glyph and changing no text.
Falsifier: Under a theme whose roles all differ, a default split, sidebar or log view built through a builder paints a cell in another color than the same widget built from its props; a divider, sidebar fill, sidebar edge, label, icon, expander, badge, active item, cursor item, hovered item, disabled item, toggle, collapsed label, row of each kind, track, thumb or `↓ N new` bar is painted in a color other than its role's; a widget paints a cell in a color no role of the theme has; or a widget paints another glyph or another text when it holds the focus, is under the pointer, has an active item or is disabled than when it does not.
Mechanism: layout-widgets
Rationale: NAV-001's rule for the family's three new widgets; gpui-kit's handle turns `ring` while active (B7/resizable/resize_handle.rs:460-468), where a terminal's cue for the focus is `selection`.
Status: Agreed 2026-10-12

[NAV-008] A resizable split, a sidebar and a log view MUST fill the height their parent allots, and a split and a log view the width, when the application set no size; a `w-N`, `h-N`, `w-full` or `h-full` class on any of their builders MUST set the size instead. A split's panes MUST share its length less one cell per divider in whole cells, by the sizes or weights the application gave them or equally when it gave none, none below its minimum, which is 4 cells unless the application sets another; when the split's length changes each pane MUST keep its share of it. A sidebar MUST be 28 cells wide, and 3 while collapsed, its header and footer, when the application gives them, above and below its items, its toggle on its last row, its items one row each, indented two cells per level, a label too long for its row cut with `…`, and a badge at the row's end. A log view's row wider than the view MUST continue on the next lines.
Falsifier: In a viewport of 240 by 60 cells, a default split of two panes inside a box of 100 by 20 cells paints panes whose widths do not sum to 99 cells or differ by more than one, or a pane or its divider outside the box; a split whose panes were 30 and 69 cells, its box widened to 200 cells, paints a first pane of other than 60 or 61 cells; a default sidebar inside a box of 100 by 20 cells is not 28 cells wide and 20 rows tall, or 3 cells wide while collapsed, or paints a child item's label other than two cells right of its group's, or a 40-character label uncut or cut without `…`; or a default log view inside a box of 100 by 20 cells paints fewer than 20 rows or a row that ends before the box's last cell, or a row of 250 characters other than on three lines.
Mechanism: layout-widgets
Rationale: BAR-003 asks a widget to fill what its parent allots; gpui-kit's sidebar is 255 pixels wide and 48 collapsed (G7/sidebar/mod.rs:27-28), here 28 cells, and one icon with a cell each side of it.
Status: Agreed 2026-10-12

[NAV-009] A split's divider MUST move on a drag, following the pointer by the cell, and while it holds the keyboard focus on Left and Right between panes side by side, or Up and Down between stacked panes, by one cell, by ten with Shift, and on Home and End to the farthest it can go; a move MUST grow the pane on one side by what it takes from the other, from the pane beside the divider down to its minimum and then from the next one beyond it, each down to its minimum, and MUST report the panes' sizes through `on_resize` once per move. A sidebar MUST move its cursor over its visible items on Up, Down, Home and End; Right MUST open a closed group and move into an open one, and Left close an open group and move from a child to its group; Enter, Space or a click on an item MUST make it the active item and report it through `on_select`, and on a group open or close it; its toggle, a focusable stop, MUST collapse and expand it on Enter, Space or a click and report the change through `on_collapse`; while collapsed it MUST show each visible item's icon, or its label's first character when it has none, and the label of the item under the cursor beside the bar while the sidebar holds the focus. A log view MUST follow its last row: rows appended while it follows MUST leave the last row on its last line; Up, Page Up, Home, the wheel up, a click on its track above the thumb or a drag of the thumb up MUST stop the following; End, a click on the `↓ N new` bar or scrolling down to the last row MUST resume it; while it does not follow and rows arrived since it stopped, its last line MUST be a bar reading `↓ N new` with the count of those rows, the word `new` replaced by the one the application gives; and rows prepended, or appended while it does not follow, MUST leave the rows in view on the lines they were on.
Falsifier: In a viewport of 240 by 60 cells, a split of two panes of 49 and 50 cells inside a box of 100 by 20 cells has a first pane other than 59 cells after a drag of its divider ten cells right, other than 50 after Right, 59 after Shift+Right or 95 after End with the divider focused, or reports no sizes or reports them twice for one move; a split of three panes of 33, 32 and 33 cells with minimums of 4 has panes other than 90, 4 and 4 after End on its first divider; a sidebar of Home, a closed group Projects holding Alpha and Beta, and Settings, with the cursor on Home, has it elsewhere than on Projects after Down, shows no Alpha after Right, has it elsewhere than on Alpha after another Right or elsewhere than on Projects after Left, reports no `on_select` for Settings after Enter on it or leaves another item active, or after Enter on its toggle is not 3 cells wide or reports no `on_collapse`; a collapsed sidebar paints an item's label inside the bar, or no label beside it for the item under the cursor; or a log view of 100 rows inside a box of 100 by 20 cells paints other than its last row on its last line after ten rows are appended, after Page Up and five appended rows paints another first line than before them or a last line that does not read `↓ 5 new`, or `↓ 5 neu` when the application gives the word `neu`, after End paints other than the last row on its last line or a bar, or after ten rows are prepended while it does not follow paints another first line than before.
Mechanism: layout-widgets
Rationale: The widget study's entries for the three (docs/widget-study.md); gpui-kit gives a drag's excess to the next panels (B7/resizable/mod.rs:278-348) and keeps a message scroller's anchor when rows are spliced (G7/message_scroller.rs:76-117), but gives its handle and its sidebar no keys.
Status: Agreed 2026-10-12

[NAV-010] The screen reader MUST be told each split's, sidebar's and log view's name from the `aria_label` its props or its builder set, with no fixed English name; a split's divider as a splitter with its orientation, the size of the pane before it as its value and the least and greatest it can take as its minimum and maximum, advertising Increment, Decrement and SetValue and acting on them as its arrow keys do; a sidebar as a navigation landmark holding a tree, each item a tree item with its label, level, position and count among its siblings, a group's expanded state, and the active item marked selected; its toggle as a button labelled by the sidebar's name, marked expanded while the sidebar is; a log view as a log that announces its rows politely, with its offsets and their ranges as a scroll view's, and its `↓ N new` bar as a button named by its text, in the application's word when it gives one, whose Click resumes the following; and disabled for any of them. Every action these widgets take from the pointer MUST have a key: a drag of a divider is an arrow, Home or End; a click on a sidebar item or its toggle is Enter or Space; a click on the `↓ N new` bar is End; and the wheel and a log view's track and thumb are an arrow, Page Up, Page Down, Home or End.
Falsifier: A widget's accessibility node has a label although its props set no `aria_label`, or has none although they set one; a divider's node is not a splitter or lacks its orientation, value, minimum, maximum or any of the three actions, or SetValue to 30 leaves a first pane other than 30 cells; a sidebar's node is not a navigation landmark holding a tree, an item's node lacks its label, level, position or count, a group's its expanded state, or the active item's is not selected; the toggle's node is not a button labelled by the sidebar's name or lacks its expanded state; a log view's node is not a log with polite announcements or lacks its offsets or their ranges; the bar's node is not a button named by its text, or its Click leaves the view not following; a disabled item's node is not marked disabled; or an action of these widgets can be taken by the pointer and by no key.
Mechanism: layout-widgets
Status: Agreed 2026-10-12

[NAV-011] On the Linux development host, with Orca, an X server and GNOME Terminal at hand, Orca MUST speak the medium widgets page of the accessibility probe this way: a split's divider named `Panes` with its first pane at 40 cells, given the focus, with `Panes`, and Right with `41`; a sidebar named `Main` of the items Home, a closed group Projects and Settings, with Home given the focus, with `Home`, Down with `Projects` and `collapsed`, and Right with `expanded`; a list named `Fruits` of 70,000 items labelled `Item 1` to `Item 70000`, given the focus, with `Item 1`, and End with `Item 70000`; and a log view named `Build log`, with the focus on the list, a row `Compiling widgets` appended to it with `Compiling widgets` while the focus stays on the list.
Falsifier: In a run of the probe's medium widgets page under Orca on a private X display and D-Bus session, the speech Orca logs after one of those steps lacks a phrase named for it, or the keyboard focus leaves the list when the log view's row is appended.
Mechanism: widgets-orca
Rationale: NAV-010's and DAT-009's checks read the published tree; a real reader proves that a splitter's value, a tree item's state, a list's options and a log's polite announcement reach the user, as the input pieces' Orca check found two defects their tree checks had missed (CTL-011, 2026-10-11).
Status: Agreed 2026-10-12
