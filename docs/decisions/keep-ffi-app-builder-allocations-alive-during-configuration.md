# Keep FFI app-builder allocations alive during configuration

Level: Judged
Decided by: Shawn and Codex
Rests on: MNT-003
Would be wrong if: A setter releases the builder allocation, a failed backend setup loses existing configuration, or the callback consumes the wrong wrapper type.

## Decision

The ordinary FFI app-builder test exposed invalid allocation ownership in the setters. Replace temporary Box ownership with a mutable reference and mem::take, preserving the allocation while invoking consuming Rust builder methods. Construct fallible backends before taking builder state. Decode root callback results as FFIElement and extract inner Element. Retain all C signatures and the documented consuming build/run operations. Verify the existing configuration/build/destroy workflows after repair; do not rerun the unsafe original path.

## Realized by

- 24a6ad39aac8269454346d74455cf1cbee0593ae Restore terminal FFI linkage and preserve handle ownership
