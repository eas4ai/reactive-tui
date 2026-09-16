# Pre-release CI and documentation review

## RID-001 mechanism construction

The contract requires main push and pull-request coverage for Linux, macOS,
and Windows, build/test/format/lint/maintenance commands, locked dependencies,
and scheduled advisory checks. No maintained main-branch workflow exists yet.

On 2026-09-16 the eight validator tests first failed because the checker was
absent. After implementation all eight passed. The valid workflow fixture is
accepted. Removing each of three platforms and each of five required commands
is rejected, as are branch/path/activity filters, disabled jobs, conditional
required commands, allowed failures, unlocked and success-masked commands,
missing advisory coverage, floating actions, and write permissions.

Repository inspection has not yet passed. The checker must reject the actual
tree until the main-branch workflow exists. Native CI execution is not covered
by these local fixtures. Read-only GitHub inspection found no legacy branch
protection and one active ruleset with creation/update/deletion restrictions
but no required-status rule. Changing remote merge policy requires separate
developer authority; do not claim workflow presence enforces passing checks.

RID-002 and RID-003 remain pending. Do not publish packages or remove queued
developer decisions without review.
