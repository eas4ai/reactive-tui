# Mechanism: binding-inherited

command: python3 -B scripts/check-inherited-abi.py
inputs:
  - .cairn/mechanisms/app-wakeups.md
  - .cairn/mechanisms/default-suite-repair.md
  - .cairn/mechanisms/embedded-terminal.md
  - .cairn/mechanisms/maintenance-ffi.md
  - .cairn/mechanisms/maintenance-format.md
  - .cairn/mechanisms/maintenance-lint.md
  - .cairn/mechanisms/registry-cache-isolation.md
  - .cairn/mechanisms/registry-concurrency.md
  - .cairn/mechanisms/suprtui-renderer.md
  - Cargo.lock
  - Cargo.toml
  - build.rs
  - README.md
  - benches
  - docs/app-wakeups.md
  - docs/embedded-terminal.md
  - docs/ffi-maintenance.md
  - docs/recon-evidence/cache-isolation
  - docs/spec
  - examples
  - examples/embedded_shell.rs
  - examples/suprtui_counter.rs
  - examples/wake_counter.rs
  - include
  - reactive-tui-macros
  - scripts
  - src
  - tests
  - tests/app_wakeups.rs
  - tests/component_macro_integration.rs
  - tests/component_macro_test.rs
  - tests/component_tests.rs
  - tests/embedded_terminal.rs
  - tests/registry_concurrency.rs
  - tests/simple_performance_test.rs
  - tests/suprtui_renderer.rs
requirements:
  - ABI-004
