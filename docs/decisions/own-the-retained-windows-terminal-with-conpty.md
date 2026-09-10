# Own the retained Windows terminal with ConPTY

Superseded by: use-the-pinned-microsoft-conpty-runtime-for-windows-terminal-sessions

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-019
Would be wrong if: Windows still uses redirected pipes as a terminal, resize does not reach the child, shutdown blocks on full output, or native tests cannot verify process and handle cleanup.
History: Earlier API reversals exposed native timing assumptions and incomplete dependency behavior. This remains Judged because it restores the promised Windows PTY without changing the API or platform contract; native failure and recovery tests are required before acceptance.

## Decision

Replace the Windows pipe stub with the operating system ConPTY API through the existing windows-sys dependency. Use no cursor-inheritance flag. Keep input and output on separate owned threads with bounded queues; drain output during pseudoconsole closure and join every worker. Own the directly launched process handle, preserve environment and working-directory configuration, report real exit codes and errors, and apply resize through ResizePseudoConsole. Validate Windows coordinate limits before creation. Test with a native console child that queries its console dimensions, reads input and exits, plus full-output shutdown and failed-launch cleanup. Cross-compilation is an editing check only; record native Windows execution before acceptance.

## Realized by

The initial operating-system ConPTY implementation was built in
`src/terminal/pty/windows.rs` and tested by the native
`tests/api_widget_behavior/conpty_probe.rs` fixture. Native handle-count checks
found a leak in the operating-system runtime, reproduced without Reactive-TUI.
The replacement and comparative evidence are recorded in
`use-the-pinned-microsoft-conpty-runtime-for-windows-terminal-sessions.md` and
`.cairn/reviews/rust-api-remediation.md`. The current source uses that superseding
runtime decision; this original runtime choice is not the accepted one.
