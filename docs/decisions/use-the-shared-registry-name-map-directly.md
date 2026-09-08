# Use the shared registry name map directly

Level: Judged
Decided by: Shawn and Codex
Rests on: CCH-001 CCH-002
Would be wrong if: Named lookup resolves stale or foreign registrations, constructor reentry regresses, or measured contention makes the direct lookup unsuitable.

## Decision

Remove the thread-local name cache and its copied version counter. Resolve names through the requested registry shared name map, releasing its read guard before looking up and invoking a factory. Independent registries keep independent maps and clones already share the correct map. This adds a short read lock per named lookup but removes duplicated maps and invalidation machinery. Retain the existing public APIs and duplicate-registration behavior; consider a cache only after measured need.

## Realized by

- 27b23aed1c1e6a7ba95f48f30f3daaa72ad68e66 fix: isolate registry lookup using shared authoritative names
