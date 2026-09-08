# FFI, lint and formatting repair

Status: Agreed 2026-09-08
Prefix: MNT

The developer authorized this next commitment after the default-suite repair.

[MNT-001]
All Rust workspace packages MUST pass cargo fmt --all -- --check.
Formatting MUST preserve behavior and test coverage.
Falsifier: the formatting command fails or a formatting-only edit changes behavior.
Mechanism: scripts/check-maintenance.py format.

[MNT-002]
The default-feature workspace targets MUST pass cargo clippy --locked --all-targets -- -D warnings.
Repairs MUST resolve the diagnosed cause without blanket lint suppression or removing test assertions to hide defects.
Falsifier: strict Clippy fails, a new broad allow hides warnings, or a behavioral lint repair lacks meaningful verification.
Mechanism: scripts/check-maintenance.py lint and inherited default-suite checks.

[MNT-003]
The ffi-feature test targets MUST compile and link, including ffi_tests and test_minimal_ffi.
Repairs MUST retain the published C signatures and exercise repaired entry points for valid lifecycle and null-argument behavior.
Falsifier: cargo test --locked --features ffi --no-run fails, an existing exported signature is removed, or repaired lifecycle/null-argument checks fail.
Mechanism: scripts/check-maintenance.py ffi plus focused FFI acceptance tests.

[MNT-004]
All inherited default-suite, registry, wakeup, embedded-terminal and renderer requirements MUST retain passing evidence after maintenance repairs.
Falsifier: any inherited requirement fails or coverage is disabled to obtain a pass.
Mechanism: inherited acceptance declarations.
