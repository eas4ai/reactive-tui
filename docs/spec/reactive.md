# Reactive state

Prefix: SIG

A `ThreadSafeSignal` (src/reactive/hooks.rs:193) is a value behind a mutex
with a version and the Apps that read it as subscribers. `get` clones the
value, `set` stores a different value and wakes the subscribers, and `update`
clones the value, runs its callback on the copy with the lock released, and
stores the result through `set`; the callback may read and write the signal
again (test rac_002 in hooks.rs). `use_reducer` dispatches an action through
`update`. Read on 2026-10-02 from the developer's review of e30afa23
(finding 2, P1): two threads that update at once both start from the same
copy and the later store overwrites the earlier, so two increments of zero
leave 1. The framework's own counters and states go through `update` too:
the dialog engine's change counter and the wizard's visited set, the image
worker's, popover's and terminal monitor's revision counters, and the
clipboard and pointer processor hooks' states. `Ref` (src/hooks/refs.rs)
already has `update` on a copy beside `update_atomic` under its lock.

The single-threaded side (src/reactive/signal.rs, src/reactive/runtime.rs)
has a `Memo` that computes a value from signals and a `RuntimeContext` with
`create_signal` and `create_effect`; the hook animations (src/hooks/animation.rs)
drive `use_animation` and `use_spring` from a frame driver the App and the
screen runtime run. Read on 2026-10-04 from the developer's production code
review of 65e618ec (findings C07, C08 and C17 to C19) and checked on
2026-10-05: a `Memo` never computes again after it is made, a context's
effects run once and are not told of signal changes, a hook animation paused
past its duration loses its task, a spring drops the velocity an impulse
gave it, and the hook animation's loop settings never reach its driver.

## Observed

(none yet)

## Draft

[SIG-001] `ThreadSafeSignal::update_atomic` MUST run its callback on the stored value while no other `update_atomic`, `set` or `get` of that signal proceeds, so that when several threads apply updates through it the value afterwards is every update applied once, in some order; `use_reducer`'s dispatch and the framework's own read-modify-writes (the dialog engine's change counter and the wizard's visited set, the image worker's, popover's and terminal monitor's revision counters, the clipboard and pointer processor hooks' states) MUST go through it; an `update_atomic` whose callback calls `get`, `set`, `update` or `update_atomic` on the same signal MUST fail at once with a message that names the misuse instead of waiting forever; and `ThreadSafeSignal::update` MUST keep running its callback on a copy with the lock released, so the callback may read and write the signal, with its documentation saying that two concurrent `update` calls may overwrite each other.
Falsifier: Two threads that each dispatch 10,000 increments to one `use_reducer` state, or each call `update_atomic` 10,000 times on one signal incrementing it, leave a value other than 20,000; one of the listed read-modify-writes still calls `update`; an `update_atomic` callback that calls `get`, `set`, `update` or `update_atomic` on its signal waits instead of failing with the message; or the doc comment of `update` does not say that concurrent updates may overwrite each other.
Mechanism: reactive-signals
Rationale: The developer's review of e30afa23 (finding 2, P1) showed two concurrent increments through `update` yield 1, and `use_reducer` dispatches through that path, so concurrently dispatched actions can disappear.
Status: Agreed 2026-10-02

[SIG-002] An `Animation` MUST run its `on_update` and `on_complete` callbacks with none of its own locks held, so a callback may read the animation's progress, state and current values and change its state, and return, whether the animation is updated directly, by an `AnimationManager` or by an App.
Falsifier: An `on_update` callback that calls `get_progress`, `get_state` or `get_current_values` on its animation, or an `on_complete` callback that calls one of them or pauses, stops or restarts the animation, does not return within 5 seconds of the update that runs it.
Mechanism: review-high
Rationale: `update` held the animation's state lock while it ran `on_update`, so a callback that read its own progress waited forever for a lock its own thread held (the developer's code review of 2026-10-04, N01).
Status: Agreed 2026-10-04

[SIG-003] `reactive::signal::Memo::get` MUST return what its compute function gives for the current values of the signals the function read: after one of those signals changes, the next `get` MUST compute again.
Falsifier: A `Memo` that doubles a `Signal` holding 1 returns 2 from `get` after the signal is set to 5.
Mechanism: review-core
Rationale: A `Memo` computed once when it was made and never again; its test called the private recompute by hand (the developer's code review of 2026-10-04, C07).
Status: Agreed 2026-10-05

[SIG-004] An effect made with `RuntimeContext::create_effect` MUST run when it is made and again after each change of a signal of that context it read during its last run, after running the cleanup its last run returned, until `unregister_effect` removes it or the context is dropped; its function is reusable, an `Fn` and no longer an `FnOnce`.
Falsifier: An effect that reads a context signal and counts its runs has run once after the signal is set; its last cleanup has not run before its second run; or it runs again after `unregister_effect` removed it.
Mechanism: review-core
Rationale: The context's signals did not report reads or writes to its runtime, and an effect's function was consumed by its first run (the developer's code review of 2026-10-04, C08).
Status: Agreed 2026-10-05

[SIG-005] A `use_animation` animation that is paused MUST keep its place however long it stays paused: after `resume` it MUST continue from the value it had when paused and finish after the playing time it had left.
Falsifier: An animation from 0 to 1 over 100 ms, paused 20 ms in and held paused for 300 ms, is no longer Playing after `resume`, does not advance from its paused value, or finishes before 80 ms more of playing time.
Mechanism: review-core
Rationale: The hook's frame driver measured progress by wall time and dropped the animation's task once that passed its duration, paused or not, and `resume` did not bring it back (the developer's code review of 2026-10-04, C17).
Status: Agreed 2026-10-05

[SIG-006] `SpringHandle::apply_impulse` MUST change the spring's motion: a spring at rest at its target that receives an impulse MUST move away from the target in the impulse's direction in the frames that follow, and come back to rest at its target.
Falsifier: `use_spring(0.0)` given `apply_impulse(10.0)` reports position 0 after the next frame, moves the other way, or does not come back to rest at 0.
Mechanism: review-core
Rationale: A frame computed the spring's motion from its position and target alone and overwrote the velocity an impulse had set (the developer's code review of 2026-10-04, C18).
Status: Agreed 2026-10-05

[SIG-007] `use_animation`'s `AnimationConfig::loop_count` and `loop_behavior` MUST reach the frame that drives the animation: `None` plays once, `Some(0)` repeats until stopped, `Some(n)` plays n times, and `LoopMode::PingPong` plays every second pass from the end value back to the start; the field's documentation MUST say so.
Falsifier: An animation configured with `loop_count: Some(0)` is not Playing two durations after it started; with `Some(2)` it stops before two durations or is still Playing after three; with `PingPong` its second pass does not run from the end value back to the start; or the doc comment of `loop_count` says that `None` repeats.
Mechanism: review-core
Rationale: The hook's frame driver stopped after one duration whatever the loop settings, and the field's documentation said `None` loops forever while the code reads `Some(0)` that way (the developer's code review of 2026-10-04, C19).
Status: Agreed 2026-10-05
