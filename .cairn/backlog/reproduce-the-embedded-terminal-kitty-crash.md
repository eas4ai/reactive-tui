# Reproduce the embedded-terminal Kitty crash

Surfaced from: EMB-006
Captured: 2026-09-15T19:23:04.928Z

The developer observed embedded_shell segfaulting in Kitty on 2026-09-15. The deterministic Unix PTY probe passes, so it does not reproduce or refute that host-terminal failure. Capture a Kitty crash trace and add a regression before claiming the embedded terminal works in that environment.
