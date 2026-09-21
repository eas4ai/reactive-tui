# Commitment: pre-release-dependency-code-quality

Status: Agreed 2026-09-14
Requirements: DQC-001, DQC-002, DQC-003, DQC-004, DQC-005

## Deliverable

Repair the production dependency graph, enforce dependency policy, keep
library diagnostics out of terminal output, remove or connect dead code and
tests, and document the maintained public API.

## Boundaries

This commitment changes dependency manifests and lockfiles, `deny.toml`,
diagnostic calls, dead or disconnected modules and tests, public-item
documentation, CI inputs needed by these checks, and Cairn mechanisms. It does
not build final release archives.

Done-when: DQC-001 through DQC-005 pass; audit and policy checks are clean or
have developer-approved exceptions; mutation demonstrates sampled tests fail
when behavior breaks; final review finds no raw library print path or
unexplained shipped dead surface.
