# Roadmap

Current: charts-plot-layer

Order agreed with the developer on 2026-09-21: charts first on a cell canvas,
then a general graphics canvas over wgpu that replaces the rasterizer
underneath, then the remaining widget families measured against gpui-kit.
The quality bar (BAR) applies to every commitment.

## charts-plot-layer

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, CHT-010, CHT-025, CHT-011, CHT-012, CHT-013, CHT-014, CHT-017, CHT-018, CHT-019, CHT-020, CHT-021, CHT-022, CHT-023, CHT-024, CHT-026, CHT-027, CHT-028

Deliver the shared plot layer and the cartesian charts on it: line, area,
scatter, bar (vertical and horizontal) and candlestick, with braille and eighth-block rasterization on a
worker, theme colors, the swatch-row tooltip with crosshair, mouse and
keyboard selection, value transitions, fill-parent sizing with mini, medium
and large size classes, goldens at three sizes, a catalog page and a manual section per type.

Done when every named requirement passes, each mechanism has recorded a
failing violating example before its passing receipt, the five workspace
gates pass, and the review finds no chart type mapping data outside the
plot layer.

## charts-radial-and-flow

Requirements: CHT-015, CHT-016 plus the quality bar

Pie, donut, radar and sankey on the same plot layer and braille canvas.

## graphics-canvas

Developer ruling 2026-09-21: stay on wgpu; take lessons from rust_pixel
(tile-first cell buffer, charts kept in characters) and beamterm (whole grid
as one instanced draw through a glyph atlas, sub-millisecond at 45,000
cells, overlay compositing) without adopting their GL stacks. The
commitment replaces the cube demo with a wgpu cell-grid renderer: a
CellFrame uploaded as instances against a glyph atlas in one draw, an
offscreen target sized to the widget, a worker, a 2D drawing layer for
paths, fills and gradients that rasterizes into the same target, and output
by host capability (half blocks everywhere, Kitty or Sixel pixels where the
host supports them). The keystone's "not a windowing system" stands; window
presentation of the same frame is a possible later commitment, not this
one. Requirements to be drafted when this commitment is next.

## widget-parity

Remaining widget families measured against gpui-kit 0.6.6 (docs/recon.md
section 13). Requirements to be drafted when this commitment is next.
