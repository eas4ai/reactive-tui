# Commitment: default-suite-repair

Status: Agreed 2026-09-08
Requirements: DFT-001, DFT-002, DFT-003, DFT-004, CCH-001, CCH-002, REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Resolve the six recorded default-suite failures with direct regressions and a
passing complete suite. Correct the contradictory JumpStart expectation using
the CSS step-easing contract. Reuse inset parsing without implicit position
changes. Keep all four documentation examples executable.

## Boundaries

Only these behavior/test/doc repairs and their acceptance checks are committed.
No FFI, broad formatting/Clippy, host-specific input or unrelated animation
changes. Existing explicit position defaults and class ordering remain intact.
Offsets now only set offsets; callers needing absolute positioning use absolute
or fixed explicitly, as with the existing non-numeric inset utilities.

Done-when: every named requirement passes, violating cases fail the checks,
and a final review covers semantics, compatibility and unchanged test coverage.
