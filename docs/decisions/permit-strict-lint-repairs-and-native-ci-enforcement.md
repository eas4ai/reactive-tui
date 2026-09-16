# Permit strict lint repairs and native CI enforcement

Level: Consequential
Decided by: Shawn
Rests on: RID-001 RID-002 MNT-002
Would be wrong if: The work repairs unrelated runtime defects, publishes packages, removes developer decision queue entries without review, changes the existing GitHub ruleset, or requires checks before all supported platforms pass.
History: The CI/documentation commitment originally excluded framework source changes and publication. Its workflow inspection now passes, but inherited strict lint failures and absent remote required checks prevent native acceptance. Shawn approved this narrow expansion without authorizing package publication.

## Decision

Shawn answered ok to escalation rid-001-rid-002-mnt-002 on 2026-09-16. Expand only to source repairs required by strict Clippy, pushing the pending catalog/graphics/CI history for native CI, and adding a main-only required-status ruleset after Linux, macOS, and Windows jobs pass. Preserve the existing ruleset. Keep package publication and unrelated backlog work out of scope. Native failures outside these narrow source repairs need a separate decision.

## Realized by

(none yet: recorded, not built)
