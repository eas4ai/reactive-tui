# Mechanism: app-wakeups

command: sh scripts/check-app-wakeups.sh
inputs:
  - docs/spec
  - Cargo.toml
  - build.rs
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
  - WAK-002 sha256:79ac5045fabc4832190c9539dec39d42c97325d0c4c974cf80a1a268729002fd
  - WAK-003 sha256:9cf558b71f488b08014a14fb49a319db11b6db4ba1fcf2d35646e12246e38008
