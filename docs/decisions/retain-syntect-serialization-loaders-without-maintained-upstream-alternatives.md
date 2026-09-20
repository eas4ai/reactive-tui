# retain syntect serialization loaders without maintained upstream alternatives

Superseded by: highlight-with-lumis-tree-sitter-backend (2026-09-20, user vetoed companion publishes; syntect removed, RUSTSEC exceptions dropped)

Level: Judged
Decided by: Codex
Rests on: DQC-002
Would be wrong if: Syntect supports maintained serialization and YAML backends, or the documented built-in and caller-loaded syntax behavior no longer needs these loaders.

## Decision

Use syntect 5.3.0 with its fancy-regex backend and retain its transitive bincode 1.3.3 (RUSTSEC-2025-0141) and yaml-rust 0.4.5 (RUSTSEC-2024-0320) until Syntect supports maintained replacements. Exposure: bincode reads Syntect's embedded trusted syntax/theme dumps; yaml-rust parses syntax definitions from paths explicitly supplied by the caller. Neither package is currently reported vulnerable or unsound. Mitigation: default Markdown uses fancy-regex rather than Oniguruma; custom syntax loading is an explicit caller action for trusted local definitions rather than an implicit network path; cargo audit continues to reject vulnerabilities and unsound advisories; and DQC-002 checks these exact package versions and this active decision on every locked-graph change.

## Realized by

- 90e77044b12e8b4d7c50b37d78f444faa230e6a8 fix: enforce DQC-002 dependency maintenance
