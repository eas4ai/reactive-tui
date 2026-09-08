# FFI, lint and formatting repair review

## Work tracking

- Done: establish the commitment and reproduce the formatting gate.
- In progress: format workspace Rust and verify unchanged behavior.
- Pending: repair strict Clippy findings and verify behavioral changes.
- Pending: repair FFI compile/link integration and verify repaired ABI entry points.
- Pending: refresh all acceptance evidence and complete final review.

## Mechanism review plan

The prior assessment records real failures for all three commands. Each new
mechanism runs that exact command and propagates its exit status. Formatting
will be demonstrated by the existing differences and the corrected workspace.
Clippy findings need source review before automatic suggestions are accepted.
FFI compilation is necessary but not sufficient: add focused execution to its
mechanism when the missing functions and linkage corrections are implemented.
MNT-004 also requires all separately inherited requirements; the FFI receipt
alone does not prove those requirements.

Formatting baseline failed with real rustfmt differences. cargo fmt --all changed 158 files. The corrected formatting command and full default suite both exit zero; the suite retains 1014 passed, zero failed and 35 ignored. git diff --check passes. No manual behavior edits are mixed into this formatting commit.
