# Mechanism: api-entry-points

command: python3 -B scripts/check-api-entry-points.py
inputs:
  - .cairn/mechanisms/api-entry-points.md
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - include
  - tests/suprtui_renderer.rs
  - scripts/check-api-entry-points.py
  - verification/api-entry-points
  - docs/application-entry-points.md
requirements:
  - API-016
