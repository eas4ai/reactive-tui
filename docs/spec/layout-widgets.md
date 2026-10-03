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
