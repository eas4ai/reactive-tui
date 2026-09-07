# Use libghostty with an owned PTY session and complete cell frames

Level: Judged
Decided by: Shawn and Codex
Rests on: EMB-001 EMB-002 EMB-003 EMB-005
Would be wrong if: The safe bindings cannot preserve terminal cells or the owned session cannot shut down with bounded buffering.

## Decision

Pin libghostty-vt from Uzaaft/libghostty-rs at 5988a0b78b4aa804d1c12e66bbfe662bd97d81c0 behind an embedded-terminal Cargo feature. Use a real PTY instead of the legacy pipe-based PseudoTerminal. Keep native terminal state on one owned worker and expose complete owned cell snapshots to SuprTUI. Add defaulted App support for background updates and an explicit quit chord so shell Ctrl+C and Escape reach the child. Start with one terminal screen composed into an application frame; arbitrary widget-tree embedding, images, mouse, and clipboard follow later.

## Realized by

(none yet: recorded, not built)

## Feasibility findings (2026-09-07)

A temporary optional dependency on this pin compiled with Zig 0.16.0 after
updating the locked bitflags from 2.9.3 to 2.11.0. The command was
`cargo check --features embedded-terminal --lib`; it passed with the existing
unused-assignment warning in src/markdown/converter.rs. This was a dependency
build probe, not acceptance evidence. The temporary manifest and lock changes
were restored afterward.

EMB-002 requires preserving text attributes. At the pinned Ghostty binding,
`crates/libghostty-vt/src/style.rs` exposes overline, underline color, and
multiple underline styles. SuprTUI at 7793deb80c5bceecc5d8ed9fc6bc6d2530531774
defines only eight base attribute bits in `src/ansi.rs:193`: bold, dim,
italic, underline, blink, inverse, hidden, and strikethrough. Its remaining
attribute bits store a link identifier. A direct cell adapter cannot represent
all Ghostty styles in that frame format. The owned-session implementation is
paused pending a decision about extending the renderer dependency or explicitly
narrowing the attribute requirement. No attribute requirement has been weakened.
