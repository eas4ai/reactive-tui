# Mechanism: api-hook-lifecycle

command: python3 -B scripts/check-api-hook-lifecycle.py
inputs:
  - .cairn/mechanisms/api-hook-lifecycle.md
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_hook_lifecycle.rs
  - scripts/check-api-hook-lifecycle.py
  - docs/spec/rust-api-remediation.md
requirements:
  - API-004
