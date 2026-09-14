# Mechanism: pre-release-ffi-app-lifetime

command: python3 -B scripts/check-pre-release-ffi-safety.py FFS-001
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - src/app.rs
  - src/ffi
  - include
  - bindings/typescript
  - tests
  - scripts/check-pre-release-ffi-safety.py
requirements:
  - FFS-001

The check MUST compile and run a C consumer that starts an application, requests
quit from another thread, waits for shutdown, and destroys every retained
handle. The same consumer MUST pass null for each optional callback. A safe
violating fixture with the former moved `Box<App>` ownership and bare nullable
function pointer MUST fail under the supported sanitizer or an equivalent
Rust lifetime and representation assertion. Source-text matching alone cannot
pass this requirement.
