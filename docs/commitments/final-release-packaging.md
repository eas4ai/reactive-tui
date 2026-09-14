# Commitment: final-release-packaging

Status: Agreed 2026-09-14
Requirements: FRP-001, FRP-002, FRP-003, FRP-004

## Deliverable

Rebuild the release package graph and archives from the fully remediated tree,
align all release metadata, and produce a source-grounded release decision from
fresh cross-platform checks and an adversarial audit.

## Boundaries

This commitment begins only after the seven remediation commitments before it
are complete. It changes final manifests, package contents, release metadata,
archive checks, and release-candidate evidence. It does not publish a crate,
create a Git tag, or create a hosted release; those external actions require a
reviewed candidate.

Done-when: FRP-001 through FRP-004 pass; every archive builds from its unpacked
contents and stays below the registry limit; metadata agrees across release
surfaces; all supported platforms and Cairn mechanisms pass at the candidate
commit; a fresh audit has no unresolved Blocker or High finding.
