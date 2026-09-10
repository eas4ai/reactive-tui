# Use the pinned Microsoft ConPTY runtime for Windows terminal sessions

Level: Consequential
Decided by: Codex
Supersedes: own-the-retained-windows-terminal-with-conpty
Cause: the stated condition occurred
Rests on: API-011 API-019
Would be wrong if: The official runtime still leaks handles, changes child input or resize behavior, or Windows packaging cannot provide a matched and verified runtime without hidden fallback.
History: The original direct Windows API choice failed native cleanup. A bare CreatePseudoConsole and ClosePseudoConsole loop on Windows Server 2022 leaks one handle per cycle without Reactive-TUI code. The same native comparison with Microsoft.Windows.Console.ConPTY 1.24.260710001 holds a constant handle count for six cycles. Earlier API reversals concerned native timing and incomplete dependency behavior; preserve the strict cleanup test and require native application verification of this replacement.

## Decision

Use Microsoft.Windows.Console.ConPTY 1.24.260710001 instead of the leaking operating-system pseudoconsole implementation. Keep the existing owned session, pipes, process and worker lifecycle and public Rust APIs. Vendor the official MIT-licensed runtime files with their package SHA-256 and source provenance, provide an offline packaging script, and load only a matched pinned bundle from the application-adjacent reactive-tui-conpty directory or an explicitly configured absolute directory. Verify file digests while holding read handles that deny replacement, use an absolute DLL path with only its directory and System32 in the DLL search, and unload the module after the last session using it ends. Report missing or damaged runtime files as actionable launch errors; do not fall back to the known leaking OS API. Windows applications must ship this runtime directory, including the shim for the application architecture and the console hosts for supported native Windows architectures. Native probes must cover successful and failed launches, repeated cleanup, keyboard input, resize, exit status and full-output shutdown before acceptance. Raw SDK evidence is diagnostic only: GitHub Actions run 34476091245, package sha256:175640566a3b59c4b132070ee96c2c77e5ab7edd2e92732a5eb3610bbf63d90e.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/terminal/pty/windows.rs`, `src/terminal/pty/windows/runtime.rs`, `src/terminal/owned_pty.rs`.

Behavior checks: `tests/api_widget_behavior/conpty_probe.rs`, `scripts/check-conpty-platform.py`, `tests/api_widget_behavior/terminal.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
