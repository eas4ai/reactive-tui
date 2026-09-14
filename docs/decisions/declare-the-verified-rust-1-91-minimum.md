# Declare the verified Rust 1.91 minimum

Level: Judged
Decided by: agent
Rests on: CRT-003; installed Rust toolchains; two failed Rust 1.90 standard-library downloads on 2026-09-14
Would be wrong if: The 1.91 toolchain cannot build every release configuration or a lower toolchain is later verified without changing the package graph

## Decision

Declare Rust 1.91 as the minimum supported version for the 0.1.0 crate set. Verify every stable release configuration with that installed toolchain. This avoids advertising Rust 1.90 without a completed build while remaining close to libghostty-vt's own Rust 1.90 floor.

## Realized by

- b9f7bc64 release: prepare crates.io 0.1.0 packages

## Realized by

(none yet: recorded, not built)
