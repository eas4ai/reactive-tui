# Mechanism: default-suite-repair

command: python3 scripts/check-default-suite.py
inputs:
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - reactive-tui-macros
  - src
  - scripts
  - tests
  - examples
requirements:
  - DFT-001
  - DFT-002
  - DFT-003
  - DFT-004
