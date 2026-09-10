# Mechanism: binding-c-consumer

command: python3 -B scripts/check-c-binding-abi.py
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
  - tests/binding_abi_consumer.c
requirements:
  - ABI-002
reviewed:
  - ABI-002 sha256:d8e46af26519da25a85cdcbaf60e6f9aac4f721346080e07e751827832c93e81
