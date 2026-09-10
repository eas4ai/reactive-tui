# Own a bounded and cancellable screen-reader transport per App

Level: Judged
Decided by: Codex
Rests on: API-011, API-018, API-020
Would be wrong if: The terminal event loop blocks on a slow reader, queued events grow without a bound, a bus failure is silently accepted, shutdown leaves a worker or accessible tree behind, or one App failure breaks another App.
History: The real reader workflow passes, but source review found a process-global unbounded outgoing queue, an unowned worker, ignored connection errors and no restart after worker failure. The incoming App action queue alone does not bound transport.

## Decision

Give each private Unix adapter its own application context, bounded outgoing queue, cancellation signal, worker join handle and failure notification to App. Queue overload and bus failures terminate that connection with an actionable App error; do not drop events and keep reporting success. Bound each asynchronous bus operation and cancel pending work during shutdown. Join the owned worker before terminal cleanup. A later App starts a fresh connection. Auto-enable only on interactive Linux hosts with a desktop session-bus address; explicit screen_reader(true) attempts startup and reports failure even without one. Verify overload, cancellation during a stalled operation, startup failure, failure propagation, successive and simultaneous App isolation, and the real Orca workflows.

## Realized by

Implementation: `src/accessibility/connection.rs`, `src/accessibility/platform/unix/transport.rs`.

Behavior checks: `src/accessibility/platform/unix/transport/tests.rs`, `tests/api_widget_behavior/transport_failures.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
