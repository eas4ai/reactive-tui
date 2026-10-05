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
Status: Draft
