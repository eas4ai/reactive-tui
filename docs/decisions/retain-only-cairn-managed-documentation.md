# Retain only Cairn-managed documentation

Level: Consequential
Decided by: developer
Rests on: developer agreement 2026-09-14 and the existing docs ignore boundary
Would be wrong if: A removed document is still required at runtime or a retained Cairn contract artifact becomes unreadable

## Decision

Track documentation only under docs/spec, docs/commitments, and docs/decisions. Remove other tracked documentation and keep the broader docs path ignored. Preserve all .cairn records.

## Realized by

- 9aa6ccac Define Cairn-only documentation retention
