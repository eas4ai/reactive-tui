use reactive_tui::{
    core::surface::Rgba,
    display::{
        adaptive::{AdaptiveConfig, AdaptiveFpsManager, AsyncAdaptiveFpsManager},
        monitor::PerformanceMode,
    },
};
use std::time::Duration;

#[test]
fn async_fps_preserves_state_without_a_tokio_runtime() {
    futures_lite::future::block_on(async {
        let config = AdaptiveConfig::default();
        let mut sync = AdaptiveFpsManager::with_config(config.clone());
        let asynchronous = AsyncAdaptiveFpsManager::with_config(config);
        for mode in [PerformanceMode::PowerSave, PerformanceMode::Balanced] {
            sync.set_performance_mode(mode);
            asynchronous.set_performance_mode(mode).await;
            assert_eq!(asynchronous.get_target_fps().await, sync.get_target_fps());
            assert_eq!(
                asynchronous.get_frame_duration().await,
                sync.get_frame_duration()
            );
            assert!(asynchronous
                .get_recommendation_summary()
                .await
                .contains("Current Target:"));
        }
        asynchronous
            .record_frame_performance(Duration::from_millis(16), Duration::from_millis(2), false)
            .await;
        let _ = asynchronous.get_performance_metrics().await;
    });
}

#[test]
fn fast_color_paths_preserve_scalar_results() {
    let first = Rgba::new(0.2, 0.4, 0.6, 0.5);
    let second = Rgba::new(0.8, 0.2, 0.4, 1.0);
    assert!(first.equals_epsilon_fast(first, 0.001));
    assert!(!first.equals_epsilon_fast(second, 0.001));
    let blended = first.blend_fast(second, 0.5);
    for (actual, expected) in [
        (blended.r, 0.5),
        (blended.g, 0.3),
        (blended.b, 0.5),
        (blended.a, 0.75),
    ] {
        assert!((actual - expected).abs() < 0.00001);
    }
}

#[cfg(feature = "simd")]
#[test]
fn simd_alpha_operations_preserve_channels() {
    let color = Rgba::new(0.2, 0.4, 0.6, 0.5);
    let alpha = color.with_alpha_simd(0.5);
    assert_eq!((alpha.r, alpha.g, alpha.b, alpha.a), (0.2, 0.4, 0.6, 0.25));
    let premultiplied = color.premultiply_alpha_simd();
    assert_eq!(
        (
            premultiplied.r,
            premultiplied.g,
            premultiplied.b,
            premultiplied.a
        ),
        (0.1, 0.2, 0.3, 0.5)
    );
}
