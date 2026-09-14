# Mechanism: api-paint-properties

command: python3 -B scripts/check-api-paint-properties.py
inputs:
  - .cairn/mechanisms/api-paint-properties.md
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_paint_properties.rs
  - tests/support/app_input.rs
  - scripts/check-api-paint-properties.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-010
