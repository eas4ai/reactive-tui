# Mechanism: pre-release-tokio-event-loop

command: python3 -B scripts/check-pre-release-tokio-event-loop.py
inputs:
  - .cairn/mechanisms/pre-release-tokio-event-loop.md
  - Cargo.toml
  - Cargo.lock
  - src/error.rs
  - src/platform/mod.rs
  - src/platform/loop.rs
  - src/platform/parser.rs
  - tests/tokio_event_loop_lifecycle.rs
  - scripts/check-pre-release-tokio-event-loop.py
requirements:
  - RTR-003

The check MUST first prove its process validator rejects a timeout, a failed
process, and missing lifecycle completion. It MUST then exercise the public
`TokioEventLoop` sync and async start and stop routes inside both current-thread
and multi-thread Tokio runtimes.

Sync routes MAY return an error that directs an async caller to the async route,
but they MUST NOT panic. Started tasks MUST stop within the test deadline. Both
runtime cases MUST report explicit completion, and a killed or timed-out test is
a failure.
