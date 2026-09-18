# Review: pre-release-reactive-concurrency

commit: c8fba575e8b3023d9cfb44d9705c323cbececf21
findings:
  - resolved: RAC-002 now uses cloned update drafts, rejects unsafe shared-Ref expressions, preserves non-Clone construction and replacement, and passes the former Miri aliasing case.

## Scope examined

Re-read RAC-001 through RAC-003, their three mechanisms, all focused tests, the
animation owner and runtime paths, reactive effect registration and cleanup,
shared and local reference implementations, thread-safe signals, throttle and
debounce callbacks, scheduler timer storage, and fallback worker ownership.
Compared the current implementation with the commitment start at `d9019211` and
checked the latest committed receipts for all three requirements.

RAC-001 retains one owner per hook slot, routes delayed and staggered work to
the selected scheduler, cancels runtime and timer work at component cleanup,
and snapshots the animation registry before invoking callbacks. RAC-003 uses a
condition variable for new work, deadlines, and shutdown; the worker holds only
a weak scheduler between processing turns, and the global fallback registry
also retains only a weak reference.

## Failure demonstrations and corrected cases

The RAC-001 validator supplies one valid observation and independently changes
owner count, remaining runtime work, remaining scheduler work, post-unmount
updates, thread growth, and scheduler-visible stagger work. Each changed case
is rejected before the retained-owner runtime tests pass. The RAC-002 Python
validator rejects controlled panic, timeout, poison, and missing-progress
observations before six bounded re-entry tests pass. RAC-003's committed
baseline rejects the original one-millisecond poll loop; its validator also
rejects idle wakeups, a missing callback, and a worker that remains alive. The
corrected worker passes the two scheduler-local lifecycle tests.

## Resolved finding

The ordinary RAC-002 tests use integers, so they do not expose reference
aliasing. A temporary path-dependent consumer held a string slice from the
outer `Ref<Vec<String>>::update`, called `update` again through a cloned
`Ref`, cleared and reallocated the vector, and then read the held slice. This
consumer contains only safe code. `cargo +nightly miri run` rejected the
library at `src/hooks/refs.rs:92`: creating the nested `&mut T` from the stored
integer pointer violated the borrow stack. With strict provenance enabled,
Miri rejected the earlier integer-to-pointer cast at line 87 as unsupported.

Decision `require-cloned-snapshots-for-reentrant-ref-updates` resolves the
conflict. `Ref::update` now requires `T: Clone`, copies the committed value,
runs the callback on that independent draft without a lock, and commits only
after the callback returns. `Ref` and `use_ref` still accept non-`Clone` values;
`set_current` still replaces them and preserves the allocation address. A
compile-fail doctest fixes the narrower `update` contract.

The RAC-002 test now holds a string slice from the outer draft while a nested
update clears and reallocates its own draft. A syntax-aware check rejects any
unsafe expression in the shared `Ref` implementation. The same temporary safe
consumer that Miri rejected before the repair now passes under nightly Miri
with strict provenance. The focused non-`Clone` compatibility test, compile-fail
doctest, seven RAC-002 unit tests, and refreshed Cairn evidence all pass.

## Other limits

The Linux thread-count observation in RAC-001 is supplemented by syntax-aware
rejection of thread spawning, so platforms without `/proc/self/task` still
check the implementation shape. RAC-003 measures one scheduler's own worker
state and therefore does not confuse unrelated test threads with its owner.
Default-feature library tests, the named Ripwire integration targets, and
no-dependency library Clippy passed with eight build jobs. Full default
integration discovery remains affected by the repository's existing
feature-gating failure in `ffi_app_representation`; that is outside this
commitment and was not treated as passing evidence.

## RAC-003 rewording review (spec lint repair)

Re-read revised RAC-003 and its falsifier against mechanism
`pre-release-fallback-timer-lifecycle`. The revision splits one two-obligation
sentence into two sentences with identical meaning: the fallback worker sleeps
until scheduled work or shutdown, and it stops when its last owner is gone.
The falsifier is unchanged in meaning. The mechanism's validator rejects idle
wakeups, unfired timers, and surviving workers, rejects the former 1ms poll
loop by source shape, and runs the focused `rac_003_` tests serially, so both
revised obligations remain covered.

Failure demonstration: a throwaway probe (`/tmp/rac003-review-probe.py`, not
committed) called the check's own `validate_fallback_timer` with the corrected
observation (accepted) and three violating observations — idle wakeup, unfired
timer, surviving worker — each rejected; `reject_permanent_fallback_polling`
also passes against current `src/hooks/timer.rs`. No mismatch found. No code
changed during this review.
