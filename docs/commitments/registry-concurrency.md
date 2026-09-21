# Commitment: registry-concurrency

Status: Agreed 2026-09-07
Requirements: REG-001, REG-002, REG-003, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006

## Deliverable

Reproduce and repair the component registry concurrency stall, with bounded
regressions for concurrent metrics, instance mutation and cleanup reentry.
Preserve public registry signatures and isolate global performance assertions.

## Boundaries

Registry internals, tracked cleanup and their test harness are in scope.
Unrelated registration cache semantics, host input reports, legacy feature
failures, FFI and terminal dependency replacement stay in the backlog.

Done-when: all named requirements have current passing evidence, violating
locking examples fail the checks, and ownership and lock ordering are reviewed.
