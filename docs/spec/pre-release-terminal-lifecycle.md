# Pre-release terminal lifecycle safety

Status: Agreed 2026-09-14
Prefix: TRL

This specification remediates Fable audit findings H4, M5, L6, L7, L10, L12,
and L18.

[TRL-001]
A backend worker panic and a main-thread panic MUST restore raw mode, the
alternate screen, cursor state, and other owned terminal modes. The panic
message MUST remain visible after restoration.
Falsifier: An injected panic leaves the shell in application mode, loses the
panic message, or prevents shutdown from completing.
Mechanism: PTY tests inject worker and main-thread panics and assert the final
terminal byte sequence, restored termios state, and visible panic text.

[TRL-002]
SIGTERM, SIGINT, and SIGHUP MUST request bounded shutdown through the existing
wake path. Panic-hook installation MUST chain the prior hook. The installation MUST NOT
tear down terminal state owned by another thread.
Falsifier: A supported signal exits without restoration, shutdown blocks, a
prior hook is skipped, or one thread restores terminal state owned elsewhere.
Mechanism: Subprocess tests deliver each signal and controlled panics, then
verify exit, hook chaining, and terminal restoration.

[TRL-003]
Every public title or Surface string path MUST reject or encode host control
characters. Unsolicited OSC 52 input MUST NOT become a Paste event.
Falsifier: DEL, C1, OSC, CSI, or another control reaches host output through a
public string writer, or unrequested OSC 52 input produces a paste.
Mechanism: Property tests feed control ranges through titles, surfaces, child
titles, and the input parser and inspect emitted events and bytes.

[TRL-004]
Helper processes used for terminal discovery and configuration MUST use the
project's deadline, process-group termination, and reap rules.
Falsifier: A blocked `which` or `stty` child can block shutdown or survive
its deadline.
Mechanism: PATH-injected helpers hang and fork children; the check requires a
bounded error and no surviving process.
