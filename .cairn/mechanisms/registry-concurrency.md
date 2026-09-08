# Mechanism: registry-concurrency

command: sh scripts/check-registry-concurrency.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - reactive-tui-macros
  - src
  - scripts
  - tests/simple_performance_test.rs
  - tests/registry_concurrency.rs
requirements:
  - REG-001
  - REG-002
  - REG-003
