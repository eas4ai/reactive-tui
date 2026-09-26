# Display widgets

Crate modules: `widgets`

Widget module: `display`

## Purpose

Display widgets present structured data, progress, overlays, files, and charts.
Image behavior is described in its own chapter.

## Main API

- `Chart` draws line, area, scatter, bar, candlestick, pie and donut charts
  from series, with axes, legends, tooltips, curve and fill styles, at three
  size classes.
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

## Charts

Charts draw through one plot layer (`reactive_tui::widgets::display::plot`:
scales, ticks, axes, grid, legend, tooltip, curve interpolation and min/max
decimation) and one mask canvas at two by four dots per cell. The canvas
resolves each cell to a full block, an eighth block where a rectangle edge
crosses the cell, a marker, or braille, and to `#`, `|`, `-` and `.` when the
builder forces ASCII with `.ascii(true)` or the terminal capability report
says braille and block glyphs are unavailable (the backend passes its query
result to `charts::report_glyph_support`). Filled shapes (pie and donut
slices, radar fills) are sampled at the pixel grid of the blitter image
fallback uses, chosen by the same rule and overrides (`set_image_blitter`,
the `REACTIVE_TUI_BLITTER` variable; see images-and-clipboard.md). A cell
that holds only fill takes that blitter's glyph with the two colors that
best split its samples, so where two slices meet one cell shows both; lines,
outlines and bar tips keep braille and eighth blocks. Rasterization runs on a named
`rtui-chart-*` worker thread; the main thread copies the latest snapshot into
the frame. When a chart first appears or changes size, the main thread waits
at most 4 ms for the picture at that size. If the picture is not ready, that
frame shows the chart area empty, the chart's accessibility node is marked
busy, and the worker's finish redraws the chart. A picture drawn for another
size is never painted. A chart whose width or height is unset fills its
rectangle.

Colors are tokens: a palette name such as `blue-500`, a theme variable such
as `primary` or `chart-1`, or hex. Every token resolves through
`Theme::resolve_color`, the resolver the utility classes use. Each preset
defines `--color-chart-1` to `--color-chart-5`, `--color-chart-bullish` and
`--color-chart-bearish`.

The typed builders take a `Vec<T>` and accessor closures that run once at
`.build()`, so the resulting props hold plain points and stay comparable.
`ChartsBuilder` and `builder::chart()` remain for series built by hand.

### Line chart

`LineChartBuilder::new(rows).x(|r| r.day).y(|r| r.close).name("close")`
draws one series per `.y(` call. `.stroke("chart-2")` colors the series
added last. `.natural()`, `.linear()` and `.step_after()` choose the curve;
`.dot()` marks every point. Series longer than the plot is wide are
decimated to each column's minimum and maximum, and the tooltip still
reports the original index.

### Area chart

`AreaChartBuilder` adds `.fill(` for the series added last and
`.stacked(true)` to stack series. `FillStyle::Gradient` on a series shades
the fill toward the baseline; `FillStyle::Pattern` tiles glyphs over it.

### Scatter chart

`ScatterChartBuilder` places a `•` marker on every point. The pointer selects
the point nearest in both axes.

### Bar chart

`BarChartBuilder::new(rows).band(|r| r.day).value(|r| r.total)` draws
vertical bars; `.alignment(BarGrowth::Left)` or `Right` turns them
horizontal, `Top` hangs them from the top. Several `.value(` calls group
bars; `.stacked(true)` stacks them. A bar tip resolves to an eighth block,
so 3.5 of 8 differs from 3 and 4. `.label(|r| ..)` and the large size class
show value labels in the text layer.

### Candlestick chart

`CandlestickChartBuilder::new(rows).x(..).open(..).high(..).low(..).close(..)`
draws a wick from low to high and a body from open to close. Candles that
close above their open use the theme's bullish color, the rest the bearish
color; `.bullish(` and `.bearish(` override the tokens.

### Pie chart

`PieChartBuilder::new(rows).value(|r| r.share).label(|r| r.name)` draws one
slice per row, clockwise from twelve o'clock, as filled sectors. The circle
spans twice as many columns as rows, because a cell is twice as tall as it
is wide. `.color(|r| ..)` gives each slice a color token; unset slices take
the palette in order. `.outer_radius(0.8)` shrinks the circle to a fraction
of the largest that fits, and `.pad_angle(0.05)` leaves a gap in radians
between slices. At the medium and large size classes each slice's label
sits beside the circle, joined to it by a leader line, `.label_gap(3)`
columns from the edge. A leader ends beside a cell that only its own slice
paints, and never crosses a slice, a label or another leader. A label with
no row where such a leader fits is left out, and the legend still lists its
slice. This happens, for example, to a thin slice at the top or bottom of
the circle whose outer cells it shares with a neighbour. When the legend
has no room for every slice, as a one-row legend at the medium size often
does not, it lists the slices whose labels were left out first, then the
labelled ones that still fit, in slice order. The pointer selects the slice under it and
nothing outside the circle; Left, Right, Home and End step through the
slices.

### Donut chart

`DonutChartBuilder` takes the pie builder's methods and draws a hole half
the radius wide; `.inner_radius(0.6)` sets the hole as a fraction of the
outer radius. The pointer selects nothing in the hole.

### Radar chart

`RadarChartBuilder::new(rows).label(|r| r.skill).value(|r| r.score)` puts one
spoke per row, clockwise from twelve o'clock, and one polygon per `.value(`
call, whose vertex on each spoke lies at the value's distance from the
center, from zero to the largest value or `.max_value(10.0)`. `.stroke(` and
`.fill(` color the series added last, and `.fill("none")` leaves it
unfilled; a later series lies over an earlier one. `.dot()` marks every
vertex. At the medium and large size classes `.grid_levels(4)` concentric
polygons and the spokes are drawn under the shapes, `.grid(false)` hides
them, and each spoke's label sits at its end. The pointer selects the
category of the nearest spoke and nothing outside the outer radius.

### Sankey chart

`SankeyChartBuilder::new(nodes, links)` draws flows between nodes. Each link
is `SankeyLink::new(source, target, value)`, naming its nodes by their index
in `nodes`. Nodes sit in columns: `.node_align(SankeyAlign::Justify)`, the
default, puts every sink in the last column, and `Left`, `Right` and
`Center` follow d3-sankey. A node is as tall as the larger of its incoming
and outgoing totals, and each link is a ribbon as wide as its value at both
ends, stacked at each node without overlap. `.value_scale(SankeyValueScale::Sqrt)`
maps values through their square root so a large flow does not dwarf the
small ones, and `.iterations(6)` sets how many passes move the nodes toward
their flows. `.node_width(2)` is in columns and `.node_padding(1)` in rows;
`.min_link_width(0.25)`, in rows, keeps thin flows visible.

A ribbon is shaded from its source node's color to its target's and blended
toward the chart background by `.link_opacity(0.3)`. Where two ribbons
cross, the later one covers the earlier, since a cell cannot layer
translucent colors; a ribbon that skips a column is drawn under the ribbons
that end at the nodes it passes. `.node_color(|n| ..)` gives each node a
color token; unset nodes take the palette in order.

At the medium and large size classes each node's label sits beside it: a
first-column node's on its left, a last-column node's on its right,
`.label_gap(1)` columns away, and any other node's centered above it. The
label is `.node_label(|n| n.name)`, cut to fit; the large class adds the
throughput, which `.value_label(|n, v| ..)` can format. `.labels(|n, v| ..)`
replaces the label with lines of `SankeyLabel::new(text)`, each with an
optional `.color(`. A link that names a missing node, or links that form a
cycle, show an error message instead of shapes.

The pointer selects the node under it, and nothing over a ribbon or empty
space. Left, Right, Home and End step through the nodes column by column,
top to bottom. The selected node's links keep their opacity while the rest
fade, and the tooltip names the node and its throughput.

### Size classes

Every chart picks a size class from its rectangle, and `.size_class(` forces
one:

- mini, under 40 columns or under 8 rows: shapes only, no axes or legend,
  usable down to 8 by 2 as a sparkline;
- medium: axes, ticks, a single-row legend and the tooltip;
- large, at least 200 by 40: grid, full labels, a multi-row legend and value
  labels.

A resize that crosses a boundary switches class. The tooltip is a box beside
the selected index with a swatch, name and value per series, at most eight
rows before it summarizes, and a crosshair marks the index. Left, Right,
Home and End move the selection; Escape clears it; the selection is announced
through a live region and the accessibility description at every class.

Data changes animate from the shown values to the new ones over
`.transition_duration(` milliseconds (200 by default); the reveal from zero
runs once when data first appears with `.animated(true)`; the
`reduced-motion` class skips both.

## Limits

- Chart resolution is limited by terminal cell geometry: two by four dots per
  cell for shapes, one eighth of a cell for rectangle edges.
- The App lays out one element per colored run per row, so a 700-column
  chart with a grid costs more per frame than one without; the frame budget
  is measured on the optimized build.
- Virtual scrolling still requires stable row or node identity.
- File explorer access is bounded by its capability-scoped root.
- File explorer copy and delete operations keep the inspected entry identity.
  If another process replaces that entry before use, the operation returns an
  error instead of copying or deleting the replacement.
- Overlay placement is constrained to the presented terminal rectangle.
- Data callbacks should not block the application event loop.

## Source map

- Display exports: [`src/widgets/display/mod.rs`](../src/widgets/display/mod.rs)
- Chart props and builders: [`src/widgets/display/charts.rs`](../src/widgets/display/charts.rs)
- Typed chart builders: [`src/widgets/display/charts/typed.rs`](../src/widgets/display/charts/typed.rs)
- Plot layer: [`src/widgets/display/charts/plot/mod.rs`](../src/widgets/display/charts/plot/mod.rs)
- Mask canvas: [`src/widgets/display/charts/mask.rs`](../src/widgets/display/charts/mask.rs)
- Chart goldens: [`tests/charts_goldens.rs`](../tests/charts_goldens.rs)
- Chart contract tests: [`tests/charts_contract.rs`](../tests/charts_contract.rs)
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
