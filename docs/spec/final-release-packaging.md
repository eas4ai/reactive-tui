# Final release packaging

Status: Agreed 2026-09-14
Prefix: FRP

This specification remediates Fable audit findings B1, B2, and M10 after every
pre-release remediation commitment has passed. It also revalidates the package
identity and housekeeping affected by M12 and L19. It does not authorize
publishing crates, creating a tag, or creating a hosted release.

[FRP-001]
The final package graph MUST resolve entirely through crates.io with
project-owned package identities, exact compatible dependency versions, and no
git-only normal dependency. Package ownership and registry availability MUST
be checked immediately before the release decision.
Falsifier: A package dry run resolves a path or git-only normal dependency,
selects an unrelated upstream package, or relies on an unavailable companion
version.
Mechanism: In a clean registry-style workspace, package and publish dry runs
resolve every archive only through the registry and verify package ownership.

[FRP-002]
Every final archive MUST be rebuilt from the fully remediated tree, remain
below the registry size limit, exclude Cairn records and internal binaries,
and include all source, licenses, notices, and manual pages required to build
and use that package.
Falsifier: An archive is at least 10 MB, contains an internal review artifact or
unapproved runtime binary, omits required source or legal material, or differs
from the remediated commit.
Mechanism: Package-list and archive inspection checks run for every publishable
crate and build each unpacked archive in isolation.

[FRP-003]
Version, repository URL, Rust minimum, native package layout, examples, and
release notes MUST agree across Cargo manifests, the TypeScript package, the C
ABI, README, changelog, and contribution guide.
Falsifier: Two release surfaces name different versions, repositories, minimum
Rust versions, nonexistent directories, or example paths.
Mechanism: A metadata consistency check reads each release surface and verifies
the named examples and native artifacts exist in the final packages.

[FRP-004]
The release candidate MUST pass the complete Cairn mechanism set, default and
maintenance suites on Linux, macOS, and Windows, dependency policy checks, and
a fresh adversarial audit. No Blocker or High finding MAY remain; every
remaining lower-severity exception MUST have a developer-approved decision.
Falsifier: A required platform or mechanism is missing, a check is stale, the
fresh audit reports an unresolved Blocker or High, or an exception lacks a
decision.
Mechanism: A committed release-candidate report links the exact commit to all
platform evidence, Cairn receipts, audit output, and approved exceptions.
