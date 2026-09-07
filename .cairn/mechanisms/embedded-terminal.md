# Mechanism: embedded-terminal

command: sh scripts/check-embedded-terminal.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - README.md
  - docs/embedded-terminal.md
  - reactive-tui-macros
  - src
  - tests/embedded_terminal.rs
  - tests/suprtui_renderer.rs
  - examples/embedded_shell.rs
  - examples/suprtui_counter.rs
  - scripts
requirements:
  - EMB-001
  - EMB-002
  - EMB-003
  - EMB-004
  - EMB-005
  - EMB-006
