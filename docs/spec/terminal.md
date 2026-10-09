# Terminal and embedded sessions

Prefix: TRM

The terminal widget (src/widgets/terminal.rs) runs a shell on a
`PseudoTerminal` (src/terminal/pty/unix.rs), and the embedded session
(`EmbeddedSession`, src/embedded, feature `embedded-terminal`) runs one on
the same kind of child. Both own the child through `PtyChild`
(src/terminal/owned_pty.rs): a worker thread reads the child's output from
the pseudo-terminal's master side and writes the application's input to it;
when the worker is told to stop, or the terminal or session is dropped, the
worker stops reading and calls `PtyChild::stop`, which kills the child's
foreground group and the child and waits for the child to exit
(`Child::wait`, owned_pty.rs:143-151). Read on 2026-10-03: nothing reads the
master while `stop` waits. On Linux a killed child exits whatever is left
unread. On macOS a process that exits with output still unread on its
pseudo-terminal stays in exit until that output is read, so a child that was
printing faster than it was read never finishes exiting, `stop` never
returns, and whatever joins the worker waits with it: `PseudoTerminal::kill`
(unix.rs:178-182), the drop of a `PseudoTerminal` or of an
`EmbeddedSession`, and the terminal widget's unmount. Seen twice: the Mac run
of the INP-011 tests at a9231484 hung 20 minutes this way in the test
harness (fixed in the harness, ebd59238; backlog item 64ffce50), and on
2026-10-03 the macOS runner of the Supported-platform CI workflow ran
`terminal::pty::tests::api_terminal_pty_backpressure_and_drop_reap_a_flooding_child`
until the job's 45-minute limit (run 37151822540). Windows runs the child
on ConPTY (src/terminal/pty/windows.rs) and is not covered here.

## Observed

(none yet)

## Draft

[TRM-001] On Unix, stopping a pseudo-terminal child, through `PseudoTerminal::kill`, the drop of a `PseudoTerminal`, or the stop or drop of an `EmbeddedSession`, MUST end the child, reap it and return within 1 s on Linux and on macOS, also when the child has been writing faster than its output was read and nothing reads it any more.
Falsifier: On Linux or on the macOS test host, a shell that prints without pause is spawned on a `PseudoTerminal` whose output is read once and then no more; `kill`, or the drop, returns after 1 s or more, or does not return, or a process of the child still exists afterwards.
Mechanism: pty-stop
Rationale: `PtyChild::stop` kills and waits without reading the master, and macOS holds an exiting process until its unread pseudo-terminal output is read; the macOS CI runner hung the flooding-child test for its 45-minute limit on 2026-10-03, and the Mac test host hung the INP-011 run for 20 minutes the same way.
Status: Agreed 2026-10-03
