# Existing strict Clippy failures observed during wgpu work

Surfaced from: GPU-004
Captured: 2026-09-16T19:58:21.319Z

Rust 1.91 strict Clippy fails in unchanged vendored Crossterm on clippy::clippy::unnecessary_wraps and io_other_error. With --no-deps it also reports unchanged wizard.rs:291 collapsible_else_if. Do not call the full strict lint clean; recheck under inherited code-quality or release work. The graphics manual_is_multiple_of finding is repaired in GPU-004.
