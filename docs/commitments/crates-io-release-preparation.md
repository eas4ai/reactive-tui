# Commitment: crates-io-release-preparation

Status: Agreed 2026-09-14
Requirements: CRT-001, CRT-002, CRT-003

## Deliverable

Prepare version 0.1.0 as the first non-yanked crates.io release. Give the two
maintained backend forks project-owned package names while preserving their
current Rust import names. Replace the git dependency with its registry release.
Bound every archive, add complete package metadata and legal files, write the
release changelog, and verify the release configurations.

## Boundaries

This commitment changes Cargo manifests, lockfiles, package README and license
files, the changelog, release checks, and Cairn records. It does not publish a
crate, create a Git tag or GitHub release, rewrite the root README, or change a
Rust API. Those external release actions follow the already requested root
README commitment.

Done-when: CRT-001 through CRT-003 pass, the checks reject an internal archive
file and an invalid dependency source, and final review finds no stale metadata,
unlicensed copied code, or unverified release configuration.
