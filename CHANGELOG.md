# Changelog

This file records user-visible changes to Reactive TUI. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- **Breaking:** the number in a padding, margin, gap or space class is a
  count of cells, as in the width and height classes: `p-1` pads one cell
  and `gap-2` leaves two. These classes counted in fours before: `p-1` was
  four cells and `gap-4` sixteen. To keep the size of an existing class,
  multiply its number by four. Whole numbers from 0 to 512 are accepted,
  and a number with a fraction counts as the next whole number.
  The gap argument of `builder::flex_row`, `flex_col` and `grid_layout` is
  the number of such a class and counts in cells too. So do the declarative
  grid's `gap`, `column_gap` and `row_gap`, which its documentation already
  gave in cells: `gap(1)` was four cells, and the default gaps were none
  across and four down where they are now one and one.
- **Breaking:** `grid-cols-auto-fit-N`, `grid-cols-auto-fill-N` and their
  row forms make as many tracks of at least N cells as the container holds.
  They made N tracks before. `col-span-full` and `row-span-full` span the
  grid they are in; they spanned twelve tracks before.
- Fixed gaps between neighbouring boxes that were painted one cell too wide
  or too narrow when their container started at a fraction of a cell, for
  example beside a box of a third of the screen. A gap is painted with
  exactly its number of cells at every width.
- Fixed `gap-x-N`, `gap-y-N`, `space-x-N` and `space-y-N` setting the gap
  of the other direction to zero. Added `StyleBuilder::gap_x_px`,
  `gap_y_px`, `col_span_full` and `row_span_full`.
- Fixed the data table growing taller on every layout while its filter and
  column panels were open.
- Changed the license from the MIT License to The Reactive TUI License,
  Version 1.0 (`LICENSE`, SPDX `LicenseRef-ReactiveTUI-1.0`): the MIT License
  with two riders. Offering Reactive TUI or a copy of it to developers as a
  framework needs a Framework License, and an organization that earns money
  from software built with Reactive TUI and has revenue above USD 1,000,000
  a year must sponsor the project. Versions published under the MIT License
  alone stay under it.

## [1.0.0] - Unreleased

- Vendored the companion crates under `crates/` as path-only workspace
  members and marked them `publish = false`: no companion publishes.
- Replaced syntect 5.3.0 with lumis 0.13.1 for syntax highlighting;
  bincode 1.3.3 and yaml-rust 0.4.5 leave the dependency graph.
- Added `Ref::update_atomic` for linearizable read-modify-write while
  keeping the spec'd reentrant snapshot semantics of `Ref::update`.
- Mapped named forms to Landmark and unnamed forms to Panel, resolved
  hyperlink start/end offsets through the text model, and pinned
  focused/selected states with adapter tests; screen-reader output
  verified against Orca speech.

## [0.1.0] - 2026-09-14

This is the first supported, non-yanked release. Earlier 0.0.x packages were
development snapshots and are not a compatibility baseline.

### Added

- Retained applications, keyed components, reactive signals, hooks, and
  coalesced wake notifications.
- CSS-like flex and grid layout with cell-based painting and Unicode grapheme
  handling.
- Retained controls for text input, tables, trees, menus, dialogs, tabs,
  accordions, wizards, charts, images, and terminal views.
- A SuprTUI renderer with bounded frames, terminal restoration, host input,
  image composition, and platform-specific terminal support.
- Optional embedded Unix PTY sessions interpreted by `libghostty-vt`.
- Linux AccessKit and AT-SPI accessibility support.
- An optional C ABI with generated C and TypeScript consumer bindings.
- A source-grounded framework manual under `manual/`.

### Changed

- Vendored the maintained Crossterm and SuprTUI implementations as in-repo
  companion crates under `crates/` (never published) while preserving the
  `crossterm` and `suprtui` Rust import names inside Reactive TUI.
- Replaced the pinned `libghostty-vt` git dependency with crates.io version
  0.2.1.
- Limited crate archives to public source, legal notices, the manual, and
  release documentation.
- Declared Rust 1.91 as the minimum supported Rust version.

### Platform limits

- `embedded-terminal` is available on Unix and requires Zig 0.15.2, as required
  by `libghostty-vt` 0.2.1.
- `simd` requires nightly Rust because it uses `portable_simd`.
- Linux screen-reader integration requires an AT-SPI desktop session.
- Windows PTY support uses the pinned Microsoft OpenConsole runtime included in
  the crate source.

[Unreleased]: https://github.com/eas4ai/reactive-tui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/eas4ai/reactive-tui/releases/tag/v0.1.0
