# Use libghostty with an owned PTY session and complete cell frames

Level: Judged
Decided by: Shawn and Codex
Rests on: EMB-001 EMB-002 EMB-003 EMB-005
Would be wrong if: The safe bindings cannot preserve terminal cells or the owned session cannot shut down with bounded buffering.

## Decision

Pin libghostty-vt from Uzaaft/libghostty-rs at 5988a0b78b4aa804d1c12e66bbfe662bd97d81c0 behind an embedded-terminal Cargo feature. Use a real PTY instead of the legacy pipe-based PseudoTerminal. Keep native terminal state on one owned worker and expose complete owned cell snapshots to SuprTUI. Add defaulted App support for background updates and an explicit quit chord so shell Ctrl+C and Escape reach the child. Start with one terminal screen composed into an application frame; arbitrary widget-tree embedding, images, mouse, and clipboard follow later.

## Realized by

(none yet: recorded, not built)
