# Release 0.1.0 through versioned companion crates

Level: Consequential
Decided by: agent
Rests on: developer release direction on 2026-09-14; crates.io ownership and name audit; Cargo package dependency rules
Would be wrong if: The registry packages cannot preserve the current Rust import names, the released archive cannot build without repository paths, or the developer chooses a different first stable version

## Decision

Prepare Reactive TUI 0.1.0 as the first non-yanked public release. Publish the macro crate and the two maintained backend forks under reactive-tui-prefixed package names, while keeping their Rust library names and root dependency aliases unchanged. Replace the pinned libghostty git dependency with crates.io version 0.2.1. Publish companion crates before the root crate.

## Realized by

- b9f7bc64 release: prepare crates.io 0.1.0 packages
