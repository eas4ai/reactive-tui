# Mechanism: api-component-expansion

command: python3 -B scripts/check-api-component-expansion.py
inputs:
  - .cairn/mechanisms/api-component-expansion.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_component_expansion.rs
  - scripts/check-api-component-expansion.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-002
