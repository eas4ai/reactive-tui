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

[CHT-010] The framework MUST provide a public plot layer with linear, band, point and ordinal scales, tick generation with a tick margin, axis and grid placement, label measurement and truncation, a legend and a tooltip, and every chart type MUST map data to cells only through it.
Falsifier: Two chart types place the same value on the same axis at different cells, or a chart type's renderer computes a scale or tick position without calling the plot layer.
Mechanism: plot-layer
Rationale: This is the gpui-kit plot module; the chart types are thin over it.
Status: Agreed 2026-09-21

[CHT-025] Every chart type MUST rasterize through one shared mask canvas: each shape (bar rectangle, line stroke, area polygon, pie sector, radar polygon, sankey ribbon, marker) is written as a per-dot membership mask at two by four dots per cell with a color token, and the canvas alone resolves each cell to a glyph (full block for a fully covered cell, an eighth block when coverage is a clean vertical or horizontal fraction, braille otherwise) and a color.
Falsifier: A chart type writes glyphs or colors to cells directly instead of masks, two chart types resolve the same coverage pattern to different glyphs, or a fully covered cell renders as braille.
Mechanism: charts-goldens
Rationale: The developer prefers one rasterization idea for all charts; the pie mask generalizes to every shape.
Status: Agreed 2026-09-21

[CHT-011] The band scale MUST support inner and outer padding, the linear scale MUST include zero when the data does not cross it, and negative values MUST draw from a zero baseline.
Falsifier: Bars touch with padding set, a positive-only series starts its axis above zero, or a negative bar grows upward from the axis bottom.
Mechanism: plot-layer
Status: Agreed 2026-09-21

[CHT-012] Line and area charts MUST draw strokes and polygons into the shared mask canvas with Natural, Linear and StepAfter curve styles and optional point dots; area charts MUST support overlaid and stacked series and a vertical fill gradient.
Falsifier: A diagonal line renders as detached dashes, a curve style has no visible effect, or a stacked area's upper series does not start at the lower series' top.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[CHT-013] Bar charts MUST support vertical and horizontal orientation, grouped and stacked series, growth from Bottom, Top, Left or Right, per-bar color, value labels on bars, and MUST draw bars as rectangles into the shared mask canvas so a bar tip resolves to an eighth block.
Falsifier: A bar of value 3.5 out of 8 renders the same as 3 or 4, a Top-aligned bar grows upward, or a stacked bar's segments overlap.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[CHT-014] A candlestick chart MUST draw wick and body from open, high, low and close accessors, colored by the theme's bullish and bearish colors.
Falsifier: A candle whose close is below its open uses the bullish color, or its wick does not span low to high.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[CHT-015] Pie and donut charts MUST draw each slice into the shared mask canvas by testing each dot's polar coordinates against the slice boundaries, MUST expose inner and outer radius, pad angle, per-slice color and side labels with leader lines, and MUST correct for the 2:1 cell aspect ratio.
Falsifier: A dot inside a slice's angular range and radius is left unpainted or painted with a neighbor's color, a slice label overlaps another, or a full circle renders taller than it is wide in cell aspect.
Mechanism: charts-goldens
Status: Draft

[CHT-016] Radar and sankey charts MUST draw polygons and ribbons into the shared mask canvas, radar with configurable grid levels and multi-series polygons, sankey with the node alignment, iteration and value-scale options of the reference layout.
Falsifier: A radar polygon vertex lands off its axis spoke, or a sankey link's width is not proportional to its value under the chosen scale.
Mechanism: charts-goldens
Status: Draft

[CHT-017] Every chart color, whether a series stroke, fill, bar, slice, axis, grid or tooltip swatch, MUST accept the same tokens the layout utility classes accept (palette names such as `blue-500`, theme variables such as `primary`, hex, and the `/opacity` modifier) and MUST resolve them through the layout's single color resolver and the active Theme; default series colors MUST come from theme variables `--color-chart-1` to `--color-chart-5` plus `--color-chart-bullish` and `--color-chart-bearish`, with no hex literal in the chart code.
Falsifier: A color token that works in a `bg-` utility class is rejected or resolves differently in a chart, a chart renders a color not derivable from the active Theme, or a hex color literal appears under src/widgets/display/charts.
Mechanism: charts-palette
Rationale: The chart canvas already calls `parse_color_token` for per-point colors (canvas.rs:178); this makes that the only path and adds the chart variables to the Theme presets.
Status: Agreed 2026-09-21

[CHT-018] The tooltip MUST be a box near the pointer or selection with one swatch, name and value row per series at the hovered index, MUST flip inside the chart edge, and a crosshair or highlight band MUST mark the hovered index; keyboard navigation and the aria-live announcement MUST be retained.
Falsifier: A tooltip clips at the chart edge, a series is missing from its rows, or the hovered index has no crosshair or band.
Mechanism: charts-interaction
Status: Agreed 2026-09-21

[CHT-019] Mouse movement MUST select the nearest data index, and a chart MUST redraw only when the selection or data changes.
Falsifier: Moving the mouse within one band changes the selection, or a still mouse causes redraws.
Mechanism: charts-interaction
Status: Agreed 2026-09-21

[CHT-020] The chart builder MUST mirror the reference's method names for each supported type (`x`, `y`, `band`, `value`, `stroke`, `fill`, `natural`, `linear`, `step_after`, `dot`, `tick_margin`, `alignment`, `label`, `grid`, `inner_radius`, `outer_radius`, `pad_angle`, `open`, `high`, `low`, `close`, `max_value`, `grid_levels`) over `Vec<T>` with accessor closures, alongside the existing series API.
Falsifier: A listed method is absent for a chart type that supports the feature, or accepts a different argument shape than the reference without an alias.
Mechanism: charts-builders
Status: Agreed 2026-09-21

[CHT-021] Chart rasterization MUST run on a worker that publishes a cell snapshot through the AppWaker, and a chart with no explicit size MUST fill its allotted rectangle and re-rasterize on resize.
Falsifier: Rasterization runs on the main thread, a default-sized chart leaves its parent rectangle unpainted, or a resize leaves the previous snapshot on screen.
Mechanism: frame-budget
Status: Agreed 2026-09-21

[CHT-022] Data changes MUST animate from the previous values to the new ones over the configured duration at the App frame rate, ending exactly at the target, and the reveal MUST be retained; `reduced-motion` MUST skip both.
Falsifier: A value transition overshoots or stops short of its target, or `reduced-motion` still animates.
Mechanism: charts-motion
Status: Agreed 2026-09-21

[CHT-023] Every chart type MUST have a golden at each size class (mini 20 by 5, medium 80 by 24, large 600 by 160) on the debug backend, a page in the widget catalog showing all three, and a section in the manual.
Falsifier: A chart type lacks a golden at any of the three sizes, the catalog page omits a size class, or the manual section is missing.
Mechanism: charts-goldens
Status: Agreed 2026-09-21

[CHT-024] Every chart MUST choose one of three size classes from the rectangle it is allotted, mini (no axes or legend, values only, usable inline down to 8 by 2 cells), medium (axes, ticks, legend, tooltip) and large (grid, full labels, multi-row legend, value labels), MUST switch class when the rectangle crosses a class boundary on resize, and the builder MUST allow forcing a class.
Falsifier: A chart in a mini rectangle draws an axis or legend, a chart in a large rectangle keeps medium label density, a resize across a boundary leaves the previous class on screen, or a forced class is ignored.
Mechanism: charts-goldens
Rationale: The developer requires charts that work as sparklines, panels and full dashboards from one builder call.
Status: Agreed 2026-09-21
