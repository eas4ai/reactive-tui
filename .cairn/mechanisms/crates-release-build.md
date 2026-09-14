# Mechanism: crates release build

command: bash scripts/check-crates-release-build.sh
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - crates/libghostty-vt
  - crates/libghostty-vt-sys
  - src/backend/crossterm
  - src/backend/engine
  - scripts
  - docs/spec/crates-io-release-preparation.md
  - docs/commitments/crates-io-release-preparation.md
  - .cairn/mechanisms/crates-release-build.md
requirements:
  - CRT-003
reviewed:
  - CRT-003 sha256:aa26087cf12b195f9bfc15d3afba2f4aac611f3591b4720a54a7ea1fc04a9224
  - CRT-003 sha256:9c94641b1f2578b1106e722ee97c57834564c340e219ed19ca209482acbfdf27
