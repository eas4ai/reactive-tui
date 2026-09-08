# Mechanism: maintenance-lint

command: python3 scripts/check-maintenance.py lint
inputs:
  - Cargo.toml
  - Cargo.lock
  - reactive-tui-macros
  - src
  - tests
  - benches
  - examples
  - scripts
  - include
requirements:
  - MNT-002
