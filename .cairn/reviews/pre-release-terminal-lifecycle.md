# Review: pre-release-terminal-lifecycle

commit: ee79880405e6a9bd50a9f606186f5be9448cd4cc
findings:
  - resolved: TRL-001 restores owned terminal state after worker and App-thread panics, then leaves the panic message visible.
  - resolved: TRL-002 sends SIGTERM, SIGINT, and SIGHUP through the App wake path, chains the prior panic hook, and preserves another thread's terminal ownership.
  - resolved: TRL-003 blocks host control characters at public title and Surface boundaries and discards unsolicited OSC 52 clipboard input.
  - resolved: TRL-004 routes terminal discovery and configuration helpers through the existing bounded owned-process runner.

## Scope examined

Re-read TRL-001 through TRL-004, all four mechanism declarations, the
committed implementation diff from `0e07046e` through `ee798804`, the latest
receipts, and the focused test runners. Traced App panic cleanup, termination
signal registration and release, Surface storage and diff output, host and
child title handling, OSC input parsing, and the `which` and `stty` process
paths. No production code changed during this review.

## Panic and signal attack

The panic checks use a real PTY. They enter raw mode, the alternate screen,
mouse capture, bracketed paste, and hidden-cursor mode. Worker and App-thread
panics must restore the original termios state and emit the complete restore
sequence before the final visible panic marker. The validator rejects missing
restoration, changed termios, and a marker emitted only before restoration.

`App::run` catches the panic outside the scope that owns the backend. This
lets the backend drop and restore the terminal before the panic is resumed.
The Unix panic hook only calls the previously installed hook. It does not
restore global terminal state. The cross-thread probe proves that a panic on a
non-owner thread cannot tear down a live terminal owned elsewhere.

The signal worker uses signal-hook's pipe registration, copies App wakers
outside the lifecycle lock, and requests normal App stop through the existing
wake path. The last registration unregisters the signal actions, closes the
pipe, and joins the worker after releasing the lifecycle lock. Real subprocess
tests send SIGTERM, SIGINT, and SIGHUP after terminal entry and require bounded
normal shutdown, restoration, and the post-restoration marker. These signals
are POSIX behavior; this commitment makes no claim about Windows console
control events.

## Terminal input and output attack

The `syn` inventory covers every public Surface string writer and every public
title writer or title-sequence constructor in the root crate. It fails if a
path or signature changes without a matching behavioral case. Property tests
exercise C0, DEL, C1, CSI and OSC-shaped payloads, plus ordinary Unicode,
through every inventoried path. They inspect stored cells, graphemes,
`DiffWriter` bytes, captured host title bytes, and parsed child title events.

`Surface::set` is the common safe-cell boundary and replaces control
characters before storage. Host `Terminal::set_title` rejects them. The ANSI
title constructor and child OSC title parser replace them. Both BEL- and
ST-terminated OSC 52 responses are consumed without producing `Paste`. The
validator rejects a missing writer, raw control output, and an unsolicited
paste event. This requirement is limited to title and Surface text paths; it
does not assert that unrelated working-directory or hyperlink constructors
accept untrusted strings.

## Helper process attack

The source inventory finds the two terminal helper programs: `which` in image
capability detection and `stty size` in character-dimension discovery. Each
call must use `owned_process::run` and must not call unbounded `output`,
`status`, or `spawn`. Fixtures with an unbounded call, direct-child-only kill,
or a missing helper are rejected before the behavior probes run.

The behavior probes prepend private helper executables to `PATH`. Each helper
records its PID, forks a recorded descendant, and blocks. The public image
capability path and pixel-mouse path return through their documented fallback
within the outer deadline. The owned runner applies a 250 ms local-helper
deadline, starts a Unix process group, kills that group on timeout, and waits
for the direct child. Tests require every descendant to stop and every direct
child to be absent from the process table. The Unix focus matches the
`which`/`stty` helpers and the process-group rule they previously bypassed.

## Verification and limits

The latest committed receipts pass TRL-001 through TRL-004. During development,
the real TRL-004 mechanism failed against the prior code: both helper probes
exceeded their five-second outer deadline, and the inventory found the direct
`Command::output` path. The corrected mechanism passes all four cases in under
one second. The full library suite passed 1,032 tests with 7 ignored. Targeted
Clippy for the library and TRL-004 integration target passed with warnings
denied. The stable all-features Clippy command is not a valid repository gate
because the `simd` feature enables nightly `portable_simd`; it also reports
three existing FFI warnings outside this commitment.

GitNexus rated both production edits LOW risk and found no affected execution
flows. Ripwire reported unchanged function contracts and a zero quality gate
after the required short-horizon-churn acknowledgement. Its remaining
dead-code reports are false positives for Rust integration-test entry points;
the two one-line `syn::Visit` callbacks are deliberate trait implementations.

No unresolved finding remains inside this commitment. The work adds no
dependency, persistence format, network boundary, secret handling, or public
API. It uses the existing App wake path, Surface boundary, parser boundary, and
owned-process lifecycle primitive.

## TRL-002 rewording review (spec lint repair)

Re-read revised TRL-002 and its falsifier against mechanism
`pre-release-terminal-signal-shutdown`. The revision splits one two-obligation
sentence into two sentences with identical meaning: panic-hook installation
chains the prior hook, and the installation tears down no terminal state owned
by another thread. The falsifier is unchanged in meaning. The mechanism probes
SIGTERM/SIGINT/SIGHUP shutdown with PTY termios restoration, proves the prior
hook runs exactly once, and proves a cross-thread panic leaves the terminal
active until its owner shuts it down — so both revised obligations remain
covered.

Failure demonstration: a throwaway probe (`/tmp/trl002-review-probe.py`, not
committed) ran the check's own `prove_validator()`, which feeds corrected PTY
observations (accepted) and violating ones — immediate signal exit, missing
restoration, skipped hook chaining, cross-thread restoration, unbounded child
(rejected). No mismatch found. No code changed during this review.
