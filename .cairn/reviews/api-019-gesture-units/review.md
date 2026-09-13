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

## Approved coordinate repair

The developer answered api-019-api-020-3 with ok. The specification now
records event-unit distance and velocity and restarting on unit changes.
DragState measures matching Cell and Pixel pairs using floating-point deltas;
missing or mixed positions return zero. Processor history restarts on unit
changes, and distance/direction calculations do not overflow at u32 limits.
The existing public i32 drag delta saturates; distance retains the full range.

Verification: the two distance acceptance tests first failed on the old code
(distance-tests-before.out), then passed after repair. Two additional processor
probes initially failed in the terminal: mixed-unit drag produced (99, 198)
instead of (0, 0), and swipe history stayed active across a unit change.
All four coordinate tests and all nine existing mouse-hook tests passed after
repair (coordinate-tests-final.out). Workspace formatting and strict Clippy
for the library and coordinate test passed. The initial format check failed
and formatting was corrected; both outputs remain recorded. Dependency warning
about parentheses in vendored Crossterm remains outside this change.

Ripwire edit checks exited zero. Quality delta exited 2 and test gate exited 4;
these are not passes. Their raw reports remain alongside this review.

Self-audit of this slice: signatures are preserved, numeric boundaries and
negative/positive cases are tested, and the approved contract is recorded.
This is not completion of gesture hooks or API-019. Hooks still need App
registration, component isolation, options, timers, routed pixel hit testing
and cleanup verification. Existing processor-wide state is still shared across
manual component registrations and must be repaired before acceptance.
