# Render charts in a retained measured cell canvas with owned animation deadlines

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Chart modes remain descriptive placeholders, axis limits or point colors are ignored, layout exceeds measured bounds, tooltips hit stale cells, or removal leaves animation timers alive.
History: The widget ownership decision preserves public unit components through retained children. Existing chart APIs describe seven chart modes, axes, legend placement, tooltips and animation; their old renderer only implements some of these.

## Decision

Preserve Chart, ChartProps, ChartState and both builder routes. Use a retained App child with a bounded terminal-cell canvas for all seven chart modes, measured bounds for tooltips and a component-owned scheduler timeout for finite reveal animations. Numeric X coordinates are data-point indices because the public DataPoint carries only one numeric value. Draw pie and donut sectors with terminal-cell aspect correction; negative values are invalid for these two modes. Use visible data for auto ranges, honor finite explicit axis bounds, and show configuration errors for invalid dimensions/data/ranges. Preserve point and series color precedence, line and fill styles, axis labels/grid/title, legend placement and live prop updates. Verify actual geometry and colors, tooltip metadata, resize, empty/invalid data, animation intermediate frames and clock cleanup before acceptance.

## Realized by

Implementation: `src/widgets/display/charts`.

Behavior checks: `tests/api_widget_behavior/charts.rs`, `tests/api_widget_behavior/orca_display.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
