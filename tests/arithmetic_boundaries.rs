use reactive_tui::component::Element;
use reactive_tui::display::monitor::PerformanceMode;
use reactive_tui::display::{AdaptiveConfig, AdaptiveFpsManager};
use reactive_tui::layout::grid::DeclarativeGrid;
use std::panic::{catch_unwind, AssertUnwindSafe};

fn config(min_fps: u32, max_fps: u32) -> AdaptiveConfig {
    AdaptiveConfig {
        auto_adapt: true,
        min_fps,
        max_fps,
        quality_preference: 0.5,
        power_save: false,
        mode: PerformanceMode::Auto,
    }
}

#[test]
fn adaptive_fps_bounds_never_reach_zero_or_panic() {
    let bounds = [0, 1, 30, 144, u32::MAX];
    for min_fps in bounds {
        for max_fps in bounds {
            let mut manager = catch_unwind(AssertUnwindSafe(|| {
                AdaptiveFpsManager::with_config(config(min_fps, max_fps))
            }))
            .unwrap_or_else(|_| panic!("adaptive bounds panicked: {min_fps}..={max_fps}"));

            assert!(manager.get_target_fps() > 0);
            assert!(!manager.get_frame_duration().is_zero());

            manager.set_target_fps(0);
            assert!(manager.get_target_fps() > 0);
            assert!(!manager.get_frame_duration().is_zero());
        }
    }
    println!("RTR004 PASS adaptive");
}

#[test]
fn zero_column_auto_grid_is_always_empty_and_safe() {
    for rows in 0..=4 {
        for item_count in 0..=8 {
            let items = (0..item_count)
                .map(|index| Element::text(format!("item-{index}")))
                .collect();
            let grid = catch_unwind(AssertUnwindSafe(|| {
                DeclarativeGrid::auto_grid(0, rows, items)
            }))
            .unwrap_or_else(|_| {
                panic!("zero-column grid panicked: rows={rows}, items={item_count}")
            });
            assert_eq!(grid.cols, 0);
            assert!(grid.children.is_empty());
        }
    }
    println!("RTR004 PASS grid");
}
