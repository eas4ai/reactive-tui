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
