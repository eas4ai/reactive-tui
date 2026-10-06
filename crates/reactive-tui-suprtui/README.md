# reactive-tui-suprtui

Reactive TUI's cell renderer and terminal output engine. The package preserves
the Rust library name `suprtui` for the main framework crate.

The engine owns frame comparison, ANSI output, terminal lifecycle, cell layout,
Unicode segmentation, image composition and links. It is a vendored
companion of `reactive-tui`, used as a path dependency inside this
workspace, and is never published: its manifest sets `publish = false`.

The imported SuprTUI commit and OpenTUI origin are recorded in `UPSTREAM.md`.
Their retained notice is in `LICENSE-OpenTUI`.
