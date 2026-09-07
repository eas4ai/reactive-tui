# Mechanism: suprtui-renderer

command: sh scripts/check-renderer.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - reactive-tui-macros
  - src
  - tests/suprtui_renderer.rs
  - examples/suprtui_counter.rs
  - scripts
requirements:
  - RND-001
  - RND-002
  - RND-003
  - RND-004
  - RND-005
  - RND-006
