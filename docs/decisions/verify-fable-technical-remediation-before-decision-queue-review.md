# Verify Fable technical remediation before decision queue review

Level: Judged
Decided by: Shawn
Rests on: RID-002, RID-003, remediate-fable-audit-before-final-packaging, .cairn/escalations/rid-003.md
Would be wrong if: The ordering bypasses a technical check, removes any queued decision, permits packaging before developer review, or spends hosted CI minutes without separate approval.

## Decision

Shawn directed technical remediation FIRST on 2026-09-17. List the existing technical acceptance requirements ahead of RID-003 in the current commitment, including inherited regressions affected by documentation relocation. Preserve all nine queue markers and the final queue gate. Verify locally on Linux, WinBoat Windows, and the configured macOS host; do not push, dispatch GitHub workflows, or change remote rulesets without separate explicit approval. Final release packaging requirements remain deferred. This changes execution order, not any requirement or falsifier.

## Realized by

- c160765bcb9f160ef30e31bafc147bbe98c344bd docs: prioritize Fable technical verification before queue review
