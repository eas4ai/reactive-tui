# Framework manual review

commit: 4e1f579441f5a160651f2a0e16bca54a7d5e2d2f
findings:
  - open: MAN-002 overstates FFI pointer validation by saying FFI functions validate tracked pointers generally; source shows tracking for selected handle families and a compatibility helper that does not provide full type-specific tracking.
Status: Findings

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

## Finding

The FFI chapter says FFI functions validate null and tracked pointers. The
common panic boundary has broad use, but `src/ffi/pointer.rs` describes only
selected trackers and its compatibility `validate_pointer` helper does not
maintain full type-specific tracking. The manual must narrow this statement to
the handle families and entry points that actually use tracking. No other
source contradiction was found.

## Limits

The mechanism proves structural coverage and link integrity. Human review is
still required for behavioral meaning and simple language. GitNexus concept
queries were partial because two full-text indexes returned query errors, so
the review used the current codebase-memory graph and direct cited source for
the affected areas. No Rust source, API, dependency, feature, example, or test
changed in this commitment.
