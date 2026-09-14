# Mechanism: api-dialog-lifecycle

command: python3 -B scripts/check-api-dialog-lifecycle.py
inputs:
  - .cairn/mechanisms/api-dialog-lifecycle.md
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests
  - benches
  - examples
  - scripts/check-api-dialog-lifecycle.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-012
