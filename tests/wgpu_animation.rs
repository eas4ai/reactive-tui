use reactive_tui::graphics::{
    FrameClock, FrameRequest, GpuCubeRenderer, GraphicsFrame, GraphicsWorker,
};
use std::{sync::mpsc, time::Duration};

#[test]
fn deadline_clock_skips_missed_frames_and_preserves_elapsed_time() {
    let mut regular = FrameClock::default();
    let mut irregular = FrameClock::default();
    for millis in (0..500).step_by(50) {
        assert!(regular
            .request_at(Duration::from_millis(millis), 60, 24)
            .unwrap()
            .is_some());
    }
    assert!(irregular
        .request_at(Duration::ZERO, 60, 24)
        .unwrap()
        .is_some());
    assert!(irregular
        .request_at(Duration::from_millis(49), 60, 24)
        .unwrap()
        .is_none());
    let regular_request = regular
        .request_at(Duration::from_millis(500), 60, 24)
        .unwrap()
        .unwrap();
    let irregular_request = irregular
        .request_at(Duration::from_millis(500), 60, 24)
        .unwrap()
        .unwrap();
    assert_eq!(regular_request, irregular_request);
    assert!(irregular
        .request_at(Duration::from_millis(501), 60, 24)
        .unwrap()
        .is_none());
    assert!(irregular
        .request_at(Duration::from_millis(549), 60, 24)
        .unwrap()
        .is_none());
    assert!(irregular
        .request_at(Duration::from_millis(550), 60, 24)
        .unwrap()
        .is_some());
}

#[test]
fn slow_renderer_has_one_active_and_one_replaceable_pending_frame() {
    let (started_send, started_receive) = mpsc::channel();
    let (release_send, release_receive) = mpsc::channel();
    let (notify_send, notify_receive) = mpsc::channel();
    let mut first = true;
    let mut worker = GraphicsWorker::spawn(
        move |request| {
            if first {
                first = false;
                started_send.send(()).unwrap();
                release_receive
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap();
            }
            GraphicsFrame::from_rgba(
                request.columns(),
                request.rows() * 2,
                vec![[128, 64, 32, 255]; request.columns() as usize * request.rows() as usize * 2],
            )
        },
        move || {
            notify_send.send(()).unwrap();
        },
    );
    assert!(worker.request(FrameRequest::new(2, 2, Duration::ZERO).unwrap()));
    started_receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    for millis in 1..=1000 {
        assert!(worker.request(FrameRequest::new(2, 2, Duration::from_millis(millis)).unwrap()));
        let stats = worker.stats();
        assert_eq!(stats.active, 1);
        assert_eq!(stats.pending, 1);
    }
    assert_eq!(worker.stats().replaced, 999);
    release_send.send(()).unwrap();
    notify_receive.recv_timeout(Duration::from_secs(2)).unwrap();
    notify_receive.recv_timeout(Duration::from_secs(2)).unwrap();
    let output = worker.take_latest().unwrap();
    assert_eq!(output.request.elapsed(), Duration::from_millis(1000));
    assert!(output.frame.is_ok());
    assert!(worker.take_latest().is_none());
    worker.shutdown().unwrap();
    assert_eq!(worker.stats().completed, 2);
    assert_eq!(worker.stats().active, 0);
    assert_eq!(worker.stats().pending, 0);
    assert!(!worker.request(FrameRequest::new(2, 2, Duration::ZERO).unwrap()));
}

#[test]
fn viewport_validation_precedes_queue_and_allocation() {
    for (columns, rows) in [(0, 2), (2, 0), (801, 2), (2, 301), (u32::MAX, u32::MAX)] {
        assert!(FrameRequest::new(columns, rows, Duration::ZERO).is_err());
        assert!(FrameClock::default()
            .request_at(Duration::ZERO, columns, rows)
            .is_err());
    }
    assert!(FrameRequest::new(800, 300, Duration::MAX).is_ok());
    let mut exhausted = FrameClock::default();
    assert!(exhausted.request_at(Duration::MAX, 2, 2).unwrap().is_some());
    assert!(exhausted.request_at(Duration::MAX, 2, 2).unwrap().is_none());
}

#[test]
fn real_gpu_rotation_changes_pixels_without_keyboard_input() {
    let renderer = GpuCubeRenderer::new().unwrap();
    assert!(renderer.adapter_info().is_hardware);
    let first = renderer.render_terminal(60, 24, Duration::ZERO).unwrap();
    let second = renderer
        .render_terminal(60, 24, Duration::from_secs(1))
        .unwrap();
    let changed = first
        .pixels()
        .iter()
        .zip(second.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 200, "only {changed} pixels changed");
    let before_wrap = renderer
        .render_terminal(60, 24, Duration::from_millis(8975))
        .unwrap();
    let after_wrap = renderer
        .render_terminal(60, 24, Duration::from_millis(8977))
        .unwrap();
    let seam_changes = before_wrap
        .pixels()
        .iter()
        .zip(after_wrap.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        seam_changes < 100,
        "elapsed-time rotation jumps at one-turn wrap: {seam_changes}"
    );
    println!(
        "GPU ANIMATION hardware={} changed_pixels={changed}",
        renderer.adapter_info().name
    );
}
