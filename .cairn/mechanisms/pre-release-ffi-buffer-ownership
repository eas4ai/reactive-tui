# Mechanism: pre-release-ffi-buffer-ownership

command: python3 -B scripts/check-pre-release-ffi-safety.py FFS-004
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src/ffi/component.rs
  - src/ffi/lib.rs
  - src/ffi/pointer.rs
  - src/ffi/text.rs
  - include
  - bindings/typescript
  - tests
  - scripts/check-pre-release-ffi-safety.py
requirements:
  - FFS-004

The check MUST run a sanitizer-backed C consumer that releases each returned
array with an incorrect caller length, releases it again, destroys optimized
and text buffers twice, and attempts to add an element as its own child. All
cases MUST complete with documented results and no invalid memory access. The
check MUST also require the pointer-validation contract to say that plausibility
checks do not prove allocation, liveness, ownership, or readability.
