# Mechanism: pre-release-ffi-panic-boundary

command: python3 -B scripts/check-pre-release-ffi-safety.py FFS-002
inputs:
  - Cargo.toml
  - Cargo.lock
  - build.rs
  - reactive-tui-macros
  - src/ffi
  - include
  - tests
  - scripts/check-pre-release-ffi-safety.py
requirements:
  - FFS-002

The check MUST compile and run a C consumer that writes text, shrinks below the
used length, grows again, writes again, and renders through each maintained
text-buffer route. It MUST verify length, capacity, exported storage, and
defined return values at every step. A complete syntax-aware export inventory
MUST require every exported Rust C function to enter an approved panic boundary
and MUST reject a safe fixture with one boundary removed. A source inventory or
a single runtime case alone cannot pass this requirement.
