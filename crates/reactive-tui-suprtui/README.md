# reactive-tui-suprtui

Reactive TUI's cell renderer and terminal output engine. The package preserves
the Rust library name `suprtui` for the main framework crate.

The engine owns frame comparison, ANSI output, terminal lifecycle, cell layout,
Unicode segmentation, image composition, links, clipboard operations, and
embedded terminal surfaces. It is published separately so `reactive-tui` can
resolve entirely from crates.io.

The imported SuprTUI commit and OpenTUI origin are recorded in `UPSTREAM.md`.
Their retained notice is in `LICENSE-OpenTUI`.
