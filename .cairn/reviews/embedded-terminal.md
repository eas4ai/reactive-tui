# Embedded terminal review

commit: 8f357ead6c1125a0e91d535b9b9b39bc20d85ccd
findings:
  - closed: EMB-001 PTY descriptors have RAII ownership; exec and setup failures reach spawn callers; command configuration is exercised with a real child.
  - closed: EMB-002 native borrows remain on one worker; validated owned cells preserve graphemes, occupancy, RGB, decorations and cursor state through the local renderer.
  - closed: EMB-003 native keyboard modes determine encoding; App reserves the configured quit chord and forwards child interrupt keys.
  - closed: EMB-004 resize acknowledges both PTY and native screen changes before publication; zero dimensions retain the valid frame.
  - closed: EMB-005 bounded queues, reads and latest snapshots prevent accumulating output; errors remain observable and shutdown joins the worker and reaps the direct child.
  - closed: EMB-006 App explicitly restores its backend on return and error; actual controlling-PTY probes verify termios and child cleanup; inherited renderer evidence is current.

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

Reviewed the committed candidate on 2026-09-07 without changing product code.
Examined PTY setup failure, disconnected startup receiver, native capture
failure after child spawn, natural exit, explicit shutdown and Drop. Owned
files close on each setup error. The child owner kills/reaps on scope exit;
the session stops and joins its worker. Foreground cleanup verifies the
child session, and a cached completed status prevents repeated shutdown
from targeting a reused child PID. Arbitrary detached descendants remain
outside the documented ownership contract.

Examined worker fairness under continuous output and queued input. Reads are
limited per turn, command handling has a fixed budget, pending encoded bytes
have an explicit cap, and poll is bounded. Native replies share that cap.
Only the newest immutable frame is retained by the session. Native scrollback
and APC limits are configured before child execution. Limits do not promise
a fixed total process footprint or recoverability from native abort/OOM.

Examined the native boundary: terminal, render iterators and encoder remain
on their owning thread. No unsafe Send/Sync assertion was added. Grapheme
buffer length is queried and bounded before allocation, and borrowed native
cells become owned strings and values before publication. Frame construction
rejects controls, malformed occupancy and out-of-bounds cursors. Underline
variants are explicit; an unknown native variant returns an error. Source
and build pins are recorded; this is not an audit of all upstream native code.

Examined App update/error paths and renderer handoff. Background revisions
request redraw without input; terminal exit gets a final redraw before App
leaves. A final PTY drain closes the race between the last read and child
exit observation. Resize publishes only after both operations succeed;
failure stops the session. App attempts backend shutdown even when its loop
returns an error. Complete frames can switch back to Element painting, and
extended cell decorations participate in equality and composition. Per-cell
reset prevents style leakage; controlled mutation demonstrates its necessity.

Compared mechanisms against every EMB falsifier and inherited RND evidence.
The independent host parser, snapshot assertions, exact SGR checks and real
App PTY probes cover different boundaries; none is represented as direct
host compatibility testing. No unresolved finding within this commitment
requires a code change. The developer-host counter issue and legacy failures
remain explicitly outside these completion claims.

Self-audit against production rules 1–14: scope and interfaces remain bounded;
errors and process ownership are explicit; dependency provenance, public API,
example and limits are documented; checks ran against committed inputs and
controlled violations failed. Static-tool limitations and existing warnings
are recorded above. No further revision is required for this commitment.
