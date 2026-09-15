# Mechanism: pre-release-reactive-reentrancy

command: python3 -B scripts/check-pre-release-reactive-concurrency.py RAC-002
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/hooks/refs.rs
  - src/hooks/timer.rs
  - src/reactive
  - scripts/check-pre-release-reactive-concurrency.py
requirements:
  - RAC-002

The check MUST first prove its validator rejects controlled observations for a
panic, a timeout, poisoned state, or missing later progress. It MUST then run
focused tests in isolated worker threads with bounded receive deadlines. The
tests MUST re-enter each public callback path while that callback is active:
runtime effects create and unregister effects and track a signal; `Ref::update`
and `ThreadSafeSignal::update` read and update their own owner; throttle and
debounce callbacks call their own public operations.

Every case MUST complete before its deadline without a panic. After the
re-entrant call, a separate ordinary operation MUST still complete and expose
the expected state so a swallowed panic or poisoned lock cannot count as a
pass. The shared `Ref` case MUST retain a borrow into the outer draft while a
nested update reallocates its own draft. A `syn` inventory MUST reject direct
user callback invocation while an internal `RefCell` borrow or mutex guard
remains live in these paths and MUST reject unsafe alias construction in the
shared `Ref` implementation.
