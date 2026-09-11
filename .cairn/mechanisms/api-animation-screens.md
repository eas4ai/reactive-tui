# Mechanism: api-animation-screens

command: sh -c 'cargo test --lib keyframe_ -- --test-threads=1 && cargo test --test api_animation_screens --test api_hook_lifecycle -- --test-threads=1'
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
