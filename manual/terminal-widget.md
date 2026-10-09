# Terminal widget

Crate modules: `widgets`

Widget module: `terminal`

## Purpose

The terminal widget displays and controls a child process inside a Reactive TUI
application.

## Main API

- `TerminalProps` configures the command, environment, working directory,
  dimensions, scrollback, shell integration, and callbacks.
- `TerminalState` stores the virtual terminal, process state, focus, scroll,
  title, and last error.
- `TerminalWidget` starts and stops the process, sends bytes or text, resizes the
  PTY, scrolls, reports the title, and exposes running and error state.

## Basic use

Construct `TerminalProps`, create the widget, call `start`, and render it inside
an application component. Forward focused keyboard input with `send_input` or
`send_string`. Call `resize` when its presented cell dimensions change.

## Behavior

The widget starts a pseudo-terminal and child process. A worker reads terminal
output, feeds the ANSI parser, updates a virtual screen, and wakes the app.
Painting copies the visible virtual screen into the widget's assigned layout.

Stopping the widget shuts down the owned process and PTY. Focus controls cursor
presentation and whether application input should be forwarded to the child.

## Limits

- The child process and PTY are operating-system resources and must be stopped.
  Stopping ends a shell that is still printing within a second, on macOS as on
  Linux: what the child wrote and nobody read is read and discarded while it
  exits, output paused by a Ctrl-S stop character included, since macOS keeps
  an exiting process until its unread terminal output is gone (TRM-001).
- Output uses a bounded screen and configured scrollback.
- The legacy terminal widget and the optional `embedded` module are separate
  APIs with different parser and platform dependencies.
- Windows PTY behavior depends on the bundled ConPTY runtime path.

## Source map

- Widget implementation: [`src/widgets/terminal.rs`](../src/widgets/terminal.rs)
- PTY implementation: [`src/terminal/pty.rs`](../src/terminal/pty.rs)
- Terminal screen: [`src/terminal/screen.rs`](../src/terminal/screen.rs)
- Terminal widget tests: [`tests/api_widget_behavior/terminal.rs`](../tests/api_widget_behavior/terminal.rs)
- Embedded terminal integration tests: [`tests/embedded_terminal.rs`](../tests/embedded_terminal.rs)

## Related chapters

- [Terminal and embedded sessions](terminal-and-embedded-sessions.md)
- [Events, focus, and input](events-focus-and-input.md)
- [Rendering and backends](rendering-and-backends.md)

[Back to the manual](README.md)
