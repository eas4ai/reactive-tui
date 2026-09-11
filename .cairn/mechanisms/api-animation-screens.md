# Mechanism: api-animation-screens

command: cargo test --test api_animation_screens -- --test-threads=1
inputs:
  - .cairn/mechanisms/api-animation-screens.md
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - tests
  - docs/spec/rust-api-remediation.md
requirements:
  - API-013
