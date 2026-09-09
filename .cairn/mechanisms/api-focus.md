# Mechanism: api-focus

command: python3 -B scripts/check-api-focus.py
inputs:
  - .cairn/mechanisms/api-focus.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_focus.rs
  - tests/support/app_input.rs
  - scripts/check-api-focus.py
  - docs/spec/rust-api-remediation.md
  - docs/app-focus.md
requirements:
  - API-006
