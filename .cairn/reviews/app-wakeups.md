# App wakeups review

## Specification review

Acceptance must observe the wait boundary and final rendered state, not
just a wake counter. Exercise notifications both before and during waits,
equal signal writes, timer reentrancy, earlier deadlines, coalescing and
shutdown. Existing polling roots retain behavior. Mutation demonstrations
and the final ownership review remain pending.

## Mechanism demonstrations

The baseline failed because AppWaker did not exist. Development integration
then caught two real compatibility regressions: updating a polling root again
before presenting its requested frame changed the inherited expected output
from 0,1 to 0,2; and fast input drained host events faster than the native
worker could accept its bounded queue. App now presents pending frames before
polling the root again, and TerminalView retains one rejected key while App
pauses further input until the worker signals space. Existing assertions were
retained; the inherited checks subsequently passed those cases.

Four temporary mutations each failed the targeted runtime test with exit 101:
removing ThreadSafeSignal notification left the host state stale; removing
scheduler notification left queued work asleep; discarding pending wake flags
made a preexisting request wait for its timeout; and bypassing the frame
budget failed the frame-spacing assertion. Every mutation was restored.

The first real-PTY wake probe had an invalid screen predicate: a partial ANSI
sequence could contribute a digit before the counter update arrived. The
probe now interprets complete synchronized frames with the existing ASCII
screen helper. Five repeated runs then passed. Its idle-output observation
is paired with instrumented backend assertions that idle App remains in a
single wait; absence of bytes alone would not prove absence of polling.

Specification lint required splitting three compound MUST sentences. Their
obligations and falsifiers were preserved. Final acceptance receipts follow
only after committing the corrected candidate.

Ripwire edit-check for run_loop passed. Its src-scoped quality-delta returned
2 and test-gate returned 4. The reports include generic lock/getter bodies
matched to unrelated helpers, recent-change warnings and legacy or trait
entry points without recognized graph test edges. These are not reported as
passing static gates. The actual Rust, PTY and specification checks are the
acceptance mechanisms. Clippy completed with existing legacy diagnostics;
new wake and scheduler files had no diagnostics in that run.

## Final review

Pending.

## Revised mechanism review: WAK-001

Compared the revised text and falsifier with wake.rs, the backend wait
adapter, App request handling and the acceptance runner. Splitting the
compound sentence preserved both bounded coalescing and loss-free waiting.
The primitive tests cover requests before wait and 200 registration races;
the App tests cover idle stop and final burst state; the real PTY checks a
background change against Crossterm's actual blocking input wait. Pending
storage is fixed flags and one task waker. Removing pending flag retention
failed its runtime assertion; restoring it passed the complete development
gate. No mismatch requires a separate code change.

## Revised mechanism review: WAK-002

Compared the separate equal-write and ownership obligations with Signal,
ThreadSafeSignal, render Scope and weak generation subscriptions. Tests cover
both signal types, two independent App notification sources, dependency
removal on later renders, closed handles and weak ownership. The threaded
App test checks actual displayed value after an idle change and no redraw
on an equal write. Suppressing ThreadSafeSignal notification failed that
runtime test; restoration passed the full development gate. No assertion
was weakened when the specification sentence was split. No mismatch found.

## Revised mechanism review: WAK-003

Compared scheduler notification, earliest deadline selection and callback
execution with the revised separate obligations. The integration test starts
with a distant timer, queues work while App waits, inserts an earlier timer,
and schedules another timer from a callback after clear. The scheduler unit
test cancels a running interval from inside its own callback and verifies
that only the nested timeout follows. Callback bodies execute outside both
queue and timer locks. Suppressing scheduler notification failed the queued
work assertion; restoration passed the complete development gate. Existing
timer assertions remain intact. No mismatch requires implementation work.
