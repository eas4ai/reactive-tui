# Framework manual review

commit: 99d5d3aa312d8d0b3c94a5437e31618e01f8ebcb
findings:
  - resolved: MAN-002 now limits type-specific pointer tracking to surface, renderer, and terminal routes and gives other handle families explicit create/destroy ownership rules.
Status: Complete

## Scope and coverage

Compared all 19 focused pages with the 27 top-level public modules exported by
`src/lib.rs`, the six widget families exported by `src/widgets/mod.rs`, and the
seven Cargo features. The manual check derives those three inventories from the
current source. It reports complete coverage and resolves all relative source,
test, chapter, and overview links.

Reviewed the application and component lifecycle against `src/app.rs` and
`src/component`; builders and the virtual DOM against their module exports;
reactive scheduling against signal, hook, scheduler, and wake code; widgets
against their public module roots and API behavior tests; rendering against the
backend, paint, render-tree, and SuprTUI paths; and optional terminal,
accessibility, and FFI behavior against their platform gates and implementations.

## Failure demonstration and corrected case

Before the committed evidence run, the actual mechanism was given an overview
whose Getting started jump targeted a missing heading. It exited 1 with
`broken heading link: #missing-system`. Restoring the real target made the same
mechanism exit 0 with 19 pages, 27 modules, six widget families, and seven Cargo
features. The committed MAN-001 and MAN-002 receipts record the passing case.

The two Rust examples in the manual were copied into a temporary path-dependent
consumer crate. `cargo check --jobs 8 --bins` compiled both examples against the
committed framework source.

## Finding resolution

The first review found that the FFI chapter generalized pointer tracking beyond
the implemented trackers. Commit `da8cef81` narrowed the statement: handle-based
entry points check null pointers; surface, renderer, and terminal routes use the
type-specific trackers visible in `src/ffi/pointer.rs`; other handle families
depend on their matching create and destroy operations. Fresh MAN-001 and
MAN-002 evidence at `99d5d3aa` covers the corrected page and the refreshed
GitNexus guidance. No source contradiction remains.

## Limits

The mechanism proves structural coverage and link integrity. Human review is
still required for behavioral meaning and simple language. GitNexus concept
queries were partial because two full-text indexes returned query errors, so
the review used the current codebase-memory graph and direct cited source for
the affected areas. No Rust source, API, dependency, feature, example, or test
changed in this commitment.

Ripwire test-gate reports no changed or impacted code symbols and no test
obligation. Quality-delta reports no gating regression; its one new-symbol row
comes from ignored `.gitnexus/meta.json`, which is generated index metadata and
is outside the committed manual diff. The worktree is clean apart from the
ignored Cairn in-progress record used for this review.

The final index refresh changed only the generated symbol and relationship
counts in `AGENTS.md`. It did not change manual content, source, tests, features,
or dependencies. The new evidence receipts pass against that indexed candidate.
