# Mechanism: pre-release-threaded-event-loop

command: python3 -B scripts/check-pre-release-runtime-resilience.py RTR-002
inputs:
  - .cairn/mechanisms/pre-release-threaded-event-loop.md
  - Cargo.toml
  - Cargo.lock
  - src/platform/loop.rs
  - src/platform/input_receiver.rs
  - src/platform/parser.rs
  - scripts/check-pre-release-runtime-resilience.py
requirements:
  - RTR-002

The check MUST first prove its validator rejects a timeout, a failed process,
and missing completion. It MUST then run the public `ThreadedEventLoop` in an
isolated pseudo-terminal for three bounded cases.

The saturation case MUST queue more parsed input events than the declared
capacity before consuming and then require consumer progress. The idle-stop
case MUST call `stop` while the input reader is blocked with no input. The Drop
case MUST release a running loop and require the process thread count to return
to its baseline. Every case MUST finish within its deadline and report explicit
completion; killing a timed-out fixture is a failure.
