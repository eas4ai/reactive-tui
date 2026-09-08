# Restore declared terminal entry points and retain FFI handle ownership

Level: Judged
Decided by: Shawn and Codex
Rests on: MNT-003
Would be wrong if: The bridge diverges from the shipped C signatures, shutdown frees a caller-owned handle, or borrowed surfaces cannot be used safely during their owner lifetime.

## Decision

Restore the five missing terminal entry points used by ffi_tests, using the shipped C headers as the contract and core Terminal for operations. Keep modern exports. Treat timeout zero as nonblocking, retain event layouts, and return NotSupported for events the legacy payload cannot represent. Validate terminal ownership before destruction. Keep renderer shutdown separate from destruction by reusing a non-consuming terminal-restoration method; register renderer-owned surfaces separately from caller-owned surfaces. Fix Rust test linkage and exercise the repaired APIs under a controlled pseudo-terminal. TypeScript signature drift and unrelated modern-header layout drift remain separate recorded work.

## Realized by

(none yet: recorded, not built)
