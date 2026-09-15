# Review: pre-release-reactive-concurrency

commit: 4e04c7ae5c5cb67f07306c312e4493598fba0fb3
findings:
  - open: RAC-002 makes safe `Ref::update` re-entry depend on an integer-cast raw pointer that Miri rejects for invalid mutable aliasing.

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

## Unresolved finding

The ordinary RAC-002 tests use integers, so they do not expose reference
aliasing. A temporary path-dependent consumer held a string slice from the
outer `Ref<Vec<String>>::update`, called `update` again through a cloned
`Ref`, cleared and reallocated the vector, and then read the held slice. This
consumer contains only safe code. `cargo +nightly miri run` rejected the
library at `src/hooks/refs.rs:92`: creating the nested `&mut T` from the stored
integer pointer violated the borrow stack. With strict provenance enabled,
Miri rejected the earlier integer-to-pointer cast at line 87 as unsupported.

The current mechanism can therefore pass while RAC-002 exposes undefined
behavior through a safe public method. This must be resolved before the
commitment is complete. The repair also has to reconcile the earlier API-019
decision that `Ref<AtomicUsize>::update` keeps working without a `Clone` bound;
safe synchronous mutable re-entry and that bound cannot both be provided by
the present `FnOnce(&mut T) -> R` API.

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
