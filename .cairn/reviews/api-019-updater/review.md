# Owned Updater implementation

Developer approval api-019-api-020-2 is recorded. Updater now requires update,
and App owns registrations. Weak request handles coalesce pending work, wake
only the owning App and become inert after registration removal or App close.
App snapshots pending flags before invoking callbacks in registration order,
without holding request locks around user code. Reentrant requests wait for a
later dispatch batch. Removal cancels queued work and drops callbacks on the
App thread at the next turn; an already executing callback may finish.

Three public integration tests verify worker wake from idle, rendering updated
state, independent App state, coalescing, and error/unwind/unrun cleanup. Two
unit tests cover deterministic ordering, reentrant scheduling and cancellation.
The focused API-019 updater group passed discovery and all five tests. The first
watchdog draft used a nonexistent stop method; its compile failure is retained,
and the corrected test uses request_stop. Strict all-target default Clippy passed
with the existing upstream dependency warning. Focused Ripwire edit-check passed;
quality-delta returned 2 for broad preexisting/reference findings and test-gate
returned 4. Those two tools are not passing checks.

The safe disabled-dispatch control removes both App dispatch calls, runs the
unchanged public consumer tests, and fails for absent updates, including the
live-loop repaint assertion. Source was restored in a finally block. The same
public tests then passed. Raw failing and corrected output are retained here.
No formal acceptance receipt is claimed before committing and running Cairn.
Native and complete regression evidence must be refreshed after this source
change. Other API-019 concerns remain open; inventory coverage is not weakened.

Self-audit: callback ownership and request storage are bounded per registration;
no callback runs under a state mutex. Every state is closed before callbacks are
dropped during App cleanup, preventing destructor requests from reviving work.
The migration and cancellation timing are documented. The implementation is
ready for formal checks, with the broader commitment still incomplete.
