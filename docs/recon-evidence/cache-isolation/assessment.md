# Full-suite assessment after registry cache isolation

Date: 2026-09-08
Toolchain: rustc 1.95.0 (59807616e 2026-04-14); cargo 1.95.0 (f2d3ce0bd 2026-03-21)
Base before the cache repair: `019e583f9f1d9b1a35efb569b8ca414750680548`
Tested `src/component/registry.rs` SHA-256: `f3848807d52437977d23f30697141e71ba479e0c42be724781a677b3ebd90e52`

This assessment used the corrected working tree. The code and these results
are committed together. The cache-specific tests pass; the project-wide
results below are observations, not a claim that the legacy suite is clean.
Exact commands, exit codes and durations are in [runs.json](runs.json).

## Current results

| Check | Result | Evidence |
| --- | --- | --- |
| Default-feature full suite, including doctests | Exit 101: 1005 passed, 6 failed, 35 ignored | [Complete output](default-tests-local-tmp.txt) |
| FFI test compilation | Exit 101: minimal FFI target has unresolved linker symbols | [Output](ffi-compile.txt) |
| Focused historical FFI API test compilation | Exit 101: 36 missing-function errors | [Output](ffi-api-compile.txt) |
| Formatting across all workspace packages | Exit 1: differences in 154 files | [Diff output](format.txt) |
| Clippy across all targets with warnings denied | Exit 101: library reports 47 errors; library tests report 115, with overlap | [Output](clippy.txt) |

The strict Clippy compilation stops on errors, so these counts are not an
exhaustive list for every target. The changed registry source and its new test
pass focused formatting. Targeted Clippy completes with existing library
warnings and no diagnostic in either changed path.

## Remaining default-suite failures

1. `animation::tests::test_easing_function_steps_jump_start`: returns 0.25
   where the test expects 0.0. Establish the intended step semantics before
   changing the implementation or assertion.
2. `test_z_index_with_other_utilities`: combined utility classes return
   z-index 10 where the explicit z-50 utility is expected to preserve 50.
3. Four accordion doctests fail to compile: `accordion`, `settings_accordion`,
   `faq_accordion`, and `navigation_accordion`. They omit `Element` imports;
   the first also omits `AccordionMode`.

These are the same six product/test failures recorded in the initial baseline.
The registry concurrency and new lookup regressions pass in the fresh suite.
Thirty-five ignored tests remain outside its passing count.

## Bindings and maintenance

`tests/test_minimal_ffi.rs` fails to link references including
`rtui_element_builder_create`, `rtui_element_builder_add_class`,
`rtui_element_builder_build`, and `rtui_element_destroy`. This output establishes
an integration failure; it does not establish whether exports are absent or
the test fails to link the library correctly.

`tests/ffi_tests.rs` still refers to missing Rust-visible functions such as
`rtui_terminal_create`, `rtui_terminal_destroy`, `rtui_terminal_sync`, and
`rtui_terminal_poll_event`. FFI runtime and TypeScript behavior were not tested.

Formatting debt is broad. Clippy includes suspicious boolean expressions,
constant assertions, unnecessary branches, default/unit initialization and
documentation/style issues. Review behavioral warnings before applying broad
mechanical fixes. The historical 49-error Clippy figure used a narrower command
and must not be compared directly with this all-targets run.

## Environmental failures distinguished from product failures

The first default-suite run hit the system temporary-directory quota during
19 doctest links, yielding 23 doctest failures rather than four. Its output is
preserved in [the initial run](default-tests.txt). The complete suite was rerun
with `TMPDIR` pointing to a private temporary directory on the workspace disk;
that directory was removed afterward. The rerun has no quota/linker failures
and supplies the aggregate above. No test assertions or ignore flags changed.

An inherited embedded-feature build also hit Rust 1.95's internal
`uninterned StableCrateId` error; [its output is retained](inherited-build-ice.txt).
After removing only this crate's generated incremental-cache directories,
the unchanged source passed the full inherited App/embedded/renderer gate.
This records the recovery, not a claim to have repaired the compiler.

## Suggested next commitments

Repair the four broken documentation examples and the z-index regression,
then resolve JumpStart semantics with an explicit test contract. Follow with
a focused FFI contract/linkage audit and staged formatting/Clippy cleanup.
The developer-host counter-input report and platform support remain separate;
passing PTY tests do not close those reports.
