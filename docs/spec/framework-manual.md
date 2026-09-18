# Framework manual

Status: Agreed 2026-09-14
Prefix: MAN

The developer chose a professional manual written in simple technical English.
The implementation, exported Rust API, Cargo features, examples, and tests are
the authority for its content.

[MAN-001]
The repository MUST provide `manual/README.md` as the manual landing page. Its
table of contents MUST jump to an overview section for every documented system.
Every overview section MUST explain the system briefly and link to one focused
manual page. Every focused page MUST describe its purpose, main public API,
behavior, limits, implementation source, confirming tests, and related systems.
Falsifier: An overview jump link is broken, an overview section has no focused
page, a focused page is unreachable from the landing page, a required section
is missing, or a relative manual link does not resolve.
Mechanism: `python3 scripts/check-framework-manual.py` validates the manual
structure, section links, page links, headings, and relative paths.

[MAN-002]
The manual MUST account for every public top-level module exported by
`src/lib.rs`, every public widget family exported by `src/widgets/mod.rs`, and
every Cargo feature. Each focused page MUST name the crate modules it covers and
link to tracked implementation source and confirming tests. Claims MUST follow
the current source and tests. Retained specifications and decisions may explain
intent or platform limits but MUST NOT override observed implementation.
Falsifier: A public module, widget family, or Cargo feature is absent; a page
names a module the crate does not export; a source or test link is missing or
untracked; or review finds a behavior claim contradicted by its cited source or
test.
Mechanism: `python3 scripts/check-framework-manual.py` derives the exported
module, widget, and feature sets from source and compares them with the manual's
visible coverage records.
