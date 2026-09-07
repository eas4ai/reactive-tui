# Embedded terminal sessions

The `embedded-terminal` feature runs a requested executable on a real Unix
PTY, interprets its output with libghostty, and displays its screen through
App and the locally owned renderer. It is available from this checkout.

## Run a shell

```sh
cargo run --locked --features embedded-terminal --example embedded_shell
# Or choose an executable and its arguments:
cargo run --locked --features embedded-terminal --example embedded_shell -- /bin/sh -i
```

Ctrl+Q leaves the host application. Ctrl+C, Escape, text, Enter, navigation,
and function keys go to the child. The example exits when its direct child
exits. It restores the host terminal on normal exit and on application errors.

The feature requires Unix, Rust 1.90 or newer as required by the pinned
bindings, and Zig 0.16.x on PATH. Verification used Rust 1.95.0 and Zig 0.16.0.
The first native build fetches pinned Ghostty source and Zig packages.
`libghostty-vt` is pinned to `5988a0b78b4aa804d1c12e66bbfe662bd97d81c0`;
its build script pins Ghostty to `22d13172cde98a0a4dda05d3d6a3fcb0dd8ed018`.
Do not use a different `GHOSTTY_SOURCE_DIR` when reproducing these checks.

## Use the session

Build a `std::process::Command` with the executable, arguments, environment,
and working directory. `EmbeddedSession::spawn(command, columns, rows)`
attaches its standard streams to a new controlling PTY and reports startup
errors. Set `TERM=xterm-256color` for the supported text-terminal path, as the
example does.

`send_key` queues one key for libghostty's mode-aware keyboard encoder.
`resize` updates the child PTY and interpreter, then publishes the new snapshot
before returning. A zero dimension retains the last valid size.

`snapshot` returns an immutable `Arc<CellFrame>`, a change revision, child exit
status, and any worker/IO error. A nonzero exit code is a process outcome, not
an IO error. `shutdown` and Drop stop the worker and reap the direct child.
Shutdown also terminates a foreground job whose process group is verified to
belong to the owned terminal session. This is not a general supervisor for
arbitrary daemonized descendants.

Wrap a session in `TerminalView` and pass it to `App::builder().root(...)`.
Use `.quit_key(KeyCode::Char('q'), KeyModifiers { ctrl: true,
..KeyModifiers::empty() })` to reserve Ctrl+Q. Ordinary App roots keep their
existing Ctrl+C/Escape quit behavior.

## Reactive App integration

Roots can implement the following defaulted hooks without changing existing
components:

- `update`: return `Redraw` when background state changes, `Exit` to leave,
  or `Unchanged` to wait. App checks this once per event-loop iteration.
- `try_handle_event`: return input errors to App; its default calls the
  existing `handle_event` hook.
- `resize`: receive initial and subsequent nonzero viewport dimensions.
- `cell_frame`: optionally provide a complete owned cell screen. Otherwise
  App renders the existing Element tree.

`SuprTuiBackend` supports both Element and cell frames. Unsupported backends
return an explicit error for cell frames. A frame validates dimensions,
printable graphemes, cursor bounds, and wide-cell occupancy before sharing it.
It preserves RGB colors, the eight basic style flags, overline, five underline
styles, and underline color. The renderer resets cell styles before painting
changed cells, so a styled cell cannot affect its following plain cell.

## Ownership and limits

One worker owns libghostty and the child PTY. Input uses a 16-command queue and
at most 64 KiB of pending encoded input and terminal replies. Queue saturation
returns backpressure; pending-byte overflow becomes an observable session
error. Reads process at most 64 KiB per worker turn. The master is nonblocking,
and its wait interval is at most 10 ms, so an idle child cannot block shutdown.

The session retains one latest frame. Frames have at most 262,144 cells and
1,024 UTF-8 bytes per grapheme; callers that retain old snapshots own that
additional storage. Native scrollback is limited to 2,000 lines / 1 MiB with
libghostty's documented page-granularity allowance, and APC buffering to
64 KiB. These are component limits, not a total process-memory guarantee.
Grapheme-cluster mode is enabled by default and after reset. Title reporting
stays disabled; Glyph Protocol handling is disabled for this text-only path.

This commitment covers one full-pane terminal screen. Widget-tree placement,
images, mouse forwarding, clipboard, scrollback UI, and daemon supervision
remain separate work. Existing legacy terminal APIs remain available.

## Verification

```sh
sh scripts/check-embedded-terminal.sh
```

The gate runs native keyboard/buffering tests, real-child integration tests,
the actual example in a controlling host PTY, and the inherited renderer gate.
The PTY probe verifies delayed redraw without input, child size changes,
Ctrl+C/Ctrl+Q routing, exact termios restoration, direct-child cleanup, and
restoration after a forced resource error.

A separate vt100 parser checks captured host colors, basic styles, combining
characters, wide cells, alternate-screen transitions, cursor state, and erasure.
That parser does not implement joined-emoji widths; joined clusters are checked
against the native snapshot, and renderer tests check their exact emitted bytes.
These automated probes do not claim direct testing in Kitty, GNOME Terminal,
or Ghostty. The previously reported host-specific counter-input issue remains
unresolved and distinct from these passing probes.
