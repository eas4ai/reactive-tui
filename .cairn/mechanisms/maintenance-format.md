# Mechanism: maintenance-format

command: python3 scripts/check-maintenance.py format
inputs:
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - reactive-tui-macros
  - src
  - tests
  - benches
  - examples
  - scripts
  - include
requirements:
  - MNT-001
