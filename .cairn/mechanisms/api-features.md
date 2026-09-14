# Mechanism: api-features

command: python3 -B scripts/check-api-features.py
inputs:
  - .cairn/mechanisms/api-features.md
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - tests
  - examples
  - benches
  - scripts/check-api-features.py
requirements:
  - API-015
