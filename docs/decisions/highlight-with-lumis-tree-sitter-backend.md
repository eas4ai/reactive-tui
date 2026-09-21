# highlight with lumis tree-sitter backend

Level: Judged
Decided by: Shawn
Rests on: DQC-002
Would be wrong if: Lumis leaves the dependency graph, a flagged RUSTSEC advisory returns unexcepted, or fenced-code highlighting regresses to unstyled output for exact language names.

## Decision

Replace syntect 5.3.0 with lumis 0.13.1 (MIT, tree-sitter grammars compiled
in via `lang-*` Cargo features, Neovim themes). This removes bincode 1.3.3
(RUSTSEC-2025-0141) and yaml-rust 0.4.5 (RUSTSEC-2024-0320) from the locked
graph, so their `dependency-maintenance.toml` exceptions are dropped rather
than re-verdict. Only 15 grammars are compiled in (rust, python, javascript,
typescript, bash, c, cpp, go, java, json, toml, yaml, html, css, markdown),
not `all-languages`. Language lookup stays exact and case-sensitive, so
unknown fence names keep falling back to plain code rendering. Sublime
`.tmTheme`/plist theme loading is gone; custom themes load from Lumis JSON
via `themes::from_file`.

## Realized by

- syntect-to-lumis swap: `src/syntax/{resources,highlighter,theme}.rs`,
  `Cargo.toml`, `Cargo.lock`, `deny.toml`, `dependency-maintenance.toml`,
  `scripts/check-api-residual.py`, `docs/spec/rust-api-remediation.md`
