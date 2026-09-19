use reactive_tui::graphics::{GpuCubeRenderer, GraphicsOptions, HybridCubeRenderer};
use std::time::Duration;

#[test]
fn stage_timings_include_gpu_completion_and_cpu_has_no_readback() {
    let gpu = GpuCubeRenderer::new().expect("real adapter required");
    assert!(gpu.adapter_info().is_hardware);
    let mut cpu = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        fault: None,
        ..Default::default()
    });
    for (columns, rows) in [(60, 24), (144, 50), (200, 60)] {
        let hardware = gpu
            .render_terminal(columns, rows, Duration::from_secs(1))
            .unwrap();
        assert!(!hardware.timings().render.is_zero());
        assert!(!hardware.timings().readback.is_zero());
        let fallback = cpu
            .render_terminal(columns, rows, Duration::from_secs(1))
            .unwrap();
        assert!(!fallback.timings().render.is_zero());
        assert!(fallback.timings().readback.is_zero());
        assert_eq!(hardware.pixels().len(), fallback.pixels().len());
    }
}
