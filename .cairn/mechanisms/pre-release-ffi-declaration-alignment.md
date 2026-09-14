# Mechanism: pre-release-ffi-declaration-alignment

command: python3 -B scripts/check-pre-release-ffi-safety.py FFS-005
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - reactive-tui-macros
  - src
  - include
  - bindings/typescript
  - examples
  - docs/spec/pre-release-ffi-safety.md
  - docs/spec/binding-abi-compatibility.md
  - docs/decisions/use-implemented-rust-exports-as-the-binding-compatibility-baseline.md
  - scripts
  - tests/binding_abi_consumer.c
requirements:
  - FFS-005

The check MUST regenerate the C header, TypeScript native schema, and generated
TypeScript types with the pinned generator and require a clean result. It MUST
then run the existing strict C/C++ consumer and TypeScript native consumer
against the compiled library. It MUST reject examples or ABI ownership policy
that name missing exports, use the wrong arity, or contradict the generated
declarations.
