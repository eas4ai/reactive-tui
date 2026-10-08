# Layout widgets

Crate modules: `widgets`

Widget module: `layout`

## Purpose

Layout widgets arrange other content and move through it: the tabs, the
accordion, the breadcrumb, the scroll view and the stack. Each shows a set
of places, panels or rows, moves the current one on a key or a click, and
reports the move through a callback. Their colors come from the theme's
roles (docs/spec/layout-widgets.md, NAV-001): a tab's label, a breadcrumb's
segment and an accordion's glyph are `text-muted`, the selected tab, the
current segment and a title `foreground`, the item that holds the keyboard
focus `selection` while the widget has the focus, the item under the
pointer `hover`, and a disabled item `text-muted`. They fill the width their
parent allots, and the scroll view its height too (NAV-002).

## Tabs

`builder::tabs()` takes `.tab(title, content)` for each tab, `.active(index)`
for the selected one, `.closable(true)` to let a tab close, `.class(..)` for
the container and `.aria_label(..)` for the screen reader's name of the tab
list. `widgets::layout::TabsBuilder` adds `.variant(..)` (`Line`, the
default, underlines the selected label; `Enclosed` puts it on `surface`,
`Soft` on `secondary`, `Solid` on `primary`), `.size(..)` (`Small`, `Medium`
and `Large` give a label none, one or two cells of padding at each side),
`.position(..)` for a bar at the top, bottom, left or right,
`.keyboard_activation(..)`, `.lazy_loading(..)` and `.aria_label(..)`.

Left, Right, Up and Down move between tabs and wrap, Home and End reach the
ends, Enter and Space select the focused tab (arrows select at once unless
activation is `Manual`), Delete or `x` closes a closable tab, and the digits
1 to 9 select by position. A bar wider than its parent scrolls so the tab
with the focus, or the selected tab, is whole in view (NAV-003). Each tab
tells the screen reader its label, whether it is selected, its position and
the count of tabs; its panel is named by the tab's label (NAV-004).

## Accordion

`builder::accordion()` takes `.section(AccordionSection)` for each section
(`AccordionSection::new(id, title).content(element).expanded(bool)
.disabled(bool)`), `.mode(..)` for one or several open sections,
`.animated(..)`, `.icons(expand, collapse)` for the glyphs after a title,
`.keyboard_navigation(..)` and `.aria_label(..)`. `simple_accordion(vec![(id,
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

## Source map

- Layout widget exports: [`src/widgets/layout/mod.rs`](../src/widgets/layout/mod.rs)
- The shared look: [`src/widgets/layout/look.rs`](../src/widgets/layout/look.rs)
- Tabs: [`src/widgets/layout/tabs.rs`](../src/widgets/layout/tabs.rs)
- Accordion: [`src/widgets/layout/accordion.rs`](../src/widgets/layout/accordion.rs), [`src/widgets/layout/accordion/live.rs`](../src/widgets/layout/accordion/live.rs)
- Breadcrumb: [`src/widgets/layout/breadcrumb.rs`](../src/widgets/layout/breadcrumb.rs), [`src/widgets/layout/breadcrumb/live.rs`](../src/widgets/layout/breadcrumb/live.rs)
- Scroll view: [`src/widgets/layout/scroll_view.rs`](../src/widgets/layout/scroll_view.rs)
- Stack: [`src/widgets/layout/stack.rs`](../src/widgets/layout/stack.rs)
- Builders: [`src/builder/widgets/layout.rs`](../src/builder/widgets/layout.rs), [`src/builder/widgets/accordion.rs`](../src/builder/widgets/accordion.rs), [`src/builder/widgets/breadcrumb.rs`](../src/builder/widgets/breadcrumb.rs), [`src/builder/specialized.rs`](../src/builder/specialized.rs)
- Contract tests: [`tests/layout_widgets_contract.rs`](../tests/layout_widgets_contract.rs)
- Goldens: [`tests/layout_widgets_goldens.rs`](../tests/layout_widgets_goldens.rs), [`tests/snapshots/layout_widgets`](../tests/snapshots/layout_widgets)
- Behavior tests: [`tests/api_widget_behavior/accordion.rs`](../tests/api_widget_behavior/accordion.rs), [`tests/api_widget_behavior/breadcrumb.rs`](../tests/api_widget_behavior/breadcrumb.rs), [`tests/api_widget_behavior/scroll.rs`](../tests/api_widget_behavior/scroll.rs), [`tests/api_widget_behavior/stack.rs`](../tests/api_widget_behavior/stack.rs), [`tests/api_widget_behavior/tabs.rs`](../tests/api_widget_behavior/tabs.rs)

## Related chapters

- [Layout, style, and themes](layout-style-and-themes.md)
- [Animation and screens](animation-and-screens.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
