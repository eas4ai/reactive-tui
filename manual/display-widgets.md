# Display widgets

Crate modules: `widgets`

Widget module: `display`

## Purpose

Display widgets present structured data, progress, overlays, files, and charts.
Image behavior is described in its own chapter.

## Main API

- `Chart` supports series, axes, legends, chart types, line styles, and fill
  styles.
- `Table` presents direct rows and columns.
- `DataTable` adds sorting, filtering, pagination, selection, and virtual
  scrolling.
- `Tree` displays expandable hierarchical nodes.
- `FileExplorer` reads a capability-scoped filesystem root and supports view,
  selection, and file operations.
- `ProgressBar` displays bounded progress.
- `Modal` and `Popover` place temporary content above normal content.
- Common types include `DisplaySize`, `Alignment`, `Border`, and `ScrollState`.

## Basic use

Use the direct widget types for explicit props and state. Use the matching
builders for chained construction. Mount overlays inside the same application
tree so event routing and focus state match the presented geometry.

## Behavior

Live display components translate props and retained state into elements and
paint data. Tables and trees limit visible work to the current viewport. Charts
map samples into a cell canvas. File explorer sends filesystem operations to an
owned worker and publishes results back to the application.

Modal and popover widgets maintain their own open state, placement, focus, and
event handling. Progress widgets can animate between values.

## Limits

- Chart resolution is limited by terminal cell geometry.
- Virtual scrolling still requires stable row or node identity.
- File explorer access is bounded by its capability-scoped root.
- Overlay placement is constrained to the presented terminal rectangle.
- Data callbacks should not block the application event loop.

## Source map

- Display exports: [`src/widgets/display/mod.rs`](../src/widgets/display/mod.rs)
- Data table: [`src/widgets/display/data_table.rs`](../src/widgets/display/data_table.rs)
- File explorer: [`src/widgets/display/file_explorer.rs`](../src/widgets/display/file_explorer.rs)
- Display API probe: [`tests/api_widget_behavior/display_probe.rs`](../tests/api_widget_behavior/display_probe.rs)
- Data table tests: [`tests/api_widget_behavior/data_table.rs`](../tests/api_widget_behavior/data_table.rs)
- Tree tests: [`tests/api_widget_behavior/tree.rs`](../tests/api_widget_behavior/tree.rs)

## Related chapters

- [Images and clipboard](images-and-clipboard.md)
- [Layout widgets](layout-widgets.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
