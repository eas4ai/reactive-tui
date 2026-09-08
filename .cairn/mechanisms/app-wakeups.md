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
  - examples/wake_counter.rs
  - examples/embedded_shell.rs
  - examples/suprtui_counter.rs
requirements:
  - WAK-001
  - WAK-002
  - WAK-003
  - WAK-004
  - WAK-005
reviewed:
  - WAK-001 sha256:60beb2548dfa00f5af0c5b9bfbd66f8d6772fe026c01b9a28ba68c53dffd0276
