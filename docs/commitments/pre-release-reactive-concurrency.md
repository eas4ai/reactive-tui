# Commitment: pre-release-reactive-concurrency

Status: Agreed 2026-09-14
Requirements: RAC-001, RAC-002, RAC-003

## Deliverable

Give reactive effects, animation hooks, references, and timers stable owners
with bounded work and cancellation. Run caller code only after internal borrows
and locks are released.

## Boundaries

This commitment changes `src/reactive/`, reactive and animation hooks, timer
scheduling, focused lifecycle tests, and their Cairn mechanisms. It preserves
the documented hook results and does not change final package metadata.

Done-when: RAC-001 through RAC-003 pass; re-entrant and unmount cases fail the
old mechanisms and pass the repaired ones; thread and timer counts remain
bounded during repeated renders and shutdown.
