#[cfg(test)]
mod tests {
    use super::super::core::renderer::{Renderer, FrameStats};
    use super::super::core::surface::{Rgba, Attr};

    #[test]
    fn renderer_stats_basic() {
        // Create a small renderer
        let mut renderer = Renderer::new(20, 5).expect("renderer new");

        // Draw some content
        let surface = renderer.surface_mut();
        surface.clear(Rgba::black());
        surface.write_str(0, 0, "hello", Rgba::white(), Rgba::black(), Attr::empty());

        // present
        renderer.begin_frame().unwrap();
        renderer.end_frame().unwrap();

        let stats: FrameStats = renderer.frame_stats();
        assert!(stats.bytes_written > 0, "bytes_written should be > 0");
        assert!(stats.spans_written > 0, "spans_written should be > 0");
        assert!(stats.rows_changed > 0, "rows_changed should be > 0");
        assert!(stats.frame_ms >= 0.0, "frame_ms should be measured");
    }

    #[test]
    fn renderer_overlay_toggle() {
        let mut renderer = Renderer::new(10, 3).expect("renderer new");
        renderer.set_debug_overlay(true);
        renderer.surface_mut().write_str(0, 0, "x", Rgba::white(), Rgba::black(), Attr::empty());
        renderer.begin_frame().unwrap();
        renderer.end_frame().unwrap();
        // If no panic and stats updated, overlay path executed safely
        let stats = renderer.frame_stats();
        assert!(stats.bytes_written > 0);
    }
}

