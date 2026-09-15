# Mechanism: suprtui-renderer

command: sh scripts/check-renderer.sh
inputs:
  - docs/spec
  - README.md
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - reactive-tui-macros
  - src
  - tests/suprtui_renderer.rs
  - tests/runtime_probes/suprtui_renderer.rs
  - scripts
requirements:
  - RND-001
  - RND-002
  - RND-003
  - RND-004
  - RND-005
  - RND-006
