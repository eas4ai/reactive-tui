# Mechanism: app-wakeups

command: sh scripts/check-app-wakeups.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - README.md
  - docs/app-wakeups.md
  - reactive-tui-macros
  - src
  - scripts
  - tests/app_wakeups.rs
  - tests/embedded_terminal.rs
  - tests/suprtui_renderer.rs
  - examples/embedded_shell.rs
  - examples/suprtui_counter.rs
requirements:
  - WAK-001
  - WAK-002
  - WAK-003
  - WAK-004
  - WAK-005
