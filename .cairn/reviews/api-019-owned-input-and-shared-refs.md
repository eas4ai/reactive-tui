# API-019 owned input and shared references checkpoint

Status: partial implementation verified locally; API-019 remains failing.

The developer approved the Unix InputReceiver migration in api-019-api-020.
The two new ownership decisions precede implementation. Unix streams now own
cancellation and joining, independently opened nonblocking descriptors, bounded
64-item queues and 4096-byte raw reads. Receiver, iterator and final terminal
owner cleanup wake idle reads and full queues. Weak registrations avoid cycles;
serialized joining makes concurrent teardown wait. Existing descriptors retain
their flags. Shared, callback and multi-reference hooks now retain positional
state. Callback replacement and replaced-value destruction run outside locks.
Existing generic bounds remain intact. LocalRef is unchanged.

## Verification and failure demonstrations

`api-019-shared-and-input-verification/20260913T003810644391Z/` records:
1006 library tests passed, two ignored; the complete focused Unix input group,
strict all-target Clippy, and cargo fmt --check passed. Unix input covers all
14 private-PTY cases, five unit tests, and the held-child cancellation check.
`api-019-input-final/20260913T003226430072Z/` records six checker controls,
Python syntax, and controller positive/negative cases. The controller mutant
removed parent-death SIGKILL: the test failed on the surviving probe, then safely
killed and reaped it through its PID handle. No child remains running.

`api-019-input-bound-controls/20260913T002012028520Z/` changes queue capacity
from 64 to 1024: real PTY tests reject the excess queue. Source was restored
exactly and the input group passed. A later Debug implementation preserves the
standard receiver's lack of a T:Debug bound; final library checks include it.
`api-019-ref-callback-controls/20260913T003334020570Z/` holds the value lock
across callback invocation: bounded reentry fails and the child is reaped.
After exact restoration, all nine shared/forwarded reference cases pass.
The unchanged local retention case still fails; the full reference group reports
nine passes and one failure, never a complete acceptance pass.

An earlier controller test raced with repaired, faster probe cleanup and observed
normal exit instead of SIGKILL (`api-residual/20260913T002813576457Z/`). Its raw
failure is preserved. An explicitly parked diagnostic probe now establishes
readiness before controller termination; PID handles prevent PID reuse hazards.

The timestamped `api-019-checkpoint/` directory records the migration guide's
rustdoc test, hook-state/lifecycle integration tests, external probe formatting,
Python syntax and final Ripwire checks. These are development checks, not Cairn
receipts. No historical evidence or raw output was rewritten.

## Review of mechanisms and implementation

Reviewed cancellation predicate/notification ordering, full-queue wakeup,
independent descriptor ownership, concurrent joining, iterator ownership,
callback reentry/destructor behavior, stale captures and preserved type bounds.
Each queue/lock/controller mutation fails for the intended behavior. The input
checker requires the exact unique case set and rejects nonzero, timed-out or
unreaped results. The broader residual coverage gate remains deliberately
incomplete rather than treating Linux input success as whole-catalog coverage.

Ripwire edit checks pass. Quality-delta and test-gate remain nonpassing static
reports; their raw output is retained. Global findings include ignored reference
trees. Focused findings identify dynamically dispatched checks, callbacks, public
receiver/iterator types and tests as dead despite actual execution. Similarity
reports concern small queue notifications, Arc/Mutex construction, and shared
slot calls; a cross-subsystem abstraction would obscure distinct error and
ownership paths. Long reentry tests include necessary child/deadline cleanup.
These reports do not establish acceptance and are not relabeled as passes.

## Production standard audit and remaining work

Rules 1–4: changes follow the approved repair and recorded ownership decisions;
no new dependency or unrelated rewrite. Public migration is documented and its
example compiles. The unapproved local scope is proposal only.
Rules 5–8: stream errors retain disconnect semantics; owned descriptors cover
unsafe poll/read lifetimes, no unsafe Send promise is introduced, queues are
bounded, cancellation reaches blocking waits, and diagnostics own/reap children.
Rules 9–12: API-019 is the sole in-progress todo; positive and violating cases
were executed with concurrency capped at 12. Failures remain visible.
Rules 13–14: this checkpoint is coherent but is not a production completion
claim. A separate compatibility decision is necessary for local values; its
concrete API, lifecycle cost and alternatives are in api-019-local-owner/review.md.

Native macOS verification, wider Unix signal/raw-owner/parser/legacy loop work,
other residual audit families, backend test discovery, API-020 capture-timeout
cleanup, all current formal receipts and the commitment-wide final review remain.

Final checkpoint: the migration doctest and all 12 hook state/lifecycle integration
tests passed. Final Ripwire edit checks passed; quality-delta exited 2 and
test-gate exited 4, with focused findings retained alongside raw reports.
The unrestricted staged whitespace check reports only captured test-output
whitespace; those bytes are preserved. Authored-file whitespace is checked separately.
