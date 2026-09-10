# Mechanism: api-widget-behavior

command: python3 -B scripts/check-api-widget-behavior.py
inputs:
  - .cairn/mechanisms/api-widget-behavior.md
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests
  - benches
  - scripts/check-api-widget-behavior.py
  - scripts/check-dialog-http.py
  - scripts/check-widget-platforms.py
  - scripts/check-iterm-host.py
  - scripts/check-conpty-platform.py
  - scripts/install-conpty-runtime.py
  - docs/windows-terminal.md
  - .github/workflows/clipboard-platforms.yml
  - docs/binding-abi-baseline.json
  - docs/analysis/conpty-platform
  - docs/analysis/widget-platforms
  - docs/spec/rust-api-remediation.md
  - docs/widget-acceptance.md
  - docs/dialog-http.md
  - docs/dialog-input.md
requirements:
  - API-011
