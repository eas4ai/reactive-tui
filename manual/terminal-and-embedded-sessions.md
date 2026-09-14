# Terminal and embedded sessions

Crate modules: `terminal`, `embedded`, `escape`

## Purpose

Terminal modules interpret terminal data and maintain virtual terminal state.
Embedded sessions run a child process and expose that state to an application.

## Main API

- `Terminal`, `TerminalConfig`, `TerminalModes`, `TerminalStyle`, and
  `TerminalEvent` model a virtual terminal.
- `AnsiParser`, escape parsers, `TerminalCell`, `TerminalCursor`, and
  `VirtualScreen` process output and store the screen.
- `PseudoTerminal` owns a platform PTY for the legacy terminal path.
- With `embedded-terminal` on Unix, `EmbeddedSession` owns a child PTY and a
  libghostty interpreter.
- `SessionSnapshot` provides stable text and cell state.
- `TerminalView` renders an embedded session as an element.

## Basic use

For the optional embedded path, enable `embedded-terminal`, create a
`std::process::Command`, and pass it with cell dimensions to
`EmbeddedSession::spawn`. Attach the app waker, send key events, resize with the
view, read snapshots when needed, and call `shutdown` during cleanup.

## Behavior

PTY output is parsed into a virtual screen. ANSI and escape sequences update
text, cursor, modes, title, colors, scrolling regions, and alternate-screen
state. A session snapshot copies the current stable state for rendering.

The embedded worker owns the parser and child process. Commands cross a channel
to the worker. Output and process changes wake the application. Shutdown closes
the worker, PTY, and child process and can be called more than once safely.

## Limits

- The libghostty `embedded` module is available only with the feature on Unix.
- The legacy `terminal::PseudoTerminal` has Unix and Windows implementations;
  the Windows implementation uses the bundled ConPTY runtime.
- A PTY uses terminal cells, not pixels.
- A caller must forward input and resize events; the child cannot infer widget
  focus or layout.
- Process launch, I/O, parsing, and shutdown can each return errors.

## Source map

- Terminal exports: [`src/terminal/mod.rs`](../src/terminal/mod.rs)
- Escape parser exports: [`src/escape/mod.rs`](../src/escape/mod.rs)
- Embedded session: [`src/embedded/mod.rs`](../src/embedded/mod.rs)
- Platform PTY: [`src/terminal/pty.rs`](../src/terminal/pty.rs)
- Embedded-session acceptance tests: [`tests/embedded_terminal.rs`](../tests/embedded_terminal.rs)
- Terminal screen tests: [`src/terminal/screen/tests.rs`](../src/terminal/screen/tests.rs)

## Related chapters

- [Terminal widget](terminal-widget.md)
- [Rendering and backends](rendering-and-backends.md)
- [Events, focus, and input](events-focus-and-input.md)

[Back to the manual](README.md)
