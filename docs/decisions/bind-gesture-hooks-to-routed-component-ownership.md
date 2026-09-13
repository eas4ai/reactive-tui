# Bind gesture hooks to routed component ownership

Level: Judged
Decided by: Codex
Rests on: API-019 docs/residual-api-inventory.md
Would be wrong if: A mounted hook does not receive routed mouse events, options use an identifier callers cannot supply, one component changes another component state, or unmount leaves live registrations.
History: Earlier residual API probes found gesture hooks returning unconnected default signals and ignoring all options, so compiling hook calls do not establish behavior.

## Decision

Give each App-owned ComponentRuntime one MouseEventProcessor. During a mounted component render, provide a private hook routing context whose public-facing identifier is the component invocation key, falling back to its registered component name. Mouse hooks register their retained signals in that context. Before ordinary event propagation, App resolves the painted target once, maps it to the innermost mounted component, and supplies the global pointer position plus that component's local position to the processor. Component removal unregisters every owned signal. Interpret drag_handle_selector and drop_zones as exact routed component identifiers. Apply drag thresholds before activation, long-press thresholds on release, drop-zone membership while dragging, and allow_drag_outside when computing can_drop. Standalone Hooks without an App component routing context retain inert state and make no ownership claim.

## Realized by

(none yet: recorded, not built)
