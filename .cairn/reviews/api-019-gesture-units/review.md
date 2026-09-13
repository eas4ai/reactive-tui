# Gesture coordinate contract

Observed source: src/hooks/mouse.rs documents drag_threshold in pixels and
GestureState velocity in pixels per second. DragState::distance instead handles
only Position::Cell, returning zero for Position::Pixel. The compiled public
consumer in this directory reports CELL_DISTANCE=5 PIXEL_DISTANCE=0 for the
same 3-4 displacement and fails its required five-pixel distance assertion.
This is defect-confirming evidence, not an acceptance pass.

CrosstermBackend::map_ct_event constructs Position::cell from column/row.
DirectTtyBackend::map_terminal_event preserves pixel positions when supplied,
otherwise cell positions. Thus simply wiring the currently inert gesture hooks
cannot fulfill an unconditional pixel threshold on the ordinary cell route.
A cell displacement does not establish physical pixel movement without host
cell metrics. No pixels-per-cell value is supplied by these mapping functions.

Recommendation: preserve Position::Cell and Position::Pixel and use the unit
carried by each event for gesture distances and velocity. The default threshold
5 means five cells on cell input and five pixels on pixel input. Do not subtract
mixed units; restart a gesture on a coordinate-unit change. Keep the public
function signatures and option fields. This explicitly changes the pixel-only
documentation and requires developer approval under the specification.

Alternative: retain unconditional pixel thresholds and add reliable measured
host cell metrics before translating cell input, with explicit unavailable-metric
behavior. This requires additional host capability and resize verification;
a guessed font size must not be used as evidence of physical pixel distances.

No gesture production code changed. Once the unit contract is decided, retain
real App event routing, owner cleanup, drop-zone/handle matching, click timing,
long-press timers, swipe and wheel acceptance with positive and violating cases.
