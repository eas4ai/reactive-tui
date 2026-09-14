# Mechanism: registry-cache-isolation

command: sh scripts/check-registry-cache-isolation.sh
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
  - CCH-001
  - CCH-002
