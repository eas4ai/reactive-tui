# crates.io release preparation review

commit: pending
findings:
  - pending: final release review must assess the separate pre-release audit backlog before publication.
Status: In progress

## CRT-001 mechanism review

Compared `scripts/check-crates-release.py` with CRT-001 and its falsifier. The
mechanism reads all six release manifests, rejects git dependencies, requires
exact versions on local path dependencies, validates the root and Ghostty
dependency aliases, and prints the dependency-safe publication order.

A detached worktree changed the root Ghostty source from its local path to
`https://example.invalid/libghostty-vt`. The mechanism exited 1 with
`target.cfg(unix).dependencies.libghostty-vt still uses git`. The committed
corrected case passed and reported all six packages in dependency order.

The first review found one mismatch: the mechanism checked package names and
dependency aliases but did not verify each companion crate's Rust library name.
Commit `7ec85016` added that assertion. In a new detached worktree, changing
the safe Ghostty wrapper's library name to `wrong_library_name` made the
mechanism exit 1 and require `libghostty_vt`. The committed corrected case
passed again. No requirement or falsifier mismatch remains.

## CRT-003 mechanism review

The revised package graph added two Ghostty companion package commands and a
Linux CPU-affinity wrapper. Every command remains a separate foreground process,
and `taskset` applies the selected one-to-eight logical CPUs to Cargo and all
of its children on Linux. Both direct and `mise`-provided Zig commands pass
through that wrapper.

This review found two mismatches in the new package commands. The sys package
invokes its Zig build during Cargo's archive verification, but its command does
not select Zig 0.15.2. The safe wrapper's normalized archive depends on the
unpublished sys package and therefore cannot be verified from crates.io before
the staged publication order begins. The sys package command must use
`zig_0_15_2`; the wrapper must package without archive verification while the
later embedded root build compiles the complete local wrapper and sys pair.
These mismatches must be fixed before accepting the revised digest.

The first attempted fix selected Zig 0.15.2 for the sys package. A targeted
package verification ran under CPUs 0 through 7 and reached the pinned Ghostty
build, which rejected Zig 0.15.2 and required Zig 0.16.0. The 0.15.2 constraint
came from the incompatible crates.io package and does not describe upstream
commit `5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`. CRT-003 and its mechanism
must use Zig 0.16.0 before this review can pass.

After the Zig correction, the sys package verified successfully under CPUs 0
through 7. Packaging the safe wrapper by itself then failed before archive
creation because its renamed sys dependency is not published yet; Cargo applies
this registry check even with `--no-verify`. A committed detached experiment
made all six crates workspace members and ran `cargo package --workspace
--no-verify`. Cargo assembled all six archives successfully in one command,
including the interdependent unpublished packages. The root archive was 2.7 MiB
compressed. The mechanism should use that supported workspace operation after
the feature build matrix, which already compiles every local companion crate.

Compared `scripts/check-crates-release-build.sh` with the revised CRT-003
requirement and falsifier. The script validates a numeric job count from one
through eight before invoking Cargo. Every package and feature command is a
separate foreground command. The shell exits on the first failure, so builds
cannot overlap or hide an earlier failure.

The stable feature checks use the declared Rust 1.91 toolchain. The embedded
terminal and all-feature checks select Zig 0.15.2 either from `PATH` or through
the installed `mise` tool. The nightly all-feature command includes SIMD,
embedded terminal support, FFI, and every default feature. The final root
package command omits verification because unpublished companion packages
cannot resolve from crates.io until staged publication; the earlier source
checks compile the same root inputs and the contract check validates the
normalized registry dependencies.

The first full mechanism run used system Zig 0.16.0. `libghostty-vt-sys`
rejected it with exit 101 and named its required Zig 0.15.2 version. This is the
safe violating case. After installing the exact toolchain, `mise exec
zig@0.15.2 -- zig version` returned `0.15.2`. Setting `CARGO_BUILD_JOBS=9`
made the revised mechanism exit 1 before any Cargo command. These two focused
checks confirm the corrected tool selection and job guard; the next committed
CRT-003 run will verify the complete corrected build.

No requirement or falsifier mismatch was found in the revised mechanism.
