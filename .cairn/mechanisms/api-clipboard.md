# Mechanism: api-clipboard

command: python3 -B scripts/check-api-clipboard.py
inputs:
  - .cairn/mechanisms/api-clipboard.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_clipboard.rs
  - scripts/check-api-clipboard.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-008
