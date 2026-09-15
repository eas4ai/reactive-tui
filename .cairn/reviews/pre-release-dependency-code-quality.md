# Review: pre-release-dependency-code-quality

commit: d60e847d
findings:
  - resolved: GitNexus reports a CRITICAL aggregate blast radius because the commitment touches central terminal and panic paths; focused context showed the direct runtime edits only reroute diagnostics and preserve control flow.
  - resolved: the DQC-003 declaration does not inventory workspace crates outside `src/`; a supplemental Tilth scan found only rustdoc examples and Cargo build-script directives, not runtime library output.
  - resolved: rust-analyzer's workspace-symbol cache continued to list four deleted legacy helpers after reload; the review avoided that stale cache and confirmed current state with its file syntax tree, Tilth, Cargo, and rustdoc.

## Scope examined

Re-read DQC-001 through DQC-005, the commitment boundaries, all five mechanism
declarations, the latest passing receipts and captured output, the dependency
exception manifests and decisions, and the full structural change set from the
commitment's creation through `d60e847d`. I compared each mechanism with its
falsifier and looked for incomplete feature graphs, diagnostics hidden by syntax
matching, disconnected Rust sources, ABI declarations left behind by deleted
FFI modules, production test helpers, and undocumented retained API.

Tilth found ten remaining raw output macros below `src/`; every match is inside
a test module or a rustdoc example. A supplemental scan of `crates/` and the
proc-macro crate found no runtime library output. The only executable matches
outside `src/` are `cargo:` directives in a native build script, which are
the Cargo protocol rather than application diagnostics. The captured DQC-003
probe independently exercised parser, terminal, reconciliation, focus, and
window paths without process output.

## Dependency and policy attacks

DQC-001's validator rejects incomplete policy results, unmatched advisory
exceptions, and reachable `atty`; the committed run passed locked audit and
all four cargo-deny policy sections. DQC-002 inspects locked default, minimal,
FFI, and Markdown graphs with `cargo tree --target all`, so target-specific
normal and build dependencies are included. Its validator rejects ambiguous
fork identity, default `onig_sys`, unexplained maintenance advisories, and
unapproved taffy or vte duplicates. The latest graph records one vte version,
the two explicitly decided taffy lines, and no default oniguruma dependency.

These are static dependency results and do not prove every optional dependency
behaves correctly at runtime. That limit does not contradict DQC-001 or DQC-002:
the lockfile, feature, target, source, license, advisory, and decision surfaces
named by those requirements are all represented.

## Test, FFI, and public-surface attacks

DQC-004 compiles 80 integration targets across the maintained default and FFI
surfaces, traces nested and explicit `#[path]` modules, inventories every
reachable FFI export module, scans shipped unsafe hooks, and requires two
isolated assertion mutants to fail. I searched for representative names from
the four deleted FFI modules across the complete checkout; no Rust source,
header, wrapper, or documentation still declares them. This supports deletion
as removal of disconnected source rather than removal of a linked ABI.

DQC-005 ran strict rustdoc over default, minimal, FFI, docs.rs,
embedded-terminal, and combined stable feature surfaces and inventoried 6,558
shipped public declarations. Strict rustdoc proves documentation is present,
not that prose is useful, so I manually inspected the touched boundary:
`create_test_image` is gated with `#[cfg(test)]` and rust-analyzer reports
only test-module callers; the four `*_legacy` control helpers are absent; and
`test_sync_output_support` is a documented terminal capability query, not a
test fixture despite its name. No retention exception is needed.

## Graph and language-server review

After refreshing the stale GitNexus index to `d60e847d`, the commitment-wide
comparison reported 132 files, 504 indexed symbols, 29 affected flows, and
CRITICAL aggregate risk. Focused impact analysis attributes that rating chiefly
to `run_worker` and `resume_caught_panic`: one and two direct callers
respectively, with 15 affected flows each. Tilth inspection shows the edited
paths retain their signatures and control flow while replacing direct terminal
diagnostics with `log::warn!` or `log::error!`. The DQC-003 captured probe,
the broader behavioral test gates run during implementation, and the passing
mechanisms cover those direct dependants.

rust-analyzer loaded 282 packages with no configuration or validation errors.
Its syntax tree for `src/platform/ctlseqs.rs` contains none of the deleted
legacy names, despite stale results from workspace-symbol search. I therefore
treated workspace-symbol absence as non-authoritative and used the syntax tree,
Tilth source model, GitNexus refresh, Cargo builds, and rustdoc as independent
confirmation.

No unresolved in-scope finding was identified. No executable code changed
during this review.
