# retain isolated taffy lines across layout engines

Level: Judged
Decided by: Codex
Rests on: DQC-002
Would be wrong if: Taffy 0.13 can replace the root 0.9 line without changing layout semantics, or Taffy types cross the root and renderer boundary.

## Decision

Retain taffy 0.9.2 in reactive-tui and taffy 0.13.0 in reactive-tui-suprtui for this release. Reason: the renderer owns its 0.13 layout tree internally; the root owns its 0.9 layout tree and exposes project layout values instead of Taffy types. A trial alignment produced 55 compile errors across alignment and display mappings, so migration is broader than dependency convergence and risks changing layout behavior. Keep both exact versions visible in the DQC-002 graph report and re-evaluate after a dedicated layout migration with parity tests.

## Realized by

- 90e77044b12e8b4d7c50b37d78f444faa230e6a8 fix: enforce DQC-002 dependency maintenance
