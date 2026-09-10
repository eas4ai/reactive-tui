# Mechanism: binding-typescript-consumer

command: python3 -B scripts/check-typescript-abi.py
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
  - ABI-003
