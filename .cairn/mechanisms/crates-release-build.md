# Mechanism: crates release build

command: bash scripts/check-crates-release-build.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - src/backend/crossterm
  - src/backend/engine
  - scripts/check-crates-release-build.sh
  - docs/spec/crates-io-release-preparation.md
  - docs/commitments/crates-io-release-preparation.md
  - .cairn/mechanisms/crates-release-build.md
requirements:
  - CRT-003
