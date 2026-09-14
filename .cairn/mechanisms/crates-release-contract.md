# Mechanism: crates release contract

command: python3 scripts/check-crates-release.py
inputs:
  - Cargo.toml
  - Cargo.lock
  - README.md
  - CHANGELOG.md
  - LICENSE
  - manual
  - reactive-tui-macros/Cargo.toml
  - reactive-tui-macros/README.md
  - reactive-tui-macros/LICENSE
  - reactive-tui-macros/src
  - src/backend/crossterm/Cargo.toml
  - src/backend/crossterm/README.md
  - src/backend/crossterm/LICENSE
  - src/backend/crossterm/REACTIVE_TUI_PATCH.md
  - src/backend/crossterm/src
  - src/backend/engine/Cargo.toml
  - src/backend/engine/README.md
  - src/backend/engine/LICENSE
  - src/backend/engine/LICENSE-OpenTUI
  - src/backend/engine/UPSTREAM.md
  - src/backend/engine/src
  - scripts/check-crates-release.py
  - docs/spec/crates-io-release-preparation.md
  - docs/commitments/crates-io-release-preparation.md
  - .cairn/mechanisms/crates-release-contract.md
requirements:
  - CRT-001
  - CRT-002
