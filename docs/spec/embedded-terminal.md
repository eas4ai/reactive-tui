# Embedded terminal

Status: Agreed 2026-09-07
Prefix: EMB

The developer authorized continuing the recovery after the SuprTUI foundation.
This commitment implements the previously agreed libghostty-rs direction:
one interactive shell displayed inside an application using the new renderer.
The acceptance environment is a Unix pseudo-terminal. Direct checks in the
developer's host terminal remain distinct from deterministic probes.

[EMB-001]
The embedded session MUST launch the requested executable on a real PTY with the configured dimensions, arguments, environment, and working directory.
Spawn failures MUST return an error to the caller.
Falsifier: the child reports that stdin or stdout is not a terminal, configuration is lost, or a missing executable appears to start successfully.
Mechanism: `scripts/check-embedded-terminal.sh`, controlled child-process integration tests.

[EMB-002]
The session MUST interpret child output using pinned libghostty-rs bindings and present its cell screen through SuprTUI.
The adapter MUST preserve graphemes, wide-cell occupancy, colors, text attributes, and cursor visibility and position.
Falsifier: the captured host screen differs from the expected styled child screen after output, erasure, or alternate-screen transitions.
Mechanism: `scripts/check-embedded-terminal.sh`, terminal-state and captured-output assertions.

[EMB-003]
The application MUST forward text, Enter, navigation keys, Escape, and Ctrl+C to the child using libghostty's keyboard encoder.
The internal terminal acceptance probe MUST reserve Ctrl+Q for leaving the host application while retaining existing App quit behavior by default.
Falsifier: typed commands do not execute, navigation emits the wrong mode-dependent sequence, Ctrl+C closes the host instead of interrupting the child, or Ctrl+Q cannot quit.
Mechanism: `scripts/check-embedded-terminal.sh`, keyboard and interactive PTY probes.

[EMB-004]
A nonzero pane resize MUST update both the child PTY dimensions and libghostty's screen before the next published snapshot.
Zero-size resize notifications MUST retain the last valid dimensions.
Falsifier: the child reports stale dimensions, the published screen has the wrong size, or a zero-size notification panics.
Mechanism: `scripts/check-embedded-terminal.sh`, child size queries and snapshot assertions.

[EMB-005]
Child output MUST trigger application redraw without requiring a host keypress.
The session MUST expose child exit status and I/O failures.
The session MUST reap its owned child and stop its worker on shutdown or drop.
Input and output buffering MUST be bounded.
Falsifier: delayed output stays invisible, failures appear successful, an idle session cannot shut down, the owned child survives shutdown, or output can accumulate without a bound.
Mechanism: `scripts/check-embedded-terminal.sh`, delayed output, exit, failure, and lifecycle probes.

[EMB-006]
The internal terminal acceptance probe MUST run through App and restore the host terminal on normal exit and application errors.
The existing SuprTUI renderer requirements MUST continue to pass.
Falsifier: the probe bypasses App, host terminal settings remain changed, or an existing renderer check fails.
Mechanism: `scripts/check-embedded-terminal.sh`, existing renderer gate plus internal PTY checks.
