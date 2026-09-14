# Commitment: framework-manual

Status: Agreed 2026-09-14
Requirements: MAN-001, MAN-002

## Deliverable

Create a manual landing page with a jump-linked system overview. Link every
overview section to a focused page. Cover the full public Rust module surface,
widget families, feature flags, platform behavior, and bindings in simple
technical English. Ground each page in implementation source and confirming
tests.

## Boundaries

This commitment changes the manual, its Cairn contract and check, the roadmap,
and generated source-index guidance in `AGENTS.md`. It does not change Rust
behavior, public APIs, Cargo metadata, examples, tests, or the root README.
Crates.io preparation and the root README are separate later commitments.

Done-when: MAN-001 and MAN-002 pass, the mechanism rejects broken navigation
and omitted source coverage, and final review finds no unsupported behavior
claims or unexplained public systems.
