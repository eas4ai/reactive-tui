# Mechanism: api-native-components

command: python3 -B scripts/check-api-native-components.py
inputs:
  - .cairn/mechanisms/api-native-components.md
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src
  - reactive-tui-macros
  - include
  - bindings/typescript
  - scripts/abi
  - scripts/check-binding-abi.py
  - scripts/generate-native-header.py
  - scripts/check-api-native-components.py
  - scripts/check-api-entry-points.py
  - tests/suprtui_renderer.rs
  - verification/api-entry-points/screen.rs
  - verification/api-native-components
  - docs/native-components.md
  - docs/binding-abi-baseline.json
  - docs/binding-native-baseline.json
  - docs/binding-abi-migration.json
  - docs/binding-abi-migration.md
  - docs/binding-typescript-migration.md
  - docs/binding-typescript-public-api.json
  - docs/binding-typescript-native-baseline.json
requirements:
  - API-017
