# Mechanism: api-event-routing

command: python3 -B scripts/check-api-event-routing.py
inputs:
  - .cairn/mechanisms/api-event-routing.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_event_routing.rs
  - tests/support/app_input.rs
  - scripts/check-api-event-routing.py
  - docs/spec/rust-api-remediation.md
  - docs/app-events.md
requirements:
  - API-005
