#[cfg(test)]
mod renderer_tests {
    use crate::core::renderer::{Renderer, FrameStats};
    use crate::core::surface::{Rgba, Attr};

    #[test]
    fn renderer_stats_basic() {
        let mut renderer = Renderer::new(20, 5).expect("renderer new");
        let surface = renderer.surface_mut();
        surface.clear(Rgba::black());
        surface.write_str(0, 0, "hello", Rgba::white(), Rgba::black(), Attr::empty());
        renderer.begin_frame().expect("Should be able to begin frame");
        renderer.end_frame().expect("Should be able to end frame");
        let stats: FrameStats = renderer.frame_stats();
        assert!(stats.bytes_written > 0);
        assert!(stats.spans_written > 0);
        assert!(stats.rows_changed > 0);
        assert!(stats.frame_ms >= 0.0);
    }

    #[test]
    fn renderer_overlay_toggle() {
        let mut renderer = Renderer::new(10, 3).expect("renderer new");
        renderer.set_debug_overlay(true);
        renderer.surface_mut().write_str(0, 0, "x", Rgba::white(), Rgba::black(), Attr::empty());
        renderer.begin_frame().expect("Should be able to begin frame");
        renderer.end_frame().expect("Should be able to end frame");
        let stats = renderer.frame_stats();
        assert!(stats.bytes_written > 0);
    }
}

