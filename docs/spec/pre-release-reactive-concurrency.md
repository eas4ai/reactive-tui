# Pre-release reactive and animation concurrency

Status: Agreed 2026-09-14
Prefix: RAC

This specification remediates Fable audit findings H5, M6, L2, L3, L4, and L5.

[RAC-001]
Animation, spring, transition, and stagger hooks MUST keep one scoped state
owner across renders. Unmount MUST cancel pending work. Frame progress MUST NOT
spawn one operating-system thread per frame.
Falsifier: Re-render creates a new controller for the same hook slot, an
unmounted hook continues to update, a transition grows threads with frame
count, or stagger work has no driving scheduler.
Mechanism: Deterministic scheduler tests count owners, tasks, threads, updates,
and cancellations across render and unmount cycles.

[RAC-002]
Reactive effects, Ref updates, thread-safe signals, throttles, and debounces
MUST invoke user code after releasing internal borrows and mutexes.
Falsifier: A supported re-entrant callback causes RefCell panic, deadlock, or a
poisoned state that blocks later progress.
Mechanism: Re-entrant tests call create, unregister, read, and update operations
from inside each callback and require bounded completion.

[RAC-003]
Fallback timer infrastructure MUST sleep until scheduled work or shutdown and
MUST stop when its last owner is gone.
Falsifier: One unscoped timer creates a permanent one-millisecond wake loop or
leaves a live thread after all timer owners drop.
Mechanism: Clock and thread-lifecycle tests measure idle wakeups and require
bounded teardown.
