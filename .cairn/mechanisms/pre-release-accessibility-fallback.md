# Mechanism: pre-release-accessibility-fallback

command: python3 -B scripts/check-pre-release-runtime-resilience.py RTR-001
inputs:
  - .cairn/mechanisms/pre-release-accessibility-fallback.md
  - Cargo.toml
  - Cargo.lock
  - src/app.rs
  - src/accessibility
  - src/backend/suprtui.rs
  - tests/api_widget_behavior/accessibility_probe.rs
  - tests/api_widget_behavior/transport_failures.py
  - scripts/check-pre-release-runtime-resilience.py
requirements:
  - RTR-001

The check MUST first prove its observation validator rejects an automatic App
that exits with an error, an automatic App that produces no painted first
frame, an explicit request that silently succeeds, and an explicit request that
loses the connection error. It MUST then give the same interactive App a
missing session-bus socket and a socket that accepts but never answers.

For each endpoint, automatically selected accessibility MUST paint a frame,
continue through the connection failure, accept ordinary input, exit normally,
and restore the terminal. Explicitly requested accessibility MUST return a
session-bus connection error, including the bounded deadline for the stalled
endpoint, and restore the terminal.
