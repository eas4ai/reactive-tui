# Embedded terminal review

## Specification review

The checks must prove that the child owns a real terminal, not just that a
process printed text. Host output must be interpreted independently from
Ghostty's snapshot. The input probe must drive commands through App, and
delayed output must render without keys. Cleanup checks must observe child
exit and host termios, not merely cleanup escape strings. Failure and mutation
cases will be recorded before completion. No direct-host compatibility claim
follows from changing TERM in a pseudo-terminal.

## Mechanism demonstrations

The complete embedded-terminal script passed on 2026-09-07: two native unit
checks, eight integration checks, the real App shell PTY success and error
probes, and the inherited renderer gate. Clippy completed successfully;
new embedded files have no diagnostics. Existing legacy warnings remain.

Four temporary mutations each caused the targeted test to fail (exit 101):
removing per-cell SGR reset leaked bold onto blank host cells; suppressing
overline lost its asserted decoration; ignoring the pending-input overflow
lost the required error; and suppressing App background redraw left only
the initial frame. Each mutation was restored. The full gate then passed.
These are development demonstrations, not Cairn acceptance receipts.

The independent vt100 host oracle covers combining text, CJK occupancy,
colors and base styles, erasure, alternate screens, and cursor state.
Its Unicode model cannot represent joined emoji: that case is checked in
the owned snapshot and inherited exact-byte renderer tests. Extended
underline and overline are checked in the snapshot and emitted SGR tests.
The real App PTY checks observe host termios and child reaping on normal
exit and a bounded-reply failure. Direct developer-host testing is not claimed.

Ripwire root quality-delta and test-gate returned nonzero (2 and 4): the
scan includes ignored reference trees and legacy findings. These static
reports are not acceptance passes. The changed cell style formatter was
simplified after its complexity warning; runtime gates above passed. A src-only scan also returned 2: it flags
trait/test entry points as dead code, small standard cleanup bodies as
duplicates, and explicit keyboard/style mappings and worker ownership
arguments for size. These are reviewed maintenance tradeoffs, not hidden
passes; adding generic indirection solely to silence them is not warranted.


## Final review

Pending.
