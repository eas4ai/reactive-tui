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
  - tests/binding_abi_consumer.c
requirements:
  - ABI-002
reviewed:
  - ABI-002 sha256:d8e46af26519da25a85cdcbaf60e6f9aac4f721346080e07e751827832c93e81
