# Mechanism: pre-release-ffi-ownership

command: python3 -B scripts/check-pre-release-ffi-safety.py FFS-003
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src/animation
  - src/ffi/animation.rs
  - src/ffi/app.rs
  - src/ffi/pointer.rs
  - src/ffi/stats.rs
  - include
  - bindings/typescript
  - tests
  - scripts/check-pre-release-ffi-safety.py
requirements:
  - FFS-003

The check MUST run a C consumer through failed AppBuilder construction and
cleanup, animation creation and manager transfer followed by rejected stale
operations, and manager cleanup. It MUST race callback replacement with log
delivery and use a callback that replaces itself, proving the callback is not
invoked under the storage lock. Sanitizer-backed execution and Rust concurrency
tests MUST report no double free, stale access, data race, or deadlock. Header
ownership text and the runtime sequence must agree.
