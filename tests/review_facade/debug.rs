//! Part of tests/review_facade.rs: ANI-010, the animation debugger
//! instruments what it runs.

use reactive_tui::animation::debug::{
    create_performance_debug_manager, DebugAnimationManager, DebugConfig, DebugEvent,
    DebugVerbosity,
};
use reactive_tui::animation::{AnimatedProperty, Animation, EasingFunction};
use std::time::Duration;

/// A ten-second linear opacity animation, playing.
fn playing(id: &str) -> Animation {
    let mut animation = Animation::builder(id)
        .animate_property(AnimatedProperty::Opacity(0.0, 1.0))
        .duration(Duration::from_secs(10))
        .easing(EasingFunction::Linear)
        .build();
    animation.play();
    animation
}

/// ANI-010: one `update` of a verbose manager logs the update, takes a
/// snapshot and records the frame's time.
#[test]
fn ani_010_update_logs_what_it_runs() {
    let config = DebugConfig {
        verbosity_level: DebugVerbosity::Verbose,
        enable_performance_monitoring: true,
        enable_state_logging: true,
        log_to_console: false,
        ..Default::default()
    };
    let mut manager = DebugAnimationManager::new(config);
    manager.add_animation(playing("fade"));
    std::thread::sleep(Duration::from_millis(5));
    manager.update();
    let debugger = manager.debugger();
    let updates = debugger
        .get_events()
        .iter()
        .filter(|event| matches!(event, DebugEvent::AnimationUpdated { .. }))
        .count();
    let snapshots = debugger
        .get_animation_snapshots(&"fade".to_string())
        .map_or(0, |snapshots| snapshots.len());
    let min_frame = debugger.get_performance_metrics().min_frame_time;
    assert!(
        updates >= 1 && snapshots >= 2 && min_frame != Duration::MAX,
        "ANI-010: after one update of a verbose manager with a playing animation: {updates} update events, {snapshots} snapshots, minimum frame time {min_frame:?}"
    );
}

/// ANI-010: the performance preset records timing without state logging.
#[test]
fn ani_010_the_performance_preset_records_timing() {
    let mut manager = create_performance_debug_manager();
    manager.add_animation(playing("fade"));
    std::thread::sleep(Duration::from_millis(5));
    manager.update();
    let metrics = manager.debugger().get_performance_metrics();
    assert!(
        metrics.min_frame_time != Duration::MAX,
        "ANI-010: the performance preset recorded no frame time after an update: {metrics:?}"
    );
}

/// ANI-010: the module no longer promises callback wrapping.
#[test]
fn ani_010_the_module_promises_no_callback_wrapping() {
    let source = include_str!("../../src/animation/debug.rs");
    assert!(
        !source.contains("wrap_animation_callbacks"),
        "ANI-010: src/animation/debug.rs still has wrap_animation_callbacks, an empty promise"
    );
}
