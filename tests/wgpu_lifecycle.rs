use reactive_tui::graphics::{
    FrameRequest, GraphicsCanvas, GraphicsEffect, GraphicsError, GraphicsFault, GraphicsMode,
    GraphicsOptions, GraphicsWorker, HybridCubeRenderer,
};
use std::{collections::HashSet, sync::mpsc, time::Duration};

#[test]
fn software_adapter_label_cannot_claim_hardware_gpu_mode() {
    let mode = GraphicsMode::Gpu(reactive_tui::graphics::GraphicsAdapterInfo {
        name: "software fixture (no rendering acceptance)".into(),
        backend: "Vulkan".into(),
        is_hardware: false,
    });
    assert!(mode.label().starts_with("Software wgpu ·"));
    assert!(!mode.label().starts_with("GPU ·"));
}

#[test]
fn canvas_discards_stale_viewport_output_and_owns_shutdown() {
    let wake = reactive_tui::app::AppWaker::new();
    let mut canvas = GraphicsCanvas::new(
        wake.clone(),
        GraphicsOptions {
            force_cpu: true,
            ..Default::default()
        },
    );
    canvas.advance(Duration::ZERO, 60, 24).unwrap();
    wake.wait(Some(Duration::from_secs(2)));
    canvas.advance(Duration::from_millis(49), 144, 50).unwrap();
    assert!(
        canvas.frame().is_none(),
        "old viewport frame must not survive resize"
    );
    canvas.advance(Duration::from_millis(50), 144, 50).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while canvas.frame().is_none() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
        canvas.advance(Duration::from_millis(51), 144, 50).unwrap();
    }
    let frame = canvas.frame().expect("CPU canvas frame");
    assert_eq!((frame.width(), frame.height()), (144, 100));
    assert!(canvas.mode_label().starts_with("CPU fallback"));
    canvas.shutdown().unwrap();
    assert!(matches!(
        canvas.advance(Duration::from_secs(1), 144, 50),
        Err(GraphicsError::Cancelled)
    ));
}

#[test]
fn forced_cpu_is_shaded_animated_and_truthfully_labeled() {
    let mut renderer = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        ..Default::default()
    });
    let first = renderer.render_terminal(60, 24, Duration::ZERO).unwrap();
    let second = renderer
        .render_terminal(60, 24, Duration::from_secs(1))
        .unwrap();
    assert!(matches!(first.mode(), GraphicsMode::CpuFallback(_)));
    assert!(first.mode().label().starts_with("CPU fallback"));
    assert!(first.pixels().iter().copied().collect::<HashSet<_>>().len() > 32);
    assert_ne!(first.pixels(), second.pixels());
}

#[test]
fn injected_adapter_device_loss_and_readback_failures_select_cpu() {
    for fault in [
        GraphicsFault::Adapter,
        GraphicsFault::DeviceLoss,
        GraphicsFault::Readback,
    ] {
        let mut renderer = HybridCubeRenderer::new(GraphicsOptions {
            fault: Some(fault),
            ..Default::default()
        });
        let frame = renderer
            .render_terminal(60, 24, Duration::from_millis(375))
            .unwrap();
        assert!(matches!(frame.mode(), GraphicsMode::CpuFallback(_)));
        assert!(
            frame.mode().label().contains(fault.label()),
            "{}",
            frame.mode().label()
        );
        assert!(frame.pixels().iter().copied().collect::<HashSet<_>>().len() > 32);
        let next = renderer
            .render_terminal(144, 50, Duration::from_secs(1))
            .unwrap();
        assert!(matches!(next.mode(), GraphicsMode::CpuFallback(_)));
        assert_eq!((next.width(), next.height()), (144, 100));
        println!(
            "GPU FALLBACK fault={} mode={}",
            fault.label(),
            frame.mode().label()
        );
    }
}

#[test]
fn shutdown_cancels_active_and_pending_work_without_publication() {
    let (started_send, started_receive) = mpsc::channel();
    let (cancelled_send, cancelled_receive) = mpsc::channel();
    let (notify_send, notify_receive) = mpsc::channel();
    let mut worker = GraphicsWorker::spawn_cancellable(
        move |_, cancellation| {
            started_send.send(()).unwrap();
            while !cancellation.is_cancelled() {
                std::thread::sleep(Duration::from_millis(5));
            }
            cancelled_send.send(()).unwrap();
            Err(GraphicsError::Cancelled)
        },
        move || {
            notify_send.send(()).unwrap();
        },
    );
    worker.request(FrameRequest::new(2, 2, Duration::ZERO).unwrap());
    started_receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    worker.request(FrameRequest::new(2, 2, Duration::from_secs(1)).unwrap());
    assert_eq!(worker.stats().pending, 1);
    worker.cancel();
    assert_eq!(worker.stats().pending, 0);
    cancelled_receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    worker.shutdown().unwrap();
    assert_eq!(worker.stats().active, 0);
    assert_eq!(worker.stats().completed, 1);
    assert!(worker.take_latest().is_none());
    assert!(
        notify_receive.try_recv().is_err(),
        "cancelled result was published"
    );
}

#[test]
fn torus_effect_renders_shaded_animated_cpu_pixels() {
    let mut renderer = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        effect: GraphicsEffect::Torus,
        ..Default::default()
    });
    let first = renderer.render_terminal(60, 24, Duration::ZERO).unwrap();
    let second = renderer
        .render_terminal(60, 24, Duration::from_secs(1))
        .unwrap();
    assert!(matches!(first.mode(), GraphicsMode::CpuFallback(_)));
    assert!(first.mode().label().starts_with("CPU fallback"));
    let colors = first.pixels().iter().copied().collect::<HashSet<_>>();
    assert!(colors.len() >= 16, "torus output is not shaded");
    let background = first.pixels()[0];
    assert!(
        first.pixels().iter().any(|pixel| *pixel != background),
        "torus painted only background"
    );
    assert_ne!(first.pixels(), second.pixels());
}

#[test]
fn cube_and_torus_effects_disagree_on_cpu() {
    let mut cube = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        ..Default::default()
    });
    let mut torus = HybridCubeRenderer::new(GraphicsOptions {
        force_cpu: true,
        effect: GraphicsEffect::Torus,
        ..Default::default()
    });
    let cube_frame = cube
        .render_terminal(60, 24, Duration::from_millis(375))
        .unwrap();
    let torus_frame = torus
        .render_terminal(60, 24, Duration::from_millis(375))
        .unwrap();
    assert_ne!(cube_frame.pixels(), torus_frame.pixels());
}

#[test]
fn default_effect_remains_the_shaded_cube() {
    assert_eq!(GraphicsOptions::default().effect, GraphicsEffect::Cube);
}
