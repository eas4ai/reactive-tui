# Mechanism: framework-manual

command: python3 scripts/check-framework-manual.py
inputs:
  - AGENTS.md
  - Cargo.toml
  - src/lib.rs
  - src/widgets/mod.rs
  - manual
  - scripts/check-framework-manual.py
  - docs/spec/framework-manual.md
  - docs/commitments/framework-manual.md
  - .cairn/mechanisms/framework-manual.md
requirements:
  - MAN-001
  - MAN-002
