# Charts

Prefix: CHT

Chart widgets modeled on gpui-kit 0.6.6 (crates/component/src/chart and
plot), as far as a cell grid allows. The reference's value is its shared
plot layer: scales, ticks, axes, grid, labels, legend and tooltip that every
chart type draws through. Reactive TUI keeps what it already has that the
reference lacks: reveal animation, keyboard point navigation and grouped
bars.

Observed blocks describe the code today (docs/recon.md section 13) and are
not contract. Draft blocks are the proposed work.

## Observed

[CHT-001] The Chart widget draws a `Vec<DataSeries>` of `DataPoint` values as one of BarVertical, BarHorizontal, Line, Area, Pie, Donut or Scatter through `ChartsBuilder`, `builder::widgets::chart` and the `chart!` macro.
Falsifier: A chart type listed in `ChartType` cannot be built through all three builder routes.
Mechanism: charts-builders
Status: Observed

[CHT-002] The chart paints a private cell canvas as absolutely positioned text runs using full-block glyphs for bars and pies, the horizontal box-drawing glyph stepped cell by cell for lines, middle dots for grid and shade blocks for area fills, with a hard-coded hex palette and no theme lookup.
Falsifier: A rendered chart contains a glyph other than those, or a color that came from the Theme.
Mechanism: charts-goldens
Status: Observed

[CHT-003] The chart shows a full-width tooltip band for the hovered or keyboard-selected point, moves the selection with Left, Right, Home and End, clears it with Escape, and announces the selection through an aria-live region.
Falsifier: A keyboard selection leaves the tooltip band empty or the aria-live text unchanged.
Mechanism: charts-interaction
Status: Observed

[CHT-004] The chart reveals its data from zero to full over `animation_duration` on a 16 ms timer and skips the reveal when the `reduced-motion` class is present.
Falsifier: A chart with `reduced-motion` renders a partial reveal, or a chart without it renders full values on its first frame.
Mechanism: charts-motion
Status: Observed

[CHT-005] The chart is sized by its `width` and `height` props, default 80 by 20, and shrinks to its parent but never grows past the props size.
Falsifier: A chart in a 200 by 50 parent with default props paints outside 80 by 20.
Mechanism: charts-goldens
Status: Observed

## Draft

[CHT-010] The framework MUST provide a plot layer under `src/widgets/display/charts/plot`, re-exported from `reactive_tui::widgets::display::plot`, with linear, band, point and ordinal scales, tick generation with a label-skip count, axis and grid placement, label measurement and truncation, a legend and a tooltip, and every chart renderer MUST obtain cell positions only by calling these scales.
Falsifier: Two chart types place the same value on the same axis at different cells, or a renderer file outside the plot layer divides by an axis span or otherwise maps a data value to a cell position itself.
Mechanism: plot-layer
Rationale: This is the gpui-kit plot module; the chart types are thin over it.
Status: Agreed 2026-09-22

[CHT-025] Every chart type delivered in a commitment MUST rasterize through one shared mask canvas, which alone resolves each cell to a glyph and its colors. Strokes, markers, bar rectangles and area polygons are written as per-dot membership at two by four dots per cell with a color token, and axis-aligned rectangles also carry their exact edge fractions; such a cell resolves to a full block when every dot is covered, to an eighth block when one rectangle edge crosses it at a multiple of one eighth, and to braille otherwise. Filled shapes (pie and donut sectors, radar fills, sankey ribbons) are sampled at the pixel grid of the blitter that BLT-002 chooses, the same choice image fallback makes with its application and environment overrides; a cell that holds fill and no stroke or marker MUST resolve to that blitter's glyph with the BLT-001 minimum-error two-color split of its pixels, an uncovered pixel counting as transparent; when CHT-028's ASCII mode applies, it resolves fill cells too. Text (axis labels, legend, tooltip, value labels, slice labels and leader lines) is a separate text layer drawn over the canvas.
Falsifier: A chart type writes shape glyphs or colors to cells directly instead of through the canvas, a fully covered stroke or bar cell renders as braille, a rectangle edge at 3/8 renders as a quarter, a fill-only cell's glyph or colors differ from the BLT-001 split for the chosen blitter, or a fill cell holding exactly two slice colors and no uncovered pixel shows only one of them.
Mechanism: charts-goldens
Rationale: Revised 2026-09-25: filled shapes use the renderer's blitters, which the developer ruled land before the radial charts (roadmap, 2026-09-22); lines, bars and area fills keep braille and eighths.
Status: Agreed 2026-09-25

[CHT-011] The band scale MUST support inner and outer padding, the linear scale MUST include zero when the data does not cross it, and negative values MUST draw from a zero baseline.
Falsifier: Bars touch with padding set, a positive-only series starts its axis above zero, or a negative bar grows upward from the axis bottom.
Mechanism: plot-layer
Status: Agreed 2026-09-21

[CHT-012] Line, area and scatter charts MUST draw strokes, polygons and markers into the shared mask canvas with Natural, Linear and StepAfter curve styles and optional point dots; area charts MUST support overlaid and stacked series and a vertical fill gradient rendered as a shade ramp.
Falsifier: A diagonal line renders as detached dashes, a curve style has no visible effect at the large size class, a stacked area's upper series does not start at the lower series' top, or a scatter marker lands off its data cell.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[CHT-013] Bar charts MUST support vertical and horizontal orientation, grouped and stacked series, growth from Bottom, Top, Left or Right, per-bar color, value labels drawn in the text layer at the large size class, and MUST draw bars as rectangles with exact edge fractions into the shared mask canvas so a bar tip resolves to an eighth block.
Falsifier: A bar of value 3.5 out of 8 renders the same as 3 or 4, a Top-aligned bar grows upward, a stacked bar's segments overlap, or a large-class bar has no value label.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[CHT-014] A candlestick chart MUST draw wick and body from open, high, low and close accessors, colored by the theme's bullish and bearish colors.
Falsifier: A candle whose close is below its open uses the bullish color, or its wick does not span low to high.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[CHT-015] Pie and donut charts MUST draw each slice as a filled shape (CHT-025) whose pixels are the samples whose polar coordinates, corrected for the 2:1 cell aspect ratio, fall inside the slice's angle range and between its inner and outer radius; MUST expose inner radius (zero for a pie), outer radius, pad angle and per-slice color; and at the medium and large size classes MUST place each slice's label beside the chart, joined to its slice by a leader line in the text layer, omitting a label that cannot be placed without overlapping another while keeping its slice in the legend.
Falsifier: A sample inside a slice's angle range and radius is left unpainted or painted with another slice's color, a nonzero pad angle leaves no unpainted gap between adjacent slices, two slice labels overlap, a placed label has no leader line to its slice, or a full pie's painted width in columns is not within one column of twice its painted height in rows.
Mechanism: charts-goldens
Status: Agreed 2026-09-25

[CHT-016] Radar charts MUST place one spoke per category at equal angles from the top, clockwise, and draw each series as a polygon whose vertex on each spoke lies at the category's value under a linear radial scale from the plot layer, starting at zero and ending at the maximum value or `max_value` when set; the outline is a stroke and the optional fill a filled shape (CHT-025); grid levels are drawn as concentric polygons at equal steps, their number configurable; series are drawn in series order, a later series over an earlier one.
Falsifier: A polygon vertex lands off its spoke or at a distance not proportional to its value, the grid shows a number of levels other than the configured one, or an earlier series is drawn over a later one where they overlap.
Mechanism: charts-goldens
Status: Agreed 2026-09-25

[CHT-030] Sankey charts MUST lay out nodes in columns and links between them with the node alignment (left, right, center, justify), iteration count and value scale (linear, square root) options of the reference layout, with node width and label gap in columns and node padding and minimum link width in rows; MUST draw each node as a filled rectangle (CHT-025) whose height is the larger of its incoming and outgoing totals under the value scale, and each link as a filled ribbon (CHT-025) whose width at each end is the link's value under that scale, stacked at each node without overlap and shaded from its source node's color to its target node's color, each blended toward the chart background by the link opacity; at the medium and large size classes MUST label each node beside it (a first-layer node on its left, a last-layer node on its right, any other centered above it), cut to fit, with its throughput at the large class; and a link that names a missing node, or links that form a cycle, MUST render an explicit error message in the text layer instead of shapes.
Falsifier: A link's width at either end or a node's height differs from its value under the chosen scale by more than one fill pixel, a node's height is not the larger of its incoming and outgoing totals, a node alignment or value scale option leaves the layout unchanged for a graph where it applies, two links overlap at a node, a ribbon's end color is not its node's color blended by the link opacity, a medium or large chart places no label for a node that has room, or a missing node or a cycle paints shapes.
Mechanism: charts-goldens
Status: Agreed 2026-09-25

[CHT-031] Pie, donut and radar charts MUST select, on mouse movement, the slice under the pointer (pie and donut) or the category whose spoke is nearest the pointer's angle (radar), selecting nothing outside the outer radius; Left, Right, Home and End MUST move the selection in slice or category order; the selection MUST show the CHT-018 tooltip and update the aria-live announcement, and CHT-019's same-index rule applies.
Falsifier: A pointer inside a slice selects another slice or none, a radar pointer selects a category other than the nearest spoke's, a pointer outside the outer radius selects something, Left or Right does not move to the adjacent slice or category, or a selection leaves the tooltip or the aria-live text unchanged.
Mechanism: charts-interaction
Rationale: CHT-019 selects by the x axis, which a radial chart does not have.
Status: Agreed 2026-09-25

[CHT-032] Sankey charts MUST select, on mouse movement, the node under the pointer, selecting nothing over a ribbon or an empty cell; Left, Right, Home and End MUST move the selection through the nodes by layer and, within a layer, from top to bottom; the selected node's links MUST keep the link opacity while every other link fades further, which marks the selection outside the tooltip in place of CHT-018's crosshair; and the selection MUST show the CHT-018 tooltip beside the node with the node's label and throughput and update the aria-live announcement, and CHT-019's same-index rule applies.
Falsifier: A pointer on a node selects another node or none, a pointer on a ribbon or an empty cell selects a node, Left or Right does not move to the adjacent node in that order, a selection leaves every link unchanged or fades one of the selected node's own links, or the tooltip or the aria-live text omits the node's label or throughput.
Mechanism: charts-interaction
Rationale: CHT-019 selects by the x axis and CHT-031 by angle; a Sankey selects nodes, as the reference's hover does.
Status: Agreed 2026-09-25

[CHT-029] The pie, donut, radar and sankey builders MUST mirror the reference's method names for each delivered type over `Vec<T>` with accessor closures evaluated once at `build()` into data points, so chart props stay comparable: pie and donut `value`, `label`, `color`, `inner_radius`, `outer_radius`, `pad_angle` and `label_gap`; radar `value`, `label`, `stroke`, `fill`, `dot`, `grid`, `grid_levels`, `max_value` and `outer_radius`; sankey `new(nodes, links)` over links built with `SankeyLink::new(source, target, value)`, `value_scale`, `node_align`, `iterations`, `node_width`, `node_padding`, `node_color`, `node_label`, `value_label`, `labels`, `link_opacity`, `min_link_width` and `label_gap`; arguments take the terminal equivalents of the reference types.
Falsifier: A listed method is absent for a delivered type that supports the feature, or a builder stores a closure in the props instead of the evaluated points.
Mechanism: charts-builders
Rationale: CHT-020 names the cartesian methods only; the radial and flow types take the reference's own names.
Status: Agreed 2026-09-25

[CHT-017] Every chart color, whether a series stroke, fill, bar, slice, axis, grid or tooltip swatch, MUST accept the tokens the layout utility classes accept (palette names such as `blue-500`, theme variables such as `primary`, and hex) and MUST resolve them through one resolver, `Theme::resolve_color`, that the layout's utility classes also use; the App holds the active Theme and components read it through the hook scope; default series colors MUST come from theme variables `--color-chart-1` to `--color-chart-5` plus `--color-chart-bullish` and `--color-chart-bearish`, defined in every preset with a color-blind-safe default set, with no color literal in the chart code.
Falsifier: A token that resolves in a `bg-` utility class is rejected or resolves differently in a chart, a preset lacks a chart variable, a chart renders a color not derivable from the active Theme, or a color literal appears under src/widgets/display/charts.
Mechanism: charts-palette
Rationale: The chart canvas already calls `parse_color_token` for per-point colors (canvas.rs:178), but that function knows no theme; the resolver named here is the one path.
Status: Agreed 2026-09-22

[CHT-018] The tooltip MUST be a box placed adjacent to the hovered or selected index (right of it, or left when the box would cross the chart's right edge, above when it would cross the bottom) with one swatch, name and value row per series at that index, at most eight rows before it summarizes, and a crosshair column or highlight band MUST mark the hovered index outside the box; keyboard navigation and the aria-live announcement MUST be retained, and the accessibility description MUST carry the series names and hovered values at every size class.
Falsifier: A tooltip crosses the chart edge, a series is missing from its rows below eight, the hovered index has no crosshair or band outside the box, or a mini chart's accessibility description omits the hovered value.
Mechanism: charts-interaction
Status: Agreed 2026-09-22

[CHT-019] Mouse movement MUST select the data index nearest on the x axis (nearest in both axes for scatter), and a mouse move that does not change the selected index MUST leave the frame unchanged outside a running transition or resize.
Falsifier: Two positions in one band select different indices, an empty cell beside a point selects nothing, or a same-index move changes the frame while nothing animates.
Mechanism: charts-interaction
Status: Agreed 2026-09-22

[CHT-020] The chart builder MUST mirror the reference's method names for each chart type delivered (`x`, `y`, `band`, `value`, `stroke`, `fill`, `natural`, `linear`, `step_after`, `dot`, `tick_margin`, `alignment`, `label`, `grid`, and for candlestick `open`, `high`, `low`, `close`) over `Vec<T>` with accessor closures that are evaluated once at `build()` into data points, so chart props stay comparable, alongside the existing series API; arguments take the terminal equivalents of the reference types.
Falsifier: A listed method is absent for a delivered chart type that supports the feature, or a builder stores a closure in the props instead of the evaluated points.
Mechanism: charts-builders
Status: Agreed 2026-09-22

[CHT-021] Chart rasterization MUST run on a named worker thread (`rtui-chart-*`) that writes finished cells into a snapshot slot the chart owns and requests a redraw through the AppWaker; the main thread only copies the latest snapshot into the frame. A chart whose width or height is unset MUST fill its allotted rectangle (the props' size fields become optional, unset meaning fill) and MUST re-rasterize on resize, showing the previous snapshot clipped for at most one frame.
Falsifier: Shape rasterization runs on the main thread, a chart with unset size leaves its parent rectangle unpainted, or a resize leaves the previous snapshot on screen for two frames.
Mechanism: frame-budget
Status: Agreed 2026-09-22

[CHT-022] A data change MUST animate from the previous values to the new ones over the configured duration (default 200 ms) at the App frame rate, never dropping below the previous rendering, ending exactly at the target; the reveal MUST be retained; `reduced-motion` MUST skip both.
Falsifier: A value transition dips below the previous rendering, overshoots, or stops short of its target, or `reduced-motion` still animates.
Mechanism: charts-motion
Status: Agreed 2026-09-22

[CHT-023] Every chart type delivered in the commitment MUST have a golden of the text grid plus a color digest at each size class (mini 20 by 5, medium 80 by 24, large 600 by 160) on the debug backend, a page in the widget catalog showing all three, and a section under its own heading in the manual.
Falsifier: A delivered chart type lacks a golden at any of the three sizes, the catalog page omits a size class, or the manual heading is missing.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[CHT-024] Every chart MUST choose a size class from its allotted rectangle: mini when width is under 40 columns or height under 8 rows (no axes or legend, shapes only, usable down to 8 by 2), large when width is at least 200 and height at least 40 (grid, full labels, multi-row legend, value labels), medium otherwise (axes, ticks, single-row legend, tooltip); MUST switch class when a resize crosses a boundary; and the builder MUST allow forcing a class, which then applies at any rectangle.
Falsifier: A 39 by 10 chart draws an axis, a 200 by 40 chart has no grid, a resize from 80 by 24 to 200 by 40 keeps the medium layout, or a forced class is ignored.
Mechanism: charts-goldens
Rationale: The developer requires charts that work as sparklines, panels and full dashboards from one builder call.
Status: Agreed 2026-09-22

[CHT-026] A chart with no series, an empty series, or a NaN or infinite value MUST render an explicit empty or error message in the text layer instead of shapes, and MUST never panic.
Falsifier: An empty or NaN input paints shapes, paints nothing, or panics.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[CHT-027] When a series has more points than the plot area has columns, the plot layer MUST decimate to at most two points per column by keeping each column's minimum and maximum, and the tooltip MUST still report the original index.
Falsifier: A 10,000-point series renders slower than a 1,000-point one by more than a factor of two, or the tooltip on a decimated chart reports a column index instead of a data index.
Mechanism: charts-goldens
Status: Agreed 2026-09-22

[CHT-028] When the terminal capability report says braille or block glyphs are unavailable, or the builder forces ASCII, the mask canvas MUST resolve cells with ASCII (`#`, `|`, `-`, `.`) instead, with the same geometry.
Falsifier: With glyph support reported absent, a chart emits a braille or block character.
Mechanism: charts-goldens
Status: Agreed 2026-09-22
