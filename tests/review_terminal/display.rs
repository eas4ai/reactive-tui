//! Part of tests/review_terminal.rs: the adaptive frame-rate manager.

use reactive_tui::display::monitor::PerformanceMode;
use reactive_tui::display::{AdaptiveConfig, AdaptiveFpsManager};
use std::time::Duration;

fn bounds(min_fps: u32, max_fps: u32, mode: PerformanceMode) -> AdaptiveConfig {
    AdaptiveConfig {
        min_fps,
        max_fps,
        mode,
        auto_adapt: true,
        ..AdaptiveConfig::default()
    }
}

/// PLT-015: every target stays within the configured bounds, through
/// construction, the Auto mode and an adaptive reduction, which never panics.
#[test]
fn plt_015_frame_rate_targets_stay_within_the_configured_bounds() {
    let manager = AdaptiveFpsManager::with_config(bounds(30, 30, PerformanceMode::Balanced));
    assert_eq!(
        manager.get_target_fps(),
        30,
        "PLT-015: Balanced within [30, 30] targets {}",
        manager.get_target_fps()
    );
    let manager = AdaptiveFpsManager::with_config(bounds(100, 100, PerformanceMode::Balanced));
    assert_eq!(
        manager.get_target_fps(),
        100,
        "PLT-015: Balanced within [100, 100] targets {}",
        manager.get_target_fps()
    );

    let mut manager = AdaptiveFpsManager::with_config(bounds(145, 145, PerformanceMode::Balanced));
    manager.set_performance_mode(PerformanceMode::Auto);
    assert_eq!(
        manager.get_target_fps(),
        145,
        "PLT-015: Auto within [145, 145] targets {}",
        manager.get_target_fps()
    );
    // Past the adjustment cooldown, dropped frames ask for a reduction, which
    // must clamp to the bounds rather than panic on a reversed interval.
    std::thread::sleep(Duration::from_millis(2100));
    for _ in 0..30 {
        manager.record_frame_performance(
            Duration::from_millis(50),
            Duration::from_millis(45),
            true,
        );
    }
    assert_eq!(
        manager.get_target_fps(),
        145,
        "PLT-015: dropped frames moved the target to {} with bounds [145, 145]",
        manager.get_target_fps()
    );
}
