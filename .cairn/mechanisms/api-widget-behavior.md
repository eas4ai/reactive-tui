# Mechanism: api-widget-behavior

command: python3 -B scripts/check-api-widget-behavior.py
inputs:
  - .cairn/mechanisms/api-widget-behavior.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_widget_behavior.rs
  - tests/support/app_input.rs
  - scripts/check-api-widget-behavior.py
  - docs/spec/rust-api-remediation.md
  - docs/widget-acceptance.md
requirements:
  - API-011
