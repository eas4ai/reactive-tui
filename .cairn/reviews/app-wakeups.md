# App wakeups review

commit: 1d678d488d6c128cc4de3646822a9127d25cecb3
findings:
  - closed: WAK-001 fixed pending flags survive wait entry; both condition-variable and task-waker paths observe requests; stop wakes App and closes handles.
  - closed: WAK-002 rendered signals subscribe weakly by App generation; writes notify after value locks release; equal writes, stale subscriptions and closed Apps are tested.
  - closed: WAK-003 queued work and changed deadlines wake App; timer callbacks run outside locks and cancelled intervals do not reinsert themselves.
  - closed: WAK-004 dirty frames retain their latest state while paced; wake-driven roots sleep; legacy polling roots and active animations continue; input and resize remain serviceable.
  - closed: WAK-005 native frame, exit, error and queue-space changes wake App; one pending key preserves bounded backpressure; final child output is rendered before exit.
  - closed: all inherited EMB and RND requirements have current passing committed evidence; direct developer-host claims and unrelated legacy cleanup remain excluded.

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

Reviewed the committed candidate on 2026-09-07 local time without changing
product code. Examined the wait handoff around publication, flag consumption,
task registration, timeout expiry and stop. Pending flags are guarded by the
same mutex as the condition-variable predicate. The native input adapter
registers before checking pending state; an unpark token survives a request
between the check and park. Its registration guard removes the task reference
on return and error. Spurious wakeups recheck both input and App state.

Examined signal reads racing background writes. ThreadSafeSignal records its
subscription while holding the value lock, then writers notify after releasing
that lock. The next render generation invalidates obsolete reads. Scope
restores the prior thread-local context on unwind. Subscriptions contain weak
notification-state references, not App or component owners; finished handles
are inert. No unsafe cross-thread trait implementations were introduced.

Examined the scheduler's queue transfer, earlier deadline insertion, callback
reentrancy and interval cancellation. Locks protect storage only while batches
are taken or restored. Running cancellation is recorded separately; clear
removes those records, which the runner treats as cancellation. App closes
its wake handle and clears its scheduler on drop. Arbitrary user callback
execution remains cooperative and cannot be preempted. Work items themselves
remain a queue; only notification storage has the bounded coalescing contract.

Examined rendering while new notifications arrive, active-animation deadlines,
legacy polling defaults and a root temporarily unable to accept input. Dirty
state survives frame pacing. Keyboard input is checked even during wake bursts.
TerminalView retains one unqueued key, and worker queue consumption wakes its
retry; it does not enlarge the native queue or discard keys. Timers and explicit
stop requests remain live while input is paused. Existing animation/timeline
registration and interpolation semantics remain unchanged; legacy inactive
registrations still require their normal cleanup.

Examined native final-frame and error ordering. Exit visibility is recorded only
when App obtains the final cell frame, and session errors are checked before
leaving. The new real-child test verifies final FINISHED text and reaping with
no host input. The real controlling-PTY gates verify signal redraw, input,
resize, normal/error/panic restoration and child lifecycle. Captured output is
interpreted at complete synchronized-frame boundaries. No direct developer-host
compatibility claim follows from these probes.

Checked the dependency boundary and documentation: the existing Crossterm
version gains its event-stream feature and futures-core supplies the Stream
trait. No WezTerm source was copied. Upstream input-helper cancellation is
explicitly distinguished from joining our owned renderer and terminal workers.
Standalone hook/runtime schedulers retain their documented existing lifecycle;
this work does not claim to merge those systems or resolve their legacy defects.

Final development gate and all three Cairn mechanisms passed against committed
inputs. Formatting checks on the touched Rust paths and git diff --check passed.
Final Clippy completed with no diagnostics in the new wake, scheduler, App,
input, embedded, test and example paths; legacy warnings remain. The four
controlled mutations each failed a runtime assertion and were restored.

Self-audit against production rules 1–14: implementation follows the approved
scope; interfaces are additive; ownership, error paths and limits are explicit;
source changes, example and documentation agree; verification includes real
failure demonstrations and committed receipts. No unresolved finding within
this commitment requires another code change.

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
