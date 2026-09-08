# Mechanism: binding-abi-inventory

command: python3 scripts/check-binding-abi.py
inputs:
  - Cargo.toml
  - Cargo.lock
  - reactive-tui-macros
  - src
  - include
  - bindings/typescript
  - scripts
requirements:
  - ABI-001
