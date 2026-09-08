# Mechanism: maintenance-ffi

command: python3 scripts/check-maintenance.py ffi
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
  - docs/ffi-maintenance.md
requirements:
  - ABI-004
  - MNT-003
  - MNT-004
