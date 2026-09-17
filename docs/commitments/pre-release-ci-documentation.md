# Commitment: pre-release-ci-documentation

Status: Agreed 2026-09-14
Requirements: MNT-002, RTR-001, RAC-001, RTR-004, DQC-002, DQC-001, XIS-001, RAC-003, FFS-001, FFS-004, FFS-005, FFS-003, FFS-002, XIS-003, XIS-002, DQC-003, DQC-005, RAC-002, TRL-004, TRL-003, TRL-001, TRL-002, DQC-004, RTR-002, RTR-003, API-013, API-008, API-020, API-002, API-012, API-018, API-007, API-016, API-005, API-015, API-006, API-004, API-003, API-014, API-017, API-010, API-019, API-001, API-009, API-011, WAK-001, WAK-002, WAK-003, WAK-004, WAK-005, ABI-001, ABI-002, ABI-004, ABI-003, DFT-001, DFT-002, DFT-003, DFT-004, DOC-001, EMB-001, EMB-002, EMB-003, EMB-004, EMB-005, EMB-006, EXC-001, MAN-001, MAN-002, MNT-003, MNT-004, MNT-001, CCH-001, CCH-002, REG-001, REG-002, REG-003, RND-001, RND-002, RND-003, RND-004, RND-005, RND-006, GPU-003, GPU-001, GPU-004, GPU-005, GPU-002, CAT-001, CAT-003, CAT-002, RID-001, RID-002, RID-003

## Activation

Shawn approved this existing roadmap commitment on 2026-09-16 by answering
`ok` to escalation `cat-001-cat-002-cat-003-rid-001-rid-002-rid-003-loop-087`.
The catalog requirements pass and its closing review is clean. Preserve its
pause history and the graphics evidence. This approval does not promote
unrelated backlog work or authorize publication.
See [the decision](../decisions/start-the-approved-pre-release-ci-and-documentation-commitment.md).

## Deliverable

Run the maintained suites on every supported operating system for main-branch
changes, schedule advisory checks, relink all tracked documentation and Cairn
inputs, and remove stale repository housekeeping rules.

## Boundaries

This commitment changes GitHub workflows, README and include links, manual and
Cairn source documents, mechanism declarations, ignore rules, and stale tracked
artifacts. Developer review controls the decision queue. It does not create
release archives or publish anything.

On 2026-09-16 Shawn approved escalation `rid-001-rid-002-mnt-002`, expanding
these boundaries only to source repairs required by strict Clippy and pushing
the pending catalog/graphics/CI history for native CI. After all three
supported-platform jobs pass, add a main-only required-status ruleset while
preserving the existing ruleset. This does not authorize package publication,
unrelated runtime fixes, or removal of unreviewed decision queue entries.
See [the decision](../decisions/permit-strict-lint-repairs-and-native-ci-enforcement.md).

On 2026-09-17 Shawn directed completion of the Fable audit's technical
remediation before decision-queue review. The existing technical requirements
above make the complete non-packaging mechanism set an explicit acceptance
gate, ahead of RID-003. Verify fresh results locally on Linux, Windows through
WinBoat, and the configured macOS host. Preserve all nine queue markers.
Do not push, dispatch GitHub CI, or change remote rulesets without separate
explicit approval. Packaging and publication remain blocked.
See [the ordering decision](../decisions/verify-fable-technical-remediation-before-decision-queue-review.md).

Done-when: RID-001 through RID-003 pass; push, pull-request, and scheduled event
fixtures select the required jobs; every local link and mechanism input exists;
all mechanisms have fresh evidence; the developer has reviewed the queued
decisions; final review finds no stale ignore or package rule.
