# Layout widgets

Crate modules: `widgets`

Widget module: `layout`

## Purpose

Layout widgets combine element layout with navigation, scrolling, disclosure,
and retained interaction state.

## Main API

- `Accordion` contains sections and supports single or multiple open sections.
- `Breadcrumb` renders path segments and handles limited widths through an
  overflow strategy.
- `ScrollView` tracks horizontal and vertical offsets around child content.
- `Stack` arranges children along one axis with spacing and padding.
- `Tabs` manages tab labels, active content, badges, and keyboard navigation.
- Props, state, builders, and supporting enums are exported from
  `widgets::layout`.

## Basic use

Build the widget's props or use its builder, add child elements or item data,
and place the result in the application tree. Supply stable item keys when the
set can change.

## Behavior

Accordion and tabs route keyboard and pointer actions to their current items.
Breadcrumb chooses a visible segment plan from its measured width. Scroll view
clamps offsets to the measured content and viewport. Stack maps its direction
and spacing into layout styles.

Some layout widgets use motion state for transitions. Reduced-motion handling
applies the final state without keeping animation clocks alive.

## Limits

- Scrolling requires content larger than the viewport.
- A zero-size viewport has no visible or interactive area.
- Breadcrumb overflow may replace middle segments according to its selected
  strategy.
- Tabs and accordion state must be reconciled when items are removed.

## Source map

- Layout widget exports: [`src/widgets/layout/mod.rs`](../src/widgets/layout/mod.rs)
- Scroll view: [`src/widgets/layout/scroll_view.rs`](../src/widgets/layout/scroll_view.rs)
- Tabs: [`src/widgets/layout/tabs.rs`](../src/widgets/layout/tabs.rs)
- Accordion behavior tests: [`tests/api_widget_behavior/accordion.rs`](../tests/api_widget_behavior/accordion.rs)
- Scroll behavior tests: [`tests/api_widget_behavior/scroll.rs`](../tests/api_widget_behavior/scroll.rs)
- Tabs behavior tests: [`tests/api_widget_behavior/tabs.rs`](../tests/api_widget_behavior/tabs.rs)

## Related chapters

- [Layout, style, and themes](layout-style-and-themes.md)
- [Animation and screens](animation-and-screens.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
