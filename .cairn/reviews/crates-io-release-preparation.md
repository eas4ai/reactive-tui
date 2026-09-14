# crates.io release preparation review

commit: pending
findings:
  - pending: final release review must assess the separate pre-release audit backlog before publication.
Status: In progress

## CRT-003 mechanism review

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
