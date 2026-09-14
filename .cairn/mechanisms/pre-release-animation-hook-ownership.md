# Mechanism: pre-release-animation-hook-ownership

command: python3 -B scripts/check-pre-release-reactive-concurrency.py RAC-001
inputs:
  - Cargo.toml
  - Cargo.lock
  - src/animation
  - src/app.rs
  - src/hooks/animation.rs
  - src/reactive
  - scripts/check-pre-release-reactive-concurrency.py
requirements:
  - RAC-001

The check MUST run deterministic tests inside the animation hook module so the
tests can inspect owner identity, animation-runtime tasks, component-scheduler
timers, delivered updates, and cancellations without wall-clock polling.
Animation, spring, transition, and stagger hooks MUST retain one owner in each
hook slot across repeated renders. Closing the component scope MUST cancel its
runtime tasks and scheduler timers, and later frame or timer processing MUST
deliver no update through the closed owner.

A delayed transition MUST use the component scheduler. Repeated renders and
frame updates MUST keep one pending transition timer and MUST NOT grow the
process's operating-system thread count with frame count. Stagger delays MUST
appear in the same component scheduler so App can drive them without a manual
handle call. The check MUST use `syn` to reject `std::thread::spawn` and fresh
unscoped schedulers in these hook paths. Before the corrected cases, a
validator MUST reject controlled counter fixtures for a replaced owner, an
uncancelled update, per-frame thread growth, and undriven stagger work.
