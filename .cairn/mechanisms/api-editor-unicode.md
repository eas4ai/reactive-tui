# Mechanism: api-editor-unicode

command: python3 -B scripts/check-api-editor-unicode.py
inputs:
  - .cairn/mechanisms/api-editor-unicode.md
  - Cargo.toml
  - Cargo.lock
  - src
  - reactive-tui-macros
  - tests/api_editor_unicode.rs
  - scripts/check-api-editor-unicode.py
  - docs/spec/rust-api-remediation.md
  - docs/editor-positions.md
requirements:
  - API-007
