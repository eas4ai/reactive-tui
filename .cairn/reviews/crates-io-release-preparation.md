# crates.io release preparation review

commit: 95506db8
findings:
  - resolved: publication is outside this commitment and remains blocked by the separate pre-release audit backlog.
Status: Complete

## Final commitment review

Reviewed the committed 0.1.0 candidate after CRT-001 through CRT-003 passed.
The six generated archives contain no Cairn state, specifications, release
scripts, git dependency, or normalized path dependency. Each archive contains
its declared license and README. The root archive contains 477 files and is
2,779,508 bytes compressed. The five companion archives range from 6,646 to
183,965 bytes compressed.

Compared the copied Ghostty wrapper source, sys bindings, generator, and build
script with libghostty-rs commit
`5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`. There were no differences after
normalizing the eight trailing-space-only cleanups recorded by Git. Both
packages include the upstream MIT notice and name the source commit. The sys
build pins Ghostty commit `22d13172cde98a0a4dda05d3d6a3fcb0dd8ed018`.

Queried crates.io on 2026-09-14 for all six exact package names; none was
allocated. This is a point-in-time result and must be checked again immediately
before staged publication.

Cross-checked the separate 2026-09-14 pre-release audit. This commitment resolves
its package graph, archive size, and core Rust metadata findings. It does not
resolve the main-branch CI, public README, C ABI memory-safety, terminal panic
restoration, animation task ownership, or dependency advisory findings. The
repository must not be tagged or published until that backlog is completed and
re-audited.

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

Compared `scripts/check-crates-release-build.sh` with the revised CRT-003
requirement and falsifier. The script validates a numeric job count from one
through eight before invoking Cargo. Every feature and packaging command is a
separate foreground process. On Linux, `taskset` applies the selected logical
CPUs to Cargo, Zig, and their child processes. The shell exits on the first
failure, so builds cannot overlap or hide an earlier failure.

The stable feature checks use the declared Rust 1.91 toolchain. The embedded
terminal and all-feature checks select Zig 0.16.0, the version required by
pinned libghostty-rs commit
`5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`. The nightly all-feature command
includes SIMD, embedded terminal support, FFI, and every default feature. After
the build matrix compiles all local dependencies, Cargo packages all six
workspace members together with `--no-verify`. This allows Cargo to normalize
the unpublished interdependent packages without consulting crates.io for a
predecessor that has not been staged yet.

The review exercised three safe violating cases. Nine requested jobs exited 1
before Cargo ran. Zig 0.15.2 reached the pinned Ghostty build and was rejected
with its Zig 0.16.0 requirement. An incomplete workspace lock assembled five
archives and then stopped when Cargo made `Cargo.lock` dirty; the mechanism did
not bypass that failure with `--allow-dirty`.

With the corrections committed, the Ghostty sys package verified under Zig
0.16.0 on CPUs 0 through 7. The clean workspace packaging command assembled all
six archives, and the root archive was 2.7 MiB compressed. The static release
contract also passed with the same clean tree. The next committed CRT-003 check
will run the complete feature matrix.

No requirement or falsifier mismatch was found in the revised mechanism.
