# Reactive state and hooks

Crate modules: `reactive`, `hooks`

## Purpose

Reactive values request work when state changes. Hooks attach state, effects,
timers, input tracking, animation, references, clipboard access, and performance
data to a component render scope.

## Main API

- `Signal`, `ReadSignal`, and `WriteSignal` store and split reactive values.
- `Effect` runs work and owns an optional cleanup callback.
- `Hooks` stores hook slots for a component instance.
- Core hooks include `use_signal`, `use_effect`, `use_effect_with_deps`,
  `use_context`, `use_reducer`, `use_previous`, and `use_memo`.
- Utility hooks cover timers, animation, mouse gestures, references, clipboard,
  frame timing, adaptive quality, and performance context.
- `Scheduler` queues updates, intervals, and timeouts.
- `AppWaker` wakes an idle application when work becomes ready.

## Basic use

Call hooks inside a component hook scope and in the same order on each render.
Use a signal setter or updater to change state. Register effect cleanup for
subscriptions or resources that must stop when the component leaves the tree.

## Behavior

A changed signal notifies its subscribers and registered wakers. Equal writes
do not request another redraw for comparable values. The scheduler processes
queued updates outside its internal lock, so callbacks may schedule more work.
Timers expose the next deadline to the application loop.

Hook slots are reused by call order. Effects clean up before replacement and
when their component scope is removed. Local hook scopes are separate from the
thread-safe hook types used by mounted components.

## Limits

- Hook order must remain stable across renders.
- `Signal` and its split handles have bounds that depend on the operations used.
- Timers run when the application loop processes scheduler work; they are not
  independent background executors.
- A state source must be connected to the application waker to redraw an idle
  application immediately.

## Source map

- Reactive exports: [`src/reactive/mod.rs`](../src/reactive/mod.rs)
- Signals: [`src/reactive/signal.rs`](../src/reactive/signal.rs)
- Effects and hook slots: [`src/reactive/effect.rs`](../src/reactive/effect.rs), [`src/reactive/hooks.rs`](../src/reactive/hooks.rs)
- Utility hook exports: [`src/hooks/mod.rs`](../src/hooks/mod.rs)
- Wake behavior tests: [`tests/app_wakeups.rs`](../tests/app_wakeups.rs)
- Hook lifecycle tests: [`tests/api_hook_lifecycle.rs`](../tests/api_hook_lifecycle.rs)

## Related chapters

- [Applications and components](app-and-components.md)
- [Animation and screens](animation-and-screens.md)
- [Images and clipboard](images-and-clipboard.md)

[Back to the manual](README.md)
