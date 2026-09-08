# Mechanism: binding-c-consumer

command: python3 -B scripts/check-c-binding-abi.py
inputs:
  - Cargo.toml
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
  - tests/binding_abi_consumer.c
requirements:
  - ABI-002
