# Pre-release CI and documentation

Status: Agreed 2026-09-14
Prefix: RID

This specification remediates Fable audit findings B3, B4, and L19. It covers
continuous integration, tracked documentation inputs, and repository
housekeeping required before final packaging.

[RID-001]
Pushes and pull requests to main MUST run the default suite and maintenance
checks on Linux, macOS, and Windows. A scheduled workflow MUST run the security
advisory checks.
Falsifier: A main-branch change can merge without a supported-platform build,
test, formatting, lint, or maintenance result, or no scheduled advisory check
exists.
Mechanism: Workflow inspection and event fixtures verify triggers, the three
operating-system jobs, required commands, locked dependencies, and the
scheduled security job.

[RID-002]
README, include documentation, ABI policy, baselines, and every Cairn mechanism
MUST point to tracked source-grounded documents. The complete mechanism set
MUST run after the links and inputs change.
Falsifier: A tracked document links to a missing local target, a mechanism
declares an ignored or missing input, or any mechanism lacks fresh passing
evidence after relocation.
Mechanism: A repository link and declared-input checker runs before all Cairn
requirements are checked against the committed tree.

[RID-003]
Tracked files and ignore rules MUST describe the repository that is intended to
remain. Stale exclusions and exceptions MUST be removed. No unreviewed
decision MAY remain queued at release review.
Falsifier: A tracked file unintentionally matches an ignore rule, an ignore or
package exclusion names a removed path, or `.cairn/queue` contains an
unreviewed decision when final packaging starts.
Mechanism: Housekeeping checks compare tracked paths with ignore and package
rules and require the developer-reviewed decision queue to be empty.
