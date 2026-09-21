# Commitment: registry-cache-isolation

Status: Agreed 2026-09-08
Requirements: CCH-001, CCH-002, REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Repair named lookup across independent registries and shared clones, with
regressions for registry lifetime, registration and clearing across threads.
Record a fresh default-feature full-suite assessment, FFI test compilation,
formatting and warning-denied Clippy results after the repair. The assessment
records remaining failures with evidence; it does not declare them repaired.

## Boundaries

Registry lookup internals and focused tests are in scope. Keep public APIs and
constructor reentry. Prefer one authoritative name map over an unmeasured cache
optimization. Preserve all inherited requirements. Host-specific input,
animation/CSS semantics, FFI repair and unrelated rewrites require their own
commitments informed by the refreshed assessment.

Done-when: all named requirements pass, violating lookup examples fail the
checks, the full-suite assessment is recorded, and isolation/ownership is reviewed.
