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
reviewed:
  - CRT-001 sha256:17a7bc4335252645bebc3e40947ce789b763a3a6a8b5ab3b937b37e5996e85ac
