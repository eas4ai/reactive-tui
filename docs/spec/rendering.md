# Application rendering

Status: Agreed 2026-09-07
Prefix: RND

The developer agreed to the first SuprTUI renderer milestone on 2026-09-07
following the proposed frame assertions and pseudo-terminal lifecycle proof
in `docs/renderer-foundation.md`. These blocks formalize that agreed scope.
The implementation target is the new SuprTUI backend; legacy backends remain
available. Modern host terminals receive ANSI truecolor cell output.

[RND-001]
The SuprTUI backend MUST present a complete styled layout from the current RootComponent output.
The SuprTUI renderer MUST erase removed content when the next frame replaces it.
The SuprTUI renderer MUST emit no frame bytes for an unchanged screen.
Falsifier: the captured terminal screen differs from the composed layout after an initial frame, update, removal, or unchanged render.
Mechanism: `scripts/check-renderer.sh`, RND-001 frame and layout assertions in `tests/suprtui_renderer.rs`.

[RND-002]
The SuprTUI paint path MUST preserve complete grapheme clusters and their occupied cell widths.
The painter MUST clip text without emitting half of a wide grapheme.
Falsifier: combining marks or a wide cluster are lost, a continuation cell is printed as a second glyph, or clipping splits a grapheme.
Mechanism: `scripts/check-renderer.sh`, RND-002 Unicode frame/output assertions in `tests/suprtui_renderer.rs`.

[RND-003]
The SuprTUI backend MUST recompute layout and fully repaint after a nonzero terminal resize.
The backend MUST tolerate zero-sized resize notifications without allocating an invalid frame.
Falsifier: the resized screen differs from a fresh frame at that size, old coordinates survive shrinking, or a zero-size notification panics.
Mechanism: `scripts/check-renderer.sh`, RND-003 frame assertions and `scripts/check-renderer-pty.py`.

[RND-004]
The SuprTUI output adapter MUST flush each published frame before reporting success.
The adapter MUST return write or flush errors to its caller.
The adapter MUST force a complete repaint after a failed output attempt.
Falsifier: successful presentation leaves bytes unflushed, a broken writer reports success, or retry leaves stale cells after a partial write.
Mechanism: `scripts/check-renderer.sh`, RND-004 controlled writers in `tests/suprtui_renderer.rs`.

[RND-005]
The terminal session MUST restore raw mode, cursor visibility, and the alternate screen on normal exit or an application error.
The session MUST attempt restoration during Rust unwinding.
Falsifier: the pseudo-terminal retains changed termios settings or captured output lacks restoration after exit, error, or panic.
Mechanism: `scripts/check-renderer.sh` and `scripts/check-renderer-pty.py` using an internal renderer acceptance probe.

[RND-006]
The renderer acceptance probe MUST display a state change after keyboard input through App.
The probe MUST exit cleanly on its documented quit keys.
Falsifier: the input probe cannot observe a changed counter, the process hangs on quit, or the probe fails to build.
Mechanism: `scripts/check-renderer.sh` and `scripts/check-renderer-pty.py` using the internal renderer probe.
