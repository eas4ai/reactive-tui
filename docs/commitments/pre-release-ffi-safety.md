# Commitment: pre-release-ffi-safety

Status: Agreed 2026-09-14
Requirements: FFS-001, FFS-002, FFS-003, FFS-004, FFS-005

## Deliverable

Make every maintained C ABI call safe for its documented lifetime, ownership,
nullability, panic, and concurrency rules. Regenerate and verify the C header
and TypeScript declarations against the repaired Rust exports.

## Boundaries

This commitment changes `src/ffi/`, `include/`, TypeScript binding declarations
and consumers, focused ABI tests, and their Cairn mechanisms. It may change an
export whose current contract is unsound. It does not change unrelated Rust
framework APIs or final package metadata.

Done-when: FFS-001 through FFS-005 pass; safe violating cases demonstrate that
the checks reject stale handles, invalid ownership sequences, nullability
mismatches, and ABI panics; final review finds no unguarded export or
undocumented ownership transfer.
