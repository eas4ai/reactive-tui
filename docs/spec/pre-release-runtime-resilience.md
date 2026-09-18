# Pre-release runtime resilience

Status: Agreed 2026-09-14
Prefix: RTR

This specification remediates Fable audit findings M1, M3, M4, M7, and L1.

[RTR-001]
Automatically selected accessibility transport MUST degrade to disabled when
the session bus is stale or unavailable. Explicitly requested accessibility
MUST return its connection error.
Falsifier: An automatic connection failure stops App before its first frame, or
an explicit request silently disables accessibility.
Mechanism: Fake session-bus endpoints exercise automatic and explicit modes and
verify frame delivery and error reporting.

[RTR-002]
ThreadedEventLoop queues MUST provide bounded backpressure without holding the
consumer lock. Stop or Drop MUST cancel and join a blocked input reader.
Falsifier: A full queue livelocks, stop blocks on stdin, or a reader thread
survives Drop.
Mechanism: Saturation and blocked-read tests require consumer progress and
bounded shutdown.

[RTR-003]
TokioEventLoop synchronous entry points MUST NOT call `block_on` from an
active async runtime.
Falsifier: A public start or stop call panics when invoked inside Tokio.
Mechanism: Current-thread and multi-thread Tokio tests exercise every public
sync and async lifecycle route.

[RTR-004]
Adaptive frame-rate and grid APIs MUST validate zero and reversed bounds before
arithmetic.
Falsifier: `min_fps == 0`, `min_fps > max_fps`, or
`auto_grid(columns == 0)` panics or divides by zero.
Mechanism: Boundary and property tests require a documented error or safe empty
layout for every invalid value.
