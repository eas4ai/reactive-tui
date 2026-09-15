# Require cloned snapshots for reentrant Ref updates

Level: Consequential
Decided by: Codex
Supersedes: retain-shared-reference-hooks-and-invoke-ref-callbacks-outside-locks
Cause: an unforeseen condition occurred
Rests on: RAC-002 API-019 Miri
Would be wrong if: A safe implementation can synchronously re-enter Ref::update for non-Clone values without exposing overlapping mutable access or holding an internal lock.
History: API-019 preserved all existing Ref generic bounds. RAC-002 later required same-Ref callback re-entry, and a safe path-dependent consumer made Miri reject the raw-pointer implementation for invalid mutable aliasing.

## Decision

Keep Ref and use_ref available for non-Clone values, including set_current. Require T: Clone only on Ref::update so it can clone a committed snapshot, release internal locks, run the callback on an independent draft, and commit afterward. Remove the raw-pointer re-entry path. Preserve pointer stability by committing into the existing allocation. Update compatibility coverage to require non-Clone construction and set_current while rejecting non-Clone update. Add a safe alias-retention regression case and reject unsafe code in the shared Ref implementation.

## Realized by

(none yet: recorded, not built)
