# Publish pinned Ghostty bindings as versioned companion crates

Level: Consequential
Decided by: agent
Supersedes: release-0-1-0-through-versioned-companion-crates
Cause: the stated condition occurred
Rests on: CRT-001, CRT-003, the verified API mismatch in crates.io libghostty-vt 0.2.1, and upstream commit 5988a0b
Would be wrong if: A source-compatible registry release appears before publication, the copied crates cannot preserve licensing and import names, or the archives cannot build inside crates.io limits
History: The prior decision replaced the pinned source with crates.io libghostty-vt 0.2.1. Its stated compatibility condition failed during CRT-003: the registry API omits the byte scrollback limit and other methods used by Reactive TUI.

## Decision

Prepare project-owned reactive-tui-libghostty-vt and reactive-tui-libghostty-vt-sys packages from the exact pinned upstream commit. Preserve the libghostty_vt and libghostty_vt_sys Rust library names, the current terminal API, and both scrollback safety limits. Use exact versioned path dependencies for local verification, publish the sys crate before the safe wrapper, and publish both before reactive-tui. Keep publication itself outside this commitment.

## Realized by

- 60659879 release: preserve pinned Ghostty bindings
