//! Part of tests/review_native.rs.

use reactive_tui::animation::{
    stagger_from_position, AnimatedProperty, AnimatedValue, Animation, AnimationBuilder,
    AnimationId, AnimationManager, AnimationState, AnimationTimeline, EasingFunction, LoopMode,
    SpringConfig, StaggerBuilder, StaggerOrigin,
};
use std::{
    collections::HashSet,
    panic,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

const SECOND: Duration = Duration::from_secs(1);

/// A one-second linear opacity animation from 0 to 1.
fn linear_opacity(id: &str) -> AnimationBuilder {
    Animation::builder(id)
        .animate_property(AnimatedProperty::Opacity(0.0, 1.0))
        .duration(SECOND)
        .easing(EasingFunction::Linear)
}

fn opacity(animation: &Animation) -> f32 {
    match animation.get_current_values() {
        Some(AnimatedValue::Opacity(value)) => value,
        other => panic!("expected an opacity value, got {other:?}"),
    }
}

/// The doc comment lines right above the first `needle` in `source`.
fn docs_above<'a>(source: &'a str, needle: &str) -> String {
    let start = source
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} is not in the source"));
    // The slice ends inside the item's own line; the doc comment lines come
    // before that partial line.
    let mut lines: Vec<&'a str> = source[..start]
        .rsplit('\n')
        .skip(1)
        .take_while(|line| line.trim_start().starts_with("///"))
        .collect();
    lines.reverse();
    lines.join("\n")
}

#[test]
fn ani_001_a_reversed_animation_keeps_playing_back_from_where_it_is() {
    let updates = Arc::new(AtomicUsize::new(0));
    let counter = updates.clone();
    let mut animation = linear_opacity("ani-001")
        .on_update(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .build();
    animation.play();
    animation.update(Duration::from_millis(250));
    assert!(
        (animation.get_progress() - 0.25).abs() < 0.01,
        "ANI-001: progress after 250 ms was {}",
        animation.get_progress()
    );
    animation.reverse();
    let active = animation.update(Duration::from_millis(100));
    let progress = animation.get_progress();
    assert!(
        (progress - 0.15).abs() < 0.01,
        "ANI-001: reversed at 0.25 and updated by 100 ms, progress is {progress} instead of 0.15"
    );
    assert_eq!(
        updates.load(Ordering::SeqCst),
        2,
        "ANI-001: the update after reversing ran no on_update callback"
    );
    assert_eq!(
        animation.get_state(),
        AnimationState::Playing,
        "ANI-001: a reversed animation reports {:?} instead of Playing",
        animation.get_state()
    );
    assert_eq!(
        active,
        animation.is_playing(),
        "ANI-001: update returned {active} while is_playing is {}",
        animation.is_playing()
    );
}

#[test]
fn ani_002_a_parallel_timeline_waits_for_a_delayed_animation() {
    let mut timeline = AnimationTimeline::new("ani-002-parallel", false);
    timeline.add_animation(
        linear_opacity("delayed")
            .delay(Duration::from_secs(10))
            .build(),
    );
    timeline.play();
    let active = timeline.update(Duration::from_millis(16));
    assert_eq!(
        *timeline.state.read().unwrap(),
        AnimationState::Playing,
        "ANI-002: a parallel timeline whose only animation waits out its delay is not Playing"
    );
    assert!(
        active,
        "ANI-002: the parallel timeline's update said it was over during its animation's delay"
    );
}

#[test]
fn ani_002_a_sequential_timeline_starts_the_next_animation_in_the_completing_update() {
    let mut timeline = AnimationTimeline::new("ani-002-sequential", true);
    timeline.add_animation(linear_opacity("first").build());
    timeline.add_animation(linear_opacity("second").build());
    timeline.play();
    timeline.update(SECOND);
    timeline.update(Duration::from_millis(100));
    let second = timeline.animations[1].get_progress();
    assert!(
        (second - 0.1).abs() < 0.01,
        "ANI-002: 100 ms after the first animation completed, the second is at progress {second} instead of 0.1"
    );
}

#[test]
fn ani_002_update_reports_whether_the_animation_is_still_active() {
    let mut done = linear_opacity("ani-002-done").build();
    done.play();
    assert!(
        !done.update(SECOND),
        "ANI-002: the update that completed the animation returned true"
    );
    assert!(
        done.is_completed(),
        "ANI-002: a full-duration update did not complete the animation"
    );

    let mut delayed = linear_opacity("ani-002-delayed")
        .delay(Duration::from_secs(10))
        .build();
    delayed.play();
    assert!(
        delayed.update(Duration::from_millis(16)),
        "ANI-002: an update during the delay returned false"
    );

    let mut paused = linear_opacity("ani-002-paused").build();
    paused.play();
    paused.update(Duration::from_millis(100));
    paused.pause();
    assert!(
        !paused.update(Duration::from_millis(100)),
        "ANI-002: an update while paused returned true"
    );
}

#[test]
fn ani_003_loops_run_their_callback_and_auto_reverse_alternates() {
    let loops = Arc::new(AtomicUsize::new(0));
    let counter = loops.clone();
    let mut animation = linear_opacity("ani-003-count")
        .loop_mode(LoopMode::Count(2))
        .auto_reverse(true)
        .on_loop(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .build();
    animation.play();
    animation.update(SECOND);
    assert_eq!(
        loops.load(Ordering::SeqCst),
        1,
        "ANI-003: after the first pass ended and the second began, on_loop ran {} times",
        loops.load(Ordering::SeqCst)
    );
    assert!(
        animation.state.read().unwrap().is_reversed,
        "ANI-003: the second pass of an auto_reverse animation is not reversed"
    );
    animation.update(Duration::from_millis(250));
    let value = opacity(&animation);
    assert!(
        (value - 0.75).abs() < 0.01,
        "ANI-003: a quarter into the reversed second pass, opacity is {value} instead of 0.75"
    );
}

#[test]
fn ani_003_an_infinite_loop_runs_its_callback_every_pass() {
    let loops = Arc::new(AtomicUsize::new(0));
    let counter = loops.clone();
    let mut animation = linear_opacity("ani-003-infinite")
        .loop_mode(LoopMode::Infinite)
        .on_loop(move |_, _| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .build();
    animation.play();
    for _ in 0..3 {
        animation.update(SECOND);
    }
    assert_eq!(
        loops.load(Ordering::SeqCst),
        3,
        "ANI-003: three passes of an infinite loop ran on_loop {} times",
        loops.load(Ordering::SeqCst)
    );
    assert!(
        animation.is_playing(),
        "ANI-003: an infinite loop stopped playing"
    );
}

#[test]
fn ani_003_count_three_plays_three_passes() {
    let mut animation = linear_opacity("ani-003-three")
        .loop_mode(LoopMode::Count(3))
        .build();
    animation.play();
    animation.update(SECOND);
    animation.update(SECOND);
    assert!(
        !animation.is_completed(),
        "ANI-003: Count(3) completed after two passes"
    );
    animation.update(SECOND);
    assert!(
        animation.is_completed(),
        "ANI-003: Count(3) was not completed after three passes"
    );
    let passes = animation.state.read().unwrap().loops_completed;
    assert_eq!(
        passes, 3,
        "ANI-003: Count(3) counted {passes} passes after three"
    );
    animation.update(SECOND);
    let passes = animation.state.read().unwrap().loops_completed;
    assert_eq!(passes, 3, "ANI-003: Count(3) played a fourth pass");
}

#[test]
fn ani_003_count_zero_plays_no_pass() {
    let completions = Arc::new(AtomicUsize::new(0));
    let counter = completions.clone();
    let mut animation = linear_opacity("ani-003-zero")
        .loop_mode(LoopMode::Count(0))
        .on_complete(move |_| {
            counter.fetch_add(1, Ordering::SeqCst);
        })
        .build();
    animation.play();
    let active = animation.update(Duration::from_millis(250));
    assert!(
        !active && animation.is_completed(),
        "ANI-003: Count(0) was still active after its first update: it played a pass"
    );
    let passes = animation.state.read().unwrap().loops_completed;
    assert_eq!(passes, 0, "ANI-003: Count(0) counted {passes} passes");
    assert!(
        animation.get_current_values().is_none(),
        "ANI-003: Count(0) sampled a pass: {:?}",
        animation.get_current_values()
    );
    assert_eq!(
        completions.load(Ordering::SeqCst),
        1,
        "ANI-003: on_complete ran {} times for Count(0)",
        completions.load(Ordering::SeqCst)
    );
}

#[test]
fn ani_003_the_module_keeps_one_completion_path() {
    let source = include_str!("../../src/animation/mod.rs");
    for helper in ["fn handle_animation_complete(", "fn restart_animation("] {
        assert!(
            !source.contains(helper),
            "ANI-003: src/animation/mod.rs still defines the unused {helper}"
        );
    }
}

#[test]
fn ani_004_stale_cleanup_keeps_an_animation_that_is_advancing() {
    let threshold = Duration::from_millis(500);
    let mut manager = AnimationManager::new();
    let mut animation = linear_opacity("ani-004-long")
        .duration(Duration::from_secs(10))
        .build();
    animation.play();
    let id: AnimationId = manager.add_animation(animation);
    thread::sleep(Duration::from_millis(600));
    manager.update();
    let removed = manager.cleanup_all_stale(threshold);
    assert_eq!(
        removed, 0,
        "ANI-004: cleanup removed {removed} animation(s) that an update had just advanced"
    );
    assert!(
        manager.get_animation(&id).is_some(),
        "ANI-004: the advancing animation is gone from the manager"
    );
}

#[test]
fn ani_004_stale_cleanup_keeps_a_paused_animation() {
    let mut manager = AnimationManager::new();
    let mut animation = linear_opacity("ani-004-paused")
        .duration(Duration::from_secs(10))
        .build();
    animation.play();
    animation.update(Duration::from_millis(100));
    animation.pause();
    let id: AnimationId = manager.add_animation(animation);
    thread::sleep(Duration::from_millis(600));
    let removed = manager.cleanup_all_stale(Duration::from_millis(500));
    assert_eq!(
        removed, 0,
        "ANI-004: cleanup removed a paused animation as stale"
    );
    assert!(
        manager.get_animation(&id).is_some(),
        "ANI-004: the paused animation is gone from the manager"
    );
}

#[test]
fn ani_004_cleanup_completed_documents_what_it_removes() {
    let docs = docs_above(
        include_str!("../../src/animation/mod.rs"),
        "pub fn cleanup_completed(",
    );
    assert!(
        !docs.contains("failed") && !docs.contains("stuck"),
        "ANI-004: cleanup_completed's doc comment still promises to remove failed or stuck animations: {docs}"
    );
}

#[test]
fn ani_005_generated_ids_are_unique_within_a_millisecond() {
    let ids: HashSet<AnimationId> = (0..2000)
        .map(|_| Animation::new(SECOND, EasingFunction::Linear, None, LoopMode::None).id)
        .collect();
    assert_eq!(
        ids.len(),
        2000,
        "ANI-005: 2000 animations made back to back have only {} distinct ids",
        ids.len()
    );
    let mut manager = AnimationManager::new();
    manager.add_animation(Animation::new(
        SECOND,
        EasingFunction::Linear,
        None,
        LoopMode::None,
    ));
    manager.add_animation(Animation::new(
        SECOND,
        EasingFunction::Linear,
        None,
        LoopMode::None,
    ));
    assert_eq!(
        manager.active_count(),
        2,
        "ANI-005: two animations made back to back count as {} in a manager",
        manager.active_count()
    );
}

#[test]
fn ani_005_add_animation_documents_a_repeated_id() {
    let docs = docs_above(
        include_str!("../../src/animation/mod.rs"),
        "pub fn add_animation(&mut self, animation: Animation) -> AnimationId",
    );
    assert!(
        docs.contains("replaces"),
        "ANI-005: add_animation's doc comment does not say that a repeated id replaces the earlier animation: {docs}"
    );
}

#[test]
fn ani_006_the_alternate_drivers_are_gone() {
    // The two files are named in parts: a path that must not exist is not a
    // reference the dangling-paths gate (BAR-007) should find here.
    let module = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("animation");
    for file in ["performance.rs", "lock_free.rs"] {
        assert!(
            !module.join(file).exists(),
            "ANI-006: src/animation still holds {file}"
        );
    }
    let source = include_str!("../../src/animation/mod.rs");
    for name in [
        "mod performance",
        "mod lock_free",
        "performance::",
        "lock_free::",
        "OptimizedAnimationManager",
        "InterpolationCache",
        "AnimationBatch",
        "LockFreeAnimationState",
    ] {
        assert!(
            !source.contains(name),
            "ANI-006: src/animation/mod.rs still names {name}"
        );
    }
    let changelog = include_str!("../../CHANGELOG.md");
    for name in [
        "OptimizedAnimationManager",
        "InterpolationCache",
        "LockFreeAnimationState",
    ] {
        assert!(
            changelog.contains(name),
            "ANI-006: CHANGELOG.md does not name the removed {name}"
        );
    }
}

#[test]
fn ani_007_stagger_delays_never_overflow() {
    let grid = panic::catch_unwind(|| {
        StaggerBuilder::new(100)
            .from(StaggerOrigin::Position(0, 0))
            .grid(200, 1)
            .build()
            .calculate_grid_delays(200, 1)
    });
    let delays = grid.unwrap_or_else(|_| {
        panic!("ANI-007: a 200 by 1 grid stagger from position (0, 0) panicked")
    });
    assert_eq!(
        delays.len(),
        200,
        "ANI-007: the 200-cell grid got {} delays",
        delays.len()
    );
    let far = delays[182].as_secs_f32();
    assert!(
        (far - 1.82).abs() < 0.01,
        "ANI-007: element 182 of the grid got {far} s instead of 1.82 s"
    );
    let position =
        panic::catch_unwind(|| stagger_from_position(100, 0, 0).calculate_delays(1, &[(182, 0)]));
    assert!(
        position.is_ok(),
        "ANI-007: a position stagger to (182, 0) panicked"
    );
    let range = panic::catch_unwind(|| {
        StaggerBuilder::new(100)
            .range(-1.0, -1.0)
            .build()
            .calculate_delays(1, &[])
    });
    assert!(
        range.is_ok(),
        "ANI-007: a stagger with a negative range panicked in calculate_delays"
    );
}

#[test]
fn ani_008_the_configured_velocity_is_the_positions_initial_slope() {
    for damping in [10.0_f32, 20.0, 50.0] {
        let spring = SpringConfig::new(1.0, 100.0, damping).with_velocity(1.0);
        let slope = (spring.calculate_position(0.001, 0.0, 1.0)
            - spring.calculate_position(0.0, 0.0, 1.0))
            / 0.001;
        let velocity = spring.calculate_velocity(0.0, 0.0, 1.0);
        assert!(
            (slope - velocity).abs() < 0.1,
            "ANI-008: damping {damping}: the position's initial slope {slope} disagrees with calculate_velocity {velocity}"
        );
        let still = SpringConfig::new(1.0, 100.0, damping);
        let still_slope = (still.calculate_position(0.001, 0.0, 1.0)
            - still.calculate_position(0.0, 0.0, 1.0))
            / 0.001;
        assert!(
            slope > still_slope,
            "ANI-008: damping {damping}: a positive velocity did not move the spring toward its target faster ({slope} against {still_slope} at rest)"
        );
    }
}

#[test]
fn ani_009_a_spring_eased_animation_ends_at_its_target() {
    let config = SpringConfig::new(1.0, 1.0, 2.0);
    let eased = EasingFunction::Spring(config.clone()).apply(1.0);
    assert_eq!(
        eased, 1.0,
        "ANI-009: spring easing at full progress gives {eased} instead of 1"
    );
    let mut animation = Animation::spring(Duration::from_secs(10), config);
    animation.play();
    animation.update(Duration::from_secs(10));
    assert!(
        animation.is_completed(),
        "ANI-009: a ten-second spring animation was not completed after ten seconds"
    );
    let value = opacity(&animation);
    assert!(
        (value - 1.0).abs() < 0.001,
        "ANI-009: the spring animation completed at opacity {value} instead of 1"
    );
}
#[test]
fn ani_008_a_positive_velocity_moves_a_descending_spring_toward_its_target() {
    for damping in [10.0_f32, 20.0, 50.0] {
        let spring = SpringConfig::new(1.0, 100.0, damping).with_velocity(1.0);
        let still = SpringConfig::new(1.0, 100.0, damping);
        let moved = spring.calculate_position(0.001, 1.0, 0.0);
        let rested = still.calculate_position(0.001, 1.0, 0.0);
        assert!(
            moved < rested,
            "ANI-008: damping {damping}: from 1 to 0, a positive velocity left the spring at {moved}, farther from 0 than {rested} at rest"
        );
        let slope = (moved - spring.calculate_position(0.0, 1.0, 0.0)) / 0.001;
        let velocity = spring.calculate_velocity(0.0, 1.0, 0.0);
        assert!(
            (slope - velocity).abs() < 0.1,
            "ANI-008: damping {damping}: from 1 to 0, the position's initial slope {slope} disagrees with calculate_velocity {velocity}"
        );
    }
}

#[test]
fn ani_008_a_displacement_below_precision_still_moves_by_its_velocity() {
    for damping in [10.0_f32, 20.0, 50.0] {
        let spring = SpringConfig::new(1.0, 100.0, damping).with_velocity(1.0);
        let (from, to) = (0.0, 0.005);
        let early = spring.calculate_position(0.0005, from, to);
        assert!(
            early > 0.0 && early < to,
            "ANI-008: damping {damping}: half a millisecond into a displacement of 0.005, the position is {early}, not between 0 and 0.005"
        );
        let slope = (spring.calculate_position(0.001, from, to)
            - spring.calculate_position(0.0, from, to))
            / 0.001;
        let velocity = spring.calculate_velocity(0.0, from, to);
        assert!(
            (slope - velocity).abs() < 0.1,
            "ANI-008: damping {damping}: over a displacement of 0.005 the position's initial slope {slope} disagrees with calculate_velocity {velocity}"
        );
    }
}

#[test]
fn ani_009_the_spring_has_settled_at_full_progress() {
    for config in [
        SpringConfig::new(1.0, 1.0, 50.0),
        SpringConfig::new(1.0, 1.0, 2.0),
        SpringConfig::new(1.0, 100.0, 10.0),
        SpringConfig::new(1.0, 100.0, 50.0),
    ] {
        let settle = config.estimate_duration(0.0, 1.0);
        assert!(
            config.is_settled(settle, 0.0, 1.0),
            "ANI-009: {config:?}: at its estimated settle time of {settle} s the spring is at {} moving at {}, not settled",
            config.calculate_position(settle, 0.0, 1.0),
            config.calculate_velocity(settle, 0.0, 1.0)
        );
        let near_end = EasingFunction::Spring(config.clone()).apply(0.999);
        assert!(
            (near_end - 1.0).abs() < 0.02,
            "ANI-009: {config:?}: spring easing at progress 0.999 is {near_end}, so the last frame jumps to 1"
        );
    }
}
