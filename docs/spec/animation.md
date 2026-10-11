# Animations

Prefix: ANI

The animation module (src/animation) is the duration-based motion an
application drives itself or hands to its App: an `Animation` with a
property, a duration, an easing, a delay, a loop mode and callbacks
(src/animation/mod.rs), built directly or through `AnimationBuilder` and
the `api` constructors; an `AnimationTimeline` that plays animations in
sequence or in parallel; and the `AnimationManager` that the App updates
every frame from its monotonic clock (src/app.rs) and that the C interface
exposes (src/ffi/animation.rs). Beside them the module keeps the stagger
delays (src/animation/stagger.rs), the spring physics (src/animation/spring.rs,
whose `step` the `use_spring` hook uses and whose analytic position and
velocity the `EasingFunction::Spring` easing uses, src/animation/easing.rs),
and two alternate drivers nothing in the crate uses: the optimized batch,
its interpolation cache and metrics (the `animation::performance` module) and
the lock-free state (the `animation::lock_free` module), both removed by
ANI-006. The hook animations
(`use_animation`, `use_spring`, `use_stagger`) have their own frame driver,
covered by reactive.md.

Read on 2026-10-04 from the developer's production code review of 65e618ec
(findings N03 to N11, N23 and N24) and checked against the code on
2026-10-05: reversing a playing animation stops its updates while it still
reports playing; a parallel timeline completes while its animations wait
out their delay, and `update` returns true on the frame that completes an
animation; the loop callback and `auto_reverse` live in helpers nothing
calls; stale cleanup removes an animation that is updated every frame once
the threshold has passed since it started; two animations made in one
millisecond share an id and the second replaces the first in a manager; the
interpolation cache answers for other endpoints, the optimized batch
returns progress for opacity and the destination for a transform, and the
lock-free state loses concurrent updates and cannot finish a loop count
above 255; a stagger over a 200-cell grid overflows; a spring's configured
velocity moves its position the wrong way at first; and an animation with
spring easing completes a quarter of the way to its target.

## Observed

(none yet)

## Draft

[ANI-001] An `Animation` reversed while it plays MUST keep playing: `reverse` MUST flip its direction and keep its state Playing, the next `update` MUST continue from the progress the animation had back toward its start, its `on_update` callbacks MUST keep running, and `is_playing`, `get_state` and `update` MUST agree on whether the animation plays.
Falsifier: A one-second linear opacity animation played, updated by 250 ms, reversed and updated by 100 ms reports a progress other than 0.15 (within 0.01), has run no second `on_update`, or has `is_playing` true while that `update` returned false.
Mechanism: review-native
Rationale: `reverse` set the state Reversed, which `update` refused and `is_playing` accepted, so a reversed animation stopped moving while reporting that it played (the developer's code review of 2026-10-04, N03).
Status: Agreed 2026-10-05

[ANI-002] A parallel `AnimationTimeline` MUST stay Playing until every animation in it has completed, one still waiting out its delay included; a sequential timeline MUST start its next animation in the update that completes the current one; and `Animation::update` MUST return whether the animation is still active after the update: true while it plays or waits out its delay, false from the update that completes it and whenever it is paused or stopped.
Falsifier: A parallel timeline holding one animation with a ten-second delay is Completed after an update made before the delay ends; a sequential timeline of two one-second animations updated by one second and then by 100 ms has not advanced the second to progress 0.1; `update` returns true from the update that completes a one-second animation; or `update` returns false while the animation waits out its delay.
Mechanism: review-native
Rationale: The parallel timeline took a child's false, which a delay also returns, for completion, and `update` returned true on the frame that completed the animation, so a sequential timeline needed one update more (N04).
Status: Agreed 2026-10-05

[ANI-003] An `Animation` that loops MUST run its `on_loop` callback each time a pass ends and another begins, with the number of passes completed, under `Infinite`, `Count` and `PingPong` alike; with `auto_reverse` set, the pass that begins MUST run in the direction opposite to the one that ended, under `Infinite` and `Count` as under `PingPong`; `Count(n)` MUST play n passes; the callbacks MUST run with no lock of the animation held (SIG-002); and the module MUST have one completion path, the unused helpers removed.
Falsifier: A one-second linear opacity animation with `Count(2)`, `auto_reverse(true)` and an `on_loop` counter, played and updated by one second, has a counter other than 1 or is not reversed, or updated by 250 ms more samples an opacity other than 0.75 (within 0.01); an `Infinite` animation updated through three passes ran `on_loop` fewer than three times; `Count(3)` completes before its third pass ends or plays a fourth; or src/animation/mod.rs still defines `handle_animation_complete` or `restart_animation`.
Mechanism: review-native
Rationale: The live completion branch advanced loops without the loop callback or `auto_reverse`, which lived in two helpers marked dead code (N05).
Status: Agreed 2026-10-05

[ANI-004] `AnimationManager::cleanup_all_stale(threshold)` MUST remove an animation only when it has completed, or when it plays and no `update` has advanced it within the threshold; an animation advanced within the threshold MUST stay however long ago it started, and one paused or waiting out its delay MUST stay; `cleanup_completed` MUST remove completed animations and timelines and timelines whose lock is poisoned, and its documentation MUST say that and no more.
Falsifier: A ten-second animation played, advanced by `update` after more than a 20 ms threshold has passed since it started, then cleaned up at once with that threshold, is removed; a paused animation is removed as stale; or `cleanup_completed`'s doc comment still promises to remove failed or stuck animations.
Mechanism: review-native
Rationale: The cleanup compared the clock with the start time and a frame time that `update` never refreshed, so a long or infinite animation updated every frame was removed once the threshold had passed since it started (N06).
Status: Agreed 2026-10-05

[ANI-005] Every `Animation` the crate names itself (`Animation::new`, `Animation::spring` and the `api` constructors) MUST get an id that no other animation named by the crate in the process has, drawn from one sequence; `AnimationManager::add_animation` MUST keep two animations with different ids, and its documentation MUST say that an animation added under an id already present replaces the earlier one.
Falsifier: Two animations made by `Animation::new` within one millisecond have the same id, or added to one manager leave `active_count` at 1; or `add_animation`'s doc comment does not say what a repeated id does.
Mechanism: review-native
Rationale: `Animation::new` named the animation after the elapsed millisecond, so two made in one millisecond shared an id and the second silently replaced the first in a manager (N07).
Status: Agreed 2026-10-05

[ANI-006] The crate MUST NOT ship the alternate drivers `animation::performance` (`AnimationBatch`, `BatchedUpdate`, `CacheStats`, `InterpolationCache`, `OptimizationLevel`, `OptimizedAnimationManager`, `PerformanceMetrics`, `PerformanceReport`) and `animation::lock_free` (`LockFreeAnimationState`, `LockFreeAnimationUpdater`): animations are driven by `Animation::update`, `AnimationTimeline` and `AnimationManager`, and the changelog MUST name the removal and the names that went.
Falsifier: A source file for the module `animation::performance` or `animation::lock_free` exists in the animation module's directory, `reactive_tui::animation` exports one of those names, or CHANGELOG.md does not record their removal.
Mechanism: review-native
Rationale: The optimized batch returned progress for an opacity and the destination for a transform, the interpolation cache answered for other endpoints, and the lock-free state lost concurrent updates and could not finish a loop count above 255; nothing in the crate, the C interface or the manual used them (N08, N09, N10).
Status: Agreed 2026-10-06

[ANI-007] Every stagger delay MUST be a finite, non-negative duration for any grid up to 32,767 cells a side and any element position within `i16`: distances are computed in a type that cannot overflow, and an eased or ranged delay that is negative or not finite is clamped to zero.
Falsifier: `StaggerBuilder::new(100).from(StaggerOrigin::Position(0, 0)).grid(200, 1).build().calculate_grid_delays(200, 1)` panics, returns fewer than 200 delays, or gives element 182 a delay other than 1.82 s (within 10 ms); `stagger_from_position(100, 0, 0).calculate_delays(1, &[(182, 0)])` panics; or a stagger built with `range(-1.0, -1.0)` panics in `calculate_delays`.
Mechanism: review-native
Rationale: The grid and position distances squared `i16` coordinates, so a 200-cell grid overflowed: a panic in debug builds, and in release a negative square whose root is NaN, which `Duration::from_secs_f32` refuses (N11).
Status: Agreed 2026-10-05

[ANI-008] `SpringConfig::calculate_velocity` MUST be the time derivative of `calculate_position`, with the configured `velocity` the position's initial rate of change from `from` toward `to`, in every damping regime: just after time zero a spring with a positive configured velocity MUST be closer to `to` than the same spring with none.
Falsifier: For `SpringConfig::new(1.0, 100.0, d).with_velocity(1.0)` with d of 10.0, 20.0 and 50.0, from 0 to 1, `(calculate_position(0.001, 0.0, 1.0) - calculate_position(0.0, 0.0, 1.0)) / 0.001` differs from `calculate_velocity(0.0, 0.0, 1.0)` by more than 0.1, or is not above the same slope of the spring with velocity 0.
Mechanism: review-native
Rationale: The response formulas put the configured velocity on the remaining displacement, so a positive velocity first moved the position away from `to` while `calculate_velocity` reported it positive (N23).
Status: Agreed 2026-10-05

[ANI-009] An `Animation` whose easing is `EasingFunction::Spring` MUST end at its target: the spring's time MUST run over the animation's duration so that at full progress the spring has settled, the eased value at progress 1 MUST be exactly 1, and the update that completes the animation MUST leave `get_current_values` at the `to` value.
Falsifier: `Animation::spring(Duration::from_secs(10), SpringConfig::new(1.0, 1.0, 2.0))` played and updated by ten seconds is Completed with an opacity farther than 0.001 from 1; or `EasingFunction::Spring(SpringConfig::new(1.0, 1.0, 2.0)).apply(1.0)` is not 1.
Mechanism: review-native
Rationale: The spring easing read the normalized progress as seconds of spring time, so a critically damped spring completed its ten-second animation at 26% of the way to its target (N24).
Status: Agreed 2026-10-05

[ANI-010] `DebugAnimationManager` MUST instrument what it runs: each `update` MUST record the frame's time in its performance metrics and log an update event for every animation that advanced, taking a snapshot of each at `Verbose` verbosity, with performance collection independent of state logging; and its documentation MUST say what is logged automatically and what a caller logs by hand.
Falsifier: A `Verbose` manager with performance monitoring on and console logging off, holding one playing animation, has after one `update` no `AnimationUpdated` event, a single snapshot, or a minimum frame time of `Duration::MAX`; `create_performance_debug_manager`'s manager records no timing after an `update`; or src/animation/debug.rs still promises callback wrapping.
Mechanism: review-facade
Rationale: `update` measured a frame time it never recorded, `wrap_animation_callbacks` installed nothing, the verbose snapshot branch was empty, and the performance preset disabled the state logging that timing collection depended on (the developer's code review of 2026-10-04, N16, what remains after ANI-006).
Status: Agreed 2026-10-06
