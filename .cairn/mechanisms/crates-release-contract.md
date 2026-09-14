# Mechanism: crates release contract

command: python3 scripts/check-crates-release.py
inputs:
  - Cargo.toml
  - Cargo.lock
  - .gitignore
  - README.md
  - CHANGELOG.md
  - LICENSE
  - manual
  - reactive-tui-macros
  - crates/libghostty-vt
  - crates/libghostty-vt-sys
  - src/backend/crossterm
  - src/backend/engine
  - scripts
  - docs/spec/crates-io-release-preparation.md
  - docs/commitments/crates-io-release-preparation.md
  - .cairn/mechanisms/crates-release-contract.md
requirements:
  - CRT-001
  - CRT-002
