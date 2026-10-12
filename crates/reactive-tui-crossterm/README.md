# reactive-tui-crossterm

This package is Reactive TUI's maintained fork of Crossterm 0.29. It preserves
the Rust library name `crossterm`, so Reactive TUI source and downstream type
names remain unchanged.

The fork repairs retained Unix input readiness. The exact upstream archive and
the local behavior change are recorded in `REACTIVE_TUI_PATCH.md`. The original
MIT license is retained in `LICENSE`.

Applications should depend on
[`reactive-tui`](https://crates.io/crates/reactive-tui) instead of using this
implementation package directly.
