# FFI, lint and formatting repair review

## Work tracking

- Done: establish the commitment and reproduce the formatting gate.
- Done: format workspace Rust and verify unchanged behavior.
- In progress: repair strict Clippy findings and verify behavioral changes.
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

## Clippy repair and failure demonstration

Strict cargo clippy --locked --all-targets -- -D warnings now exits zero.
The full default suite passes with 1014 passed, zero failed and 35 ignored.
Workspace formatting and git diff --check also pass. Existing asserts remain;
constant-false branches now panic explicitly, while tautological assertions now
check CSS properties, surface cells, animation defaults and capability mapping.
A deliberate mutation that discards a requested CSS foreground color fails the
strengthened color test with exit 101 (white instead of red). After restoration,
the full suite passes. The original strict gate supplies the failing lint case.

Reviewed the automatic suggestions: derived enum defaults keep the same variant;
unit struct construction, range checks and empty-collection checks retain their
meaning. Match guards retain the existing wildcard fallthrough. Reverse sort
keys retain descending z order and stable ties. Checked division keeps the zero
denominator defaults. Counter iterators start at the same offsets and retain
bounds checks. Menu callback aliases retain their exact Arc/dyn trait type.
The only new lint exception is the recorded expectation on MenuTheme to retain
Custom(MenuStyle) construction without boxing or adding a new allocation.
