# Pre-release audit remediation map

Status: Agreed 2026-09-14

The source report is
`.cairn/backlog/resolve-the-2026-09-14-pre-release-audit-findings-before-a-public-release.md`.
The report contains 4 Blocker, 6 High, 12 Medium, and 19 Low findings. Each
finding has one primary remediation commitment below. Final packaging follows
all remediation work.

| Commitment | Primary audit findings |
| --- | --- |
| `pre-release-ffi-safety` | H1, H2, H3, M9, L13, L14, L15, L16 |
| `pre-release-terminal-lifecycle` | H4, M5, L6, L7, L10, L12, L18 |
| `pre-release-reactive-concurrency` | H5, M6, L2, L3, L4, L5 |
| `pre-release-runtime-resilience` | M1, M3, M4, M7, L1 |
| `pre-release-external-input-safety` | M2, L8, L9, L11 |
| `pre-release-dependency-code-quality` | H6, M8, M11, M12, L17 |
| `pre-release-ci-documentation` | B3, B4, L19 |
| `final-release-packaging` | B1, B2, M10 |

M12 and L19 are rechecked during final packaging because package identity and
repository housekeeping affect the archive. Their primary implementation stays
in the earlier commitments shown above.
