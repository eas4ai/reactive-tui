# Organize the manual by public framework systems

Level: Judged
Decided by: agent
Rests on: developer direction on 2026-09-14; src/lib.rs public modules; Cargo.toml features; public API tests
Would be wrong if: A public system cannot be assigned to a clear chapter without hiding its behavior or repeating most of another chapter

## Decision

Create manual/README.md as the landing page. Its table of contents links to a short section for each public framework system, and each section links to a focused manual page. Every focused page names the public API, behavior, limits, implementation source, and confirming tests in simple technical English.

## Realized by

- f726ed23 docs: add source-grounded framework manual
