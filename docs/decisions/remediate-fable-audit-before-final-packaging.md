# Remediate Fable audit before final packaging

Level: Consequential
Decided by: Shawn
Rests on: the 2026-09-14 Fable pre-release audit and the developer's instruction to remediate before packaging
Would be wrong if: a packaging change is required to make a remediation check possible, or a finding cannot be assigned to one independently verifiable commitment
History: The earlier packaging decision was reversed when the registry Ghostty package proved source-incompatible. This plan therefore keeps packaging last and requires each remediation commitment to preserve the source-grounded behavior before the final package graph is rebuilt.

## Decision

Treat every finding in the Fable audit backlog as release-blocking work. Execute separate commitments for C ABI safety, terminal lifecycle, reactive and animation concurrency, runtime resilience, external input safety, dependency and code quality, and release CI and documentation. Only after those commitments pass and a fresh cross-platform audit is clean may the project rebuild final crates.io packages and make a release decision.

## Realized by

- 658742220ec97fa68a153182aac57910364bc779 docs: sequence pre-release audit remediation
