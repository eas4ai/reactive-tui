# Release registry locks before component callbacks

Level: Judged
Decided by: Shawn and Codex
Rests on: REG-001 REG-002 REG-003
Would be wrong if: Concurrent operations still stall, callbacks reenter held locks, or detached instances fail to unmount once.

## Decision

Detach instances and copy factory handles while holding the relevant map lock, then release it before invoking constructors, lifecycle callbacks or destructors. Avoid nested instance/statistics locks; performance snapshots may span concurrent operations but settle after workers finish. Remove the unused strong registry ownership from tracked instances while preserving constructor signatures. Serialize only the legacy tests that assert whole-global-registry counts; add separate bounded concurrent and lifecycle regressions.

## Realized by

- 36581ee8c8265816468762194b97f05b1aed85e1 fix: release registry locks before callbacks and cleanup
