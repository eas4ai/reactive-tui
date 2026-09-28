# Changelog

This file records user-visible changes to Reactive TUI. The project follows
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
