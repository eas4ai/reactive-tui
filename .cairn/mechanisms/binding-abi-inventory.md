# Mechanism: binding-abi-inventory

command: python3 -B scripts/check-binding-abi.py
inputs:
  - Cargo.toml
  - build.rs
  - Cargo.lock
  - reactive-tui-macros
  - src
  - include
  - bindings/typescript
  - scripts
  - docs/binding-abi-audit.md
  - docs/binding-abi-baseline.json
  - docs/binding-native-baseline.json
  - docs/binding-abi-migration.json
  - docs/binding-abi-migration.md
  - docs/binding-typescript-migration.json
  - docs/binding-typescript-native-baseline.json
  - docs/binding-typescript-public-api.json
  - docs/binding-typescript-migration.md
  - docs/binding-abi-demonstrations
requirements:
  - ABI-001
reviewed:
  - ABI-001 sha256:8ccfadc26a46308a5de031720c710c7255be2390c0d8b1973aafce8e23560a23
