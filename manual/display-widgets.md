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

## Modal

`Modal` places a box over the screen with a veil behind it. Build it with
`modal()` (`.title()`, `.content()`, `.contents()`, `.visible(true)`,
`.closable()`, `.size(width, height)`) or with `ModalProps`. The box is
`bg-surface text-foreground` with a border in `border`, the veil `overlay`,
the title and the footer one cell in, the close button `text-muted`; a
button without a style takes the primary look when its action confirms and
the secondary look otherwise, and the `selection` roles while it holds the
focus (`PRIMARY_BUTTON`, `SECONDARY_BUTTON`, `DANGER_BUTTON`). Set
`modal_style`, `backdrop_style`, `header_style`, `footer_style` and
`close_button_style` to change that.

A box whose width is `ModalSize::Auto` is as wide as its content, its title
or its buttons need plus one cell of padding at each side, and at most half
the viewport; its content wraps at that width. `ModalPosition` centers the
box on the screen or puts it at an edge or a corner, one cell from it;
`offset` moves it by cells from there. The modal is painted whole on the
screen even when the element that owns it stands inside a box that clips
its content. Escape closes a closable modal; Tab moves between its buttons
and Enter presses the focused one; Alt with an arrow moves a draggable
modal one cell and Alt and Shift with an arrow resize a resizable one by
a cell. The screen reader hears the box as a
dialog labeled by its title. `on_placed` is called with the box's position
and size each time they change.

## Popover

`Popover` opens a box beside its trigger. Build it with `popover()`
(`.trigger()`, `.content()`, `.class()`) or with `PopoverProps`. The box is
`bg-surface text-foreground` with a border in `border` and one cell of
padding inside it; the arrow, one row deep and pointing at the trigger, is
a piece of the box, filled in `surface` with its outline in `border`; the
veil of `backdrop_filter` is `overlay`. The box
opens `gap` cells from its trigger (one by default: one row under or over
it, one cell beside it, the arrow in them) at the side `position` names,
moved by `offset`, and at the opposite side when only that side holds it
(`BoundaryBehavior::Flip`, the default). It stacks over a modal or a dialog
and under a toast and a menu panel.

Enter or Space on the trigger opens the popover and Escape closes it. Opened
by a key, the popover moves the focus into its content when the content
holds a focusable element, and back to the trigger when it closes;
`auto_focus` does that for a click too, and `focus_trap` keeps the focus
inside. The trigger tells the screen reader whether the popover is open.
A popover is painted whole even when its trigger stands inside a modal or
a box that clips its content.

## Charts

Charts draw through one plot layer (`reactive_tui::widgets::display::plot`:
scales, ticks, axes, grid, legend, tooltip, curve interpolation and min/max
decimation) and one mask canvas at two by four dots per cell, or a pixel
picture where the terminal takes one (see Plot pictures). The mask canvas
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
defines `--color-chart-1` to `--color-chart-5`, `--color-chart-bullish`,
`--color-chart-bearish` and `--color-chart-grid`. The chart's chrome takes
its colors from roles: axis lines `border`, tick labels, axis titles and
legend names `text-muted`, the title `foreground`, the grid `chart-grid`,
the tooltip `surface` behind `foreground` text in a `border` frame, the
crosshair `text-muted`, the highlight band `hover` and the selected marker
`ring`. In ASCII mode (`.ascii(true)`, or a terminal without the glyphs)
the axes, grid, tooltip frame, crosshair, swatches and markers are ASCII
too.

The typed builders take a `Vec<T>` and accessor closures that run once at
`.build()`, so the resulting props hold plain points and stay comparable.
`ChartsBuilder` and `builder::chart()` build every chart type from series
made by hand, with a method for every option the typed builders have: the
per-point ones (`.tooltip_title(`, `.tooltip_value(`, `.tooltip_value_color(`,
`.tooltip_content(`, `.label_color(`, `.fill_with(` and `.fill_gradient(`)
take their closure over the `DataPoint` instead of the row and apply to the
series added last, so the same chart built either way has the same props.
The `chart!` macro has a form per type (`chart![line: "name" => [1.0, 2.0]]`,
`chart![candlestick: "name" => [(o, h, l, c)]]`,
`chart![sankey: nodes => [(from, to, value)]]`).

### Plot pictures

Where the terminal takes Kitty graphics or Sixel and the crate is built with
`wgpu-graphics`, a line, area, scatter, bar or candlestick chart draws its
plot area as one pixel picture instead of braille (CHT-037): the grid and
reference lines, the strokes, area fills, markers, bars and candles, and the
hover marks, drawn on the canvas's drawing thread (wgpu-graphics.md), one
picture pixel per screen pixel at the terminal's cell size. The chart reads
what the terminal takes and how big a cell is from its layout
(`LayoutInfo::terminal`, a `TerminalInfo`), which the backend fills before
the first frame and after every resize; the chart's `rtui-chart-*` worker
still lays the chart out and builds the picture's scene, and the main
thread copies. Axes, ticks, titles, the legend, value labels, tick labels
placed inside the plot and the tooltip stay cell text, the text inside the
plot painted over the picture. Until a chart's first picture is ready its
plot is blank with the text already drawn; where the terminal takes no
pixels, or the feature is off, the chart draws in cells as before, byte for
byte. Pie, donut, radar and sankey charts draw in cells.

In the picture a stroke is an eighth of a cell high (at least one pixel)
with round joins and caps, drawn as one curve segment per pair of points; a
dashed or dotted line keeps its pattern; a scatter marker is a disc half a
cell high and a line's dot a third; grid, reference and crosshair lines are
one pixel; a gradient area fades from the stroke to transparent at the
baseline; a pattern fill draws its lines, dots or tile glyphs inside the
area; bars have whole-pixel edges, a gradient from base to tip and corners
rounded by `.corner_radius(cells)` (default none). Thinning keeps two
points per pixel column (CHT-027).

The hover is drawn in the picture (CHT-038): on a line or area chart a
crosshair through the hovered index with a dot in a halo on every series
there; on a bar chart a band in the `hover` role over the hovered band,
under the bars, whose center glides from the bar hovered before to the new
one over 150 ms while the other bars fade toward the background, 45 percent
at one band away; on a scatter a ring in the `ring` role around the
selected point; on a candlestick the crosshair through the candle. The
dots, halo and fade ease in over 150 ms when the hover begins. Each change
of the hovered index is a new picture, and one picture per frame while the
band glides or the marks ease in; `reduced-motion` snaps both; a mouse move
that keeps the index sends nothing (CHT-019). The tooltip box stays cell
text over the picture.

Switches: `charts::set_graphics_options(GraphicsOptions)` chooses, for
every chart of the process, the renderer and font the picture's canvas
uses, and `output: Some(CanvasOutput::Blocks)` keeps every plot in cells;
`REACTIVE_TUI_CANVAS=blocks` does the same from the environment and wins
over it, as it does for a canvas. The catalog's `--cpu` draws the plots on
the software renderer.

### Line chart

`LineChartBuilder::new(rows).x(|r| r.day).y(|r| r.close).name("close")`
draws one series per `.y(` call. `.stroke("chart-2")` colors the series
added last. `.natural()`, `.linear()` and `.step_after()` choose the curve
of the series added last, or of the whole chart before any series. Dots are
off until `.dot()` turns them on for the series added last (or for every
series before any is added). Series longer than the plot is wide are
decimated to each column's minimum and maximum, and the tooltip still
reports the original index.

The value axis takes the reference's options: `.y_domain(min, max)` pins it
(shapes outside stop at the plot's edge; equal ends draw nothing),
`.point_count(n)` lays the category axis out for `n` points with the data
taking the leading ones, `.y_axis(false)` hides its labels,
`.y_axis_label_placement(AxisLabelPlacement::Inside)` draws them inside the
plot beside their grid rows, `.y_tick_count(n)` sets how many ticks place
the grid rows and labels (at least two), `.y_tick_format(|v| ..)` formats
their text once at `.build()`, `.x_tick_count(n)` labels `n` categories
spread from the first to the last, `.grid_columns(n)` divides the plot into
`n` columns of vertical grid lines, `.grid_dashed(false)` draws the grid
solid, `.reference_line(v)` draws a dashed line across the plot at `v`, and
`.y_padding(top, bottom)` keeps rows clear past the extreme values.
`.tick_margin(n)` shows every n-th category label on either orientation.

### Area chart

`AreaChartBuilder` adds `.fill(` for the series added last and
`.stacked(true)` to stack series. The fill is its own color, apart from the
stroke: unset, it takes the stroke color, and it shows at the series'
`fill_opacity` (0.4 by default, `DataSeries::with_fill_opacity`) over the
chart background. `FillStyle::Gradient` on a series is strongest at the
stroke and fades to the background at the baseline; `FillStyle::Pattern`
tiles glyphs over it. Each series keeps its own curve and dots.

### Scatter chart

`ScatterChartBuilder::new(rows).x(|r| r.weight).y(|r| r.height)` places a
`•` marker where each point's numeric x and y fall on linear axes, each
series with its own points; `.label(|r| ..)` names a point for its tooltip,
which otherwise shows its x. The pointer selects the point nearest in both
axes among every original point, thinned or not, and the selected point is
ringed. The line chart's axis, tick, grid and reference-line methods apply.

### Bar chart

`BarChartBuilder::new(rows).band(|r| r.day).value(|r| r.total)` draws
vertical bars; `.alignment(BarGrowth::Left)` or `Right` turns them
horizontal, `Top` hangs them from the top. Several `.value(` calls group
bars; `.stacked(true)` stacks them, and the value axis covers the stacked
totals. A bar tip resolves to an eighth block, so 3.5 of 8 differs from 3
and 4; in a plot picture it lands on its nearest whole pixel, and
`.corner_radius(cells)` rounds the bars' corners there. `.label(|r| ..)` and the large size class show value labels, placed
tallest first so none overlaps another or leaves the plot; `.label_color(|r|
..)` colors each; `.fill_with(|r| ..)` colors each bar from its row and
`.fill_gradient(|r, range, to_bar| ..)` shades it from base to tip.
`.padding_inner(0.4)` and `.padding_outer(0.2)` are the band paddings,
`.max_band_width(cells)` caps a band, `.min_length(cells)` keeps a tiny bar
visible, `.band_count(n)` lays the band axis out for `n` bands and
`.band_tick_count(n)` labels `n` of them. `.label_axis(false)` hides the
band axis, `.value_axis(false)` the value labels; `.value_tick_count(`,
`.value_axis_label_placement(` and `.value_tick_format(` are the value
axis's counterparts of the line chart's options, whichever way the bars
run.

### Candlestick chart

`CandlestickChartBuilder::new(rows).x(..).open(..).high(..).low(..).close(..)`
draws a wick from low to high and a body from open to close, the body
`.body_width_ratio(0.8)` of its band and at most `.max_band_width(cells)`
wide. Candles that close above their open use the theme's bullish color,
the rest the bearish color; `.bullish(` and `.bearish(` override the tokens.
Candles reveal and animate like the other types: their open, high, low and
close all move.

### Tooltip, keys and the screen reader

Every cartesian builder takes `.interactive(false)` to turn hover, the keys
and the tooltip off, `.tooltip_title(|r| ..)` for the tooltip's title row in
place of the category, `.tooltip_value(|r, v| ..)` for a point's value text,
`.tooltip_value_color(|r, v| ..)` for its color and `.tooltip_content(|r|
..)` for whole content lines in place of the series rows; `.aria_label(`
names the chart for the screen reader in place of its title. The tooltip is
an opaque box on the `surface` role: a title row with the category, then a
swatch, name and value per series, the followed series first, a candle as
four rows (open, high, low, close), and a crosshair column, a highlight band
in the `hover` role or a ring on a scatter's point marks the selection
outside the box. Left, Right, Home and End move the selection along the
followed series (a numeric scatter's points in x order); Up and Down choose
the series; Escape clears. The screen reader is told the chart's name, its
type and state, the selection as its value, and one child per series.

### Pie chart

`PieChartBuilder::new(rows).value(|r| r.share).label(|r| r.name)` draws one
slice per row, clockwise from twelve o'clock, as filled sectors. The circle
spans twice as many columns as rows, because a cell is twice as tall as it
is wide. `.color(|r| ..)` gives each slice a color token; unset slices take
the palette in order. `.outer_radius(0.8)` shrinks the circle to a fraction
of the largest that fits, and `.pad_angle(0.05)` leaves a gap in radians
between slices. At the medium and large size classes each slice's label
sits beside the circle, joined to it by a leader line, `.label_gap(3)`
columns from the edge. At `.label_gap(0)` a label touches the circle; one
whose leader finds no room there, as on the circle's widest rows, sits one
column further out. A leader ends beside a cell that only its own slice
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
unfilled; a later series lies over an earlier one. Every vertex carries a
dot unless `.dots(false)` turns them off. At the medium and large size classes `.grid_levels(4)` concentric
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
the selected index with a title row, then a swatch, name and value per
series, at most eight rows before it summarizes, and a crosshair, band or
ring marks the index. Left, Right, Home and End move the selection, Up and
Down choose the series; Escape clears it; the selection is announced through
a live region and the accessibility value at every class.

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
