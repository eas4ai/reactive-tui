# Mechanism: api-styling

command: python3 -B scripts/check-api-styling.py
inputs:
  - .cairn/mechanisms/api-styling.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_styling.rs
  - tests/support/app_input.rs
  - scripts/check-api-styling.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-009
