use crate::core::geometry::Size;
use crate::core::render_stats::{DetailedFrameStats, PerformanceMetrics, RenderStatsCollector};
use crate::core::surface::{DiffWriter, Rgba, Surface};
use crate::core::terminal::Terminal;
use crate::error::{ReactiveError, Result};
use std::time::{Duration, Instant};

/// Statistics for a single frame render
#[derive(Debug, Default, Clone, Copy)]
pub struct FrameStats {
    /// Total bytes written to terminal
    pub bytes_written: usize,
    /// Number of text spans written
    pub spans_written: usize,
    /// Number of rows that changed
    pub rows_changed: usize,
    /// Frame render time in milliseconds
    pub frame_ms: f32,
}

/// Last statistics supplied by the embedding host, independent of measured frame statistics.
#[derive(Debug, Default, Clone, Copy)]
pub struct HostStats {
    /// Host-reported time value.
    pub time: Option<f64>,
    /// Host-reported frames per second.
    pub fps: Option<u32>,
    /// Host-reported frame callback time.
    pub frame_callback_time: Option<f64>,
    /// Host-reported used heap bytes.
    pub heap_used: Option<u32>,
    /// Host-reported total heap bytes.
    pub heap_total: Option<u32>,
    /// Host-reported array-buffer bytes.
    pub array_buffers: Option<u32>,
}

impl HostStats {
    fn overlay_text(&self) -> String {
        [
            ("FPS", self.fps.map(|v| v.to_string())),
            ("Time", self.time.map(|v| v.to_string())),
            ("Callback", self.frame_callback_time.map(|v| v.to_string())),
            ("Heap used", self.heap_used.map(|v| v.to_string())),
            ("Heap total", self.heap_total.map(|v| v.to_string())),
            ("Array buffers", self.array_buffers.map(|v| v.to_string())),
        ]
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| format!(" | Host {name} {value}")))
        .collect()
    }
}

/// Double-buffered terminal renderer with diff-based updates
pub struct Renderer {
    term: Terminal,
    front: Surface,
    back: Surface,
    diff: DiffWriter,
    last_stats: FrameStats,
    frame_start: Option<Instant>,
    debug_overlay: bool,
    // Enhanced statistics
    stats_collector: RenderStatsCollector,
    detailed_stats_enabled: bool,
    // Timing breakdown
    diff_start: Option<Instant>,
    write_start: Option<Instant>,
    surface_start: Option<Instant>,
    output_failed: bool,
    terminal_active: bool,
    built_hits: Vec<u32>,
    shown_hits: Vec<u32>,
    row_offset: u32,
    host_stats: HostStats,
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Use std::panic::catch_unwind to ensure terminal cleanup even during double panic
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.restore_terminal()));

        // If normal cleanup failed, try emergency terminal restore
        if result.is_err() || matches!(result, Ok(Err(_))) {
            // Emergency terminal restoration - try basic operations individually
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                use crossterm::{cursor, execute, terminal};
                let _ = execute!(std::io::stdout(), cursor::Show);
                let _ = execute!(std::io::stdout(), terminal::LeaveAlternateScreen);
                let _ = terminal::disable_raw_mode();
            }));
        }
    }
}

impl Renderer {
    /// Create a new renderer with specified dimensions
    pub fn new(width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(ReactiveError::invalid_parameter(
                "Renderer dimensions must be greater than 0",
            ));
        }
        let mut term = Terminal::new()?;
        term.capability_gate()?;
        term.enter_modern_mode()?;
        let caps = term.capabilities();
        let mut images = crate::backend::ImageOutputOptions {
            kitty_graphics: caps.kitty_graphics,
            sixel: caps.sixel,
            iterm2_inline: caps.iterm2_graphics,
            ..Default::default()
        };
        images.refresh_cell_pixels();
        let mut diff = DiffWriter::new();
        diff.set_image_options(images);
        Ok(Self {
            term,
            front: Surface::new(width, height),
            back: Surface::new(width, height),
            diff,
            last_stats: FrameStats::default(),
            frame_start: None,
            debug_overlay: false,
            stats_collector: RenderStatsCollector::new(120), // Keep 2 seconds at 60fps
            detailed_stats_enabled: false,
            diff_start: None,
            write_start: None,
            surface_start: None,
            output_failed: true,
            terminal_active: true,
            built_hits: vec![0; width * height],
            shown_hits: vec![0; width * height],
            row_offset: 0,
            host_stats: HostStats::default(),
        })
    }

    /// Create a new renderer with Size
    pub fn with_size(size: Size) -> Result<Self> {
        Self::new(size.width, size.height)
    }

    /// Resize the renderer to new dimensions
    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.reinit(width, height);
        self.back.reinit(width, height);
        self.built_hits = vec![0; width * height];
        self.shown_hits = vec![0; width * height];
        self.diff.refresh_image_cell_pixels();
        self.output_failed = true;
    }

    /// Override detected graphics support with caller-confirmed host settings.
    pub fn set_image_options(&mut self, mut options: crate::backend::ImageOutputOptions) {
        options.refresh_cell_pixels();
        self.diff.set_image_options(options);
        self.output_failed = true;
    }

    /// Enable high-performance mode with large output buffers
    pub fn enable_high_performance_mode(&mut self) -> Result<()> {
        self.term.enable_buffered_mode()
    }

    /// Disable high-performance mode and return to direct writes
    pub fn disable_high_performance_mode(&mut self) -> Result<()> {
        self.term.disable_buffered_mode()
    }

    /// Get terminal write statistics
    pub fn write_stats(&self) -> &crate::core::terminal::TerminalWriteStats {
        self.term.write_stats()
    }

    /// Reset write statistics
    pub fn reset_write_stats(&mut self) {
        self.term.reset_write_stats();
    }

    /// Enable detailed performance statistics collection
    pub fn enable_detailed_stats(&mut self) {
        self.detailed_stats_enabled = true;
    }

    /// Disable detailed performance statistics collection
    pub fn disable_detailed_stats(&mut self) {
        self.detailed_stats_enabled = false;
    }

    /// Get comprehensive performance metrics
    pub fn performance_metrics(&self) -> PerformanceMetrics {
        self.stats_collector.get_metrics()
    }

    /// Get the latest detailed frame statistics
    pub fn latest_detailed_stats(&self) -> Option<&DetailedFrameStats> {
        self.stats_collector.latest_frame()
    }

    /// Clear all collected statistics
    pub fn clear_stats(&mut self) {
        self.stats_collector.clear();
    }

    /// Check if performance is considered good
    pub fn is_performance_good(&self) -> bool {
        self.stats_collector.is_performance_good()
    }

    /// Get performance grade (A-F)
    pub fn performance_grade(&self) -> char {
        self.stats_collector.performance_grade()
    }

    /// Resize using Size
    pub fn resize_to(&mut self, size: Size) {
        self.resize(size.width, size.height);
    }

    /// Clear the back buffer with a solid color
    pub fn clear(&mut self, color: Rgba) {
        self.back.clear(color);
    }

    /// Enable or disable debug overlay
    pub fn set_debug_overlay(&mut self, enabled: bool) {
        self.debug_overlay = enabled;
    }

    /// Begin a new rendering frame
    pub fn begin_frame(&mut self) -> Result<()> {
        self.frame_start = Some(Instant::now());
        self.surface_start = Some(Instant::now());
        self.term.begin_sync()
    }

    /// End the current frame and send updates to terminal
    pub fn end_frame(&mut self) -> Result<()> {
        let output = self
            .write_frame()
            .and_then(|()| self.term.flush_buffered())
            .and_then(|()| self.term.flush());
        // End synchronized output even after preparation, write or flush failed.
        let ended = self.term.end_sync();
        let result = output.and(ended);
        self.output_failed = result.is_err();
        if self.output_failed {
            self.term.discard_buffered();
        }
        result?;
        self.diff.acknowledge_output();
        std::mem::swap(&mut self.built_hits, &mut self.shown_hits);
        self.built_hits.fill(0);
        if self.front.dims() == self.back.dims() {
            self.front.copy_from(&self.back);
        } else {
            self.front = self.back.clone_into_new();
        }
        Ok(())
    }

    fn write_frame(&mut self) -> Result<()> {
        let surface_time = self
            .surface_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Start diff timing
        self.diff_start = Some(Instant::now());

        // Diff and write
        self.diff
            .try_diff(&self.front, &self.back, self.output_failed)?;
        let out = self.diff.output();

        let diff_time = self
            .diff_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Start write timing
        self.write_start = Some(Instant::now());

        self.term.write_all_buffered(out)?;

        let write_time = self
            .write_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or_default();
        // Basic stats (backward compatibility)
        self.last_stats.bytes_written = out.len();
        self.last_stats.spans_written = self.diff.last_spans_written();
        self.last_stats.rows_changed = self.diff.last_rows_changed();
        let frame_time = if let Some(start) = self.frame_start.take() {
            let elapsed = start.elapsed();
            self.last_stats.frame_ms = elapsed.as_secs_f32() * 1000.0;
            elapsed
        } else {
            std::time::Duration::ZERO
        };

        // Collect detailed statistics if enabled
        if self.detailed_stats_enabled {
            let (width, height) = self.back.dims();
            let cells_total = (width * height) as u32;
            let cells_updated = self.diff.last_spans_written() as u32; // Approximation

            let detailed_stats = DetailedFrameStats {
                frame_time,
                diff_time,
                write_time,
                surface_time,
                cells_updated,
                cells_total,
                bytes_written: out.len() as u64,
                spans_written: self.diff.last_spans_written() as u32,
                rows_changed: self.diff.last_rows_changed() as u32,
                timestamp: Instant::now(),
                memory_usage: self.estimate_memory_usage(),
                // Component metrics - will be tracked in future
                components_created: 0,
                components_destroyed: 0,
                active_components: 0,
                component_creation_time: Duration::ZERO,
                component_cleanup_time: Duration::ZERO,
            };

            self.stats_collector.record_frame(detailed_stats);
        }
        // Optional debug overlay
        if self.debug_overlay {
            let (_w, h) = self.back.dims();
            let write_stats = self.term.write_stats();

            let overlay = if self.detailed_stats_enabled {
                let metrics = self.stats_collector.get_metrics();
                format!(
                    "\x1b[{row};1H\x1b[7mFPS: {fps:.1} | Frame: {ms:.1}ms | Diff: {diff:.1}ms | Write: {write:.1}ms | Eff: {eff:.1}% | Grade: {grade} | Buf: {buf:.1}%\x1b[0m",
                    row = h as u64 + u64::from(self.row_offset),
                    fps = metrics.fps,
                    ms = self.last_stats.frame_ms,
                    diff = diff_time.as_secs_f32() * 1000.0,
                    write = write_time.as_secs_f32() * 1000.0,
                    eff = metrics.efficiency_ratio * 100.0,
                    grade = self.performance_grade(),
                    buf = write_stats.buffer_utilization * 100.0
                )
            } else {
                format!(
                    "\x1b[{row};1H\x1b[7mframe: {ms:.2}ms | bytes: {b} | spans: {s} | rows: {r} | buf: {buf:.1}%\x1b[0m",
                    row = h as u64 + u64::from(self.row_offset),
                    ms = self.last_stats.frame_ms,
                    b = self.last_stats.bytes_written,
                    s = self.last_stats.spans_written,
                    r = self.last_stats.rows_changed,
                    buf = write_stats.buffer_utilization * 100.0
                )
            };

            let mut overlay = overlay;
            // Append host values beside measured values before resetting the style.
            overlay.truncate(overlay.len() - "\x1b[0m".len());
            overlay.push_str(&self.host_stats.overlay_text());
            overlay.push_str("\x1b[0m");
            self.term.write_all_buffered(overlay.as_bytes())?;
        }
        Ok(())
    }

    /// Shutdown the renderer and restore terminal state
    pub fn shutdown(mut self) -> Result<()> {
        self.restore_terminal()
    }

    /// Restore terminal state while retaining the renderer allocation for FFI destruction.
    pub(crate) fn restore_terminal(&mut self) -> Result<()> {
        if !self.terminal_active {
            return Ok(());
        }
        self.terminal_active = false;
        let flushed = self.term.disable_buffered_mode();
        let cleaned = self
            .term
            .write_raw(&self.diff.image_cleanup())
            .and_then(|()| self.term.flush());
        let restored = self.term.exit_modern_mode();
        flushed.and(cleaned).and(restored)
    }

    /// Get statistics from the last rendered frame
    pub fn frame_stats(&self) -> FrameStats {
        self.last_stats
    }

    /// Get a mutable reference to the back buffer surface
    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.back
    }
    /// Get a reference to the back buffer surface
    pub fn surface(&self) -> &Surface {
        &self.back
    }

    /// Register a clipped region in the next frame's hit grid. Later registrations win.
    pub fn add_hit_region(&mut self, x: i32, y: i32, width: u32, height: u32, id: u32) {
        let (w, h) = self.dims();
        let left = i64::from(x).clamp(0, w as i64) as usize;
        let top = i64::from(y).clamp(0, h as i64) as usize;
        let right = (i64::from(x) + i64::from(width)).clamp(0, w as i64) as usize;
        let bottom = (i64::from(y) + i64::from(height)).clamp(0, h as i64) as usize;
        for row in top..bottom {
            self.built_hits[row * w + left..row * w + right].fill(id);
        }
    }

    /// Read an id from the last successfully completed frame, or zero outside it.
    pub fn check_hit(&self, x: usize, y: usize) -> u32 {
        let (w, h) = self.dims();
        if x < w && y < h {
            self.shown_hits[y * w + x]
        } else {
            0
        }
    }

    /// Bounding boxes of nonzero ids in the shown grid, in id order.
    pub fn hit_regions(&self) -> std::collections::BTreeMap<u32, (usize, usize, usize, usize)> {
        let mut regions = std::collections::BTreeMap::new();
        let (w, _) = self.dims();
        for (index, &id) in self
            .shown_hits
            .iter()
            .enumerate()
            .filter(|(_, id)| **id != 0)
        {
            let (x, y) = (index % w, index / w);
            let bounds = regions.entry(id).or_insert((x, y, x, y));
            bounds.0 = bounds.0.min(x);
            bounds.1 = bounds.1.min(y);
            bounds.2 = bounds.2.max(x);
            bounds.3 = bounds.3.max(y);
        }
        regions
    }

    /// Shift every emitted text cursor row; changing the offset forces a repaint.
    pub fn set_render_offset(&mut self, offset: u32) {
        if self.row_offset != offset {
            self.row_offset = offset;
            self.diff.set_row_offset(offset);
            self.output_failed = true;
        }
    }

    /// Last host values, editable independently of the renderer's measured statistics.
    pub fn host_stats_mut(&mut self) -> &mut HostStats {
        &mut self.host_stats
    }

    /// Write dimensions, both surfaces' cell text, frame statistics and host statistics.
    pub fn dump_buffers(&self, path: &std::path::Path) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::io::BufWriter::new(std::fs::File::create(path)?);
        let (w, h) = self.dims();
        writeln!(file, "Dimensions: {w}x{h}")?;
        for (name, surface) in [("Front", &self.front), ("Back", &self.back)] {
            writeln!(file, "{name}:")?;
            for y in 0..h {
                for x in 0..w {
                    let grapheme = surface.grapheme(x, y);
                    write!(
                        file,
                        "{}",
                        if grapheme.is_empty() || grapheme == "\0" {
                            " "
                        } else {
                            &grapheme
                        }
                    )?;
                }
                writeln!(file)?;
            }
        }
        writeln!(file, "Frame statistics: {:?}", self.last_stats)?;
        writeln!(file, "Host statistics: {:?}", self.host_stats)?;
        file.flush()
    }

    /// Estimate memory usage of the renderer
    fn estimate_memory_usage(&self) -> usize {
        let (width, height) = self.back.dims();
        let surface_size = width * height * std::mem::size_of::<crate::core::surface::Cell>() * 2; // front + back
        let diff_buffer_size = self.diff.output().len();
        let stats_size =
            self.stats_collector.samples().len() * std::mem::size_of::<DetailedFrameStats>();

        surface_size + diff_buffer_size + stats_size + 1024 // Base overhead
    }

    /// Get the dimensions of the renderer
    pub fn dims(&self) -> (usize, usize) {
        self.back.dims()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::surface::{Attr, Rgba};
    use crate::terminal::test_terminal::on_terminal;

    #[test]
    fn ffi_004_hit_grid_clips_publishes_clears_and_resizes() {
        on_terminal(
            "core::renderer::tests::ffi_004_hit_grid_clips_publishes_clears_and_resizes",
            || {
                let mut renderer = Renderer::new(4, 3).unwrap();
                renderer.add_hit_region(-1, -1, 3, 3, 7);
                renderer.add_hit_region(1, 1, u32::MAX, u32::MAX, 9);
                renderer.add_hit_region(-5, -5, 1, 1, 99);
                renderer.add_hit_region(100, 100, u32::MAX, u32::MAX, 99);
                assert_eq!(renderer.check_hit(0, 0), 0);
                renderer.begin_frame().unwrap();
                renderer.end_frame().unwrap();
                assert_eq!(renderer.check_hit(0, 0), 7);
                assert_eq!(renderer.check_hit(1, 1), 9);
                assert_eq!(renderer.check_hit(3, 2), 9);
                assert_eq!(renderer.check_hit(4, 2), 0);
                assert_eq!(renderer.hit_regions().get(&7), Some(&(0, 0, 1, 1)));
                assert_eq!(renderer.hit_regions().get(&9), Some(&(1, 1, 3, 2)));
                renderer.begin_frame().unwrap();
                renderer.end_frame().unwrap();
                assert_eq!(renderer.check_hit(0, 0), 0);
                renderer.add_hit_region(0, 0, 4, 3, 8);
                renderer.resize(2, 1);
                renderer.begin_frame().unwrap();
                renderer.end_frame().unwrap();
                assert_eq!(renderer.check_hit(0, 0), 0);
            },
        );
    }

    #[test]
    fn ffi_004_dump_keeps_both_surfaces_graphemes_and_host_values() {
        on_terminal(
            "core::renderer::tests::ffi_004_dump_keeps_both_surfaces_graphemes_and_host_values",
            || {
                let mut renderer = Renderer::new(4, 1).unwrap();
                assert!(renderer.surface_mut().set_grapheme(
                    0,
                    0,
                    "e\u{301}",
                    crate::core::surface::Cell::default()
                ));
                renderer.surface_mut().write_str(
                    1,
                    0,
                    "界",
                    Rgba::white(),
                    Rgba::black(),
                    Attr::empty(),
                );
                renderer.host_stats = HostStats {
                    time: Some(16.0),
                    fps: Some(4242),
                    frame_callback_time: Some(1.0),
                    heap_used: Some(123456),
                    heap_total: Some(999999),
                    array_buffers: Some(7),
                };
                renderer.begin_frame().unwrap();
                renderer.end_frame().unwrap();
                renderer.surface_mut().write_str(
                    0,
                    0,
                    "BACK",
                    Rgba::white(),
                    Rgba::black(),
                    Attr::empty(),
                );
                let dir = tempfile::tempdir().unwrap();
                let path = dir.path().join("buffers.txt");
                renderer.dump_buffers(&path).unwrap();
                let dump = std::fs::read_to_string(&path).unwrap();
                for text in [
                    "Dimensions: 4x1",
                    "Front:\ne\u{301}界  ",
                    "Back:\nBACK",
                    "Frame statistics:",
                    "4242",
                    "123456",
                    "999999",
                    "array_buffers: Some(7)",
                ] {
                    assert!(dump.contains(text), "missing {text:?} from {dump:?}");
                }
                assert!(renderer
                    .dump_buffers(&dir.path().join("missing/file"))
                    .is_err());
            },
        );
    }

    #[test]
    fn test_renderer_creation() {
        on_terminal("core::renderer::tests::test_renderer_creation", || {
            let renderer = Renderer::new(80, 24).unwrap();
            assert_eq!(renderer.dims(), (80, 24));
        });
    }

    #[test]
    fn test_renderer_invalid_dimensions() {
        // Test zero width
        let result = Renderer::new(0, 24);
        assert!(result.is_err());

        // Test zero height
        let result = Renderer::new(80, 0);
        assert!(result.is_err());

        // Test both zero
        let result = Renderer::new(0, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_renderer_with_size() {
        on_terminal("core::renderer::tests::test_renderer_with_size", || {
            let renderer = Renderer::with_size(Size::new(100, 30)).unwrap();
            assert_eq!(renderer.dims(), (100, 30));
        });
    }

    #[test]
    fn test_renderer_resize() {
        on_terminal("core::renderer::tests::test_renderer_resize", || {
            let mut renderer = Renderer::new(80, 24).unwrap();
            assert_eq!(renderer.dims(), (80, 24));

            renderer.resize(120, 40);
            assert_eq!(renderer.dims(), (120, 40));

            // Test resize to smaller dimensions
            renderer.resize(60, 20);
            assert_eq!(renderer.dims(), (60, 20));
        });
    }

    #[test]
    fn test_renderer_resize_to() {
        on_terminal("core::renderer::tests::test_renderer_resize_to", || {
            let mut renderer = Renderer::new(80, 24).unwrap();
            renderer.resize_to(Size::new(100, 50));
            assert_eq!(renderer.dims(), (100, 50));
        });
    }

    #[test]
    fn test_frame_stats_initialization() {
        on_terminal(
            "core::renderer::tests::test_frame_stats_initialization",
            || {
                let renderer = Renderer::new(80, 24).unwrap();
                let stats = renderer.frame_stats();

                // Initial stats should be zero
                assert_eq!(stats.bytes_written, 0);
                assert_eq!(stats.spans_written, 0);
                assert_eq!(stats.rows_changed, 0);
                assert_eq!(stats.frame_ms, 0.0);
            },
        );
    }

    #[test]
    fn test_debug_overlay_toggle() {
        on_terminal("core::renderer::tests::test_debug_overlay_toggle", || {
            let mut renderer = Renderer::new(80, 24).unwrap();

            // Debug overlay is disabled by default and follows the toggle
            assert!(!renderer.debug_overlay);
            renderer.set_debug_overlay(true);
            assert!(renderer.debug_overlay);
            renderer.set_debug_overlay(false);
            assert!(!renderer.debug_overlay);
        });
    }

    #[test]
    fn test_detailed_stats_toggle() {
        on_terminal("core::renderer::tests::test_detailed_stats_toggle", || {
            let mut renderer = Renderer::new(80, 24).unwrap();

            // Detailed stats are off by default and follow the toggle
            assert!(!renderer.detailed_stats_enabled);
            renderer.enable_detailed_stats();
            assert!(renderer.detailed_stats_enabled);
            renderer.disable_detailed_stats();
            assert!(!renderer.detailed_stats_enabled);
        });
    }

    #[test]
    fn test_performance_metrics() {
        on_terminal("core::renderer::tests::test_performance_metrics", || {
            let renderer = Renderer::new(80, 24).unwrap();

            // Should be able to get metrics even without any frames
            let metrics = renderer.performance_metrics();
            assert!(metrics.fps >= 0.0);
            assert!(metrics.efficiency_ratio >= 0.0);
            assert!(metrics.efficiency_ratio <= 1.0);
        });
    }

    #[test]
    fn test_performance_grade() {
        on_terminal("core::renderer::tests::test_performance_grade", || {
            let renderer = Renderer::new(80, 24).unwrap();

            let grade = renderer.performance_grade();
            // Should be a valid grade letter
            assert!("ABCDEF".contains(grade));
        });
    }

    #[test]
    fn test_clear_surface() {
        on_terminal("core::renderer::tests::test_clear_surface", || {
            let mut renderer = Renderer::new(80, 24).unwrap();
            let red = Rgba::new(1.0, 0.0, 0.0, 1.0);

            renderer.clear(red);
            // Every back-buffer cell takes the clear colour
            let (width, height) = renderer.dims();
            assert_eq!(renderer.surface().get(0, 0).bg, red);
            assert_eq!(renderer.surface().get(width - 1, height - 1).bg, red);
        });
    }

    #[test]
    fn test_surface_access() {
        on_terminal("core::renderer::tests::test_surface_access", || {
            let mut renderer = Renderer::new(80, 24).unwrap();

            // Test immutable access
            let surface = renderer.surface();
            assert_eq!(surface.dims(), (80, 24));

            // Test mutable access
            let surface_mut = renderer.surface_mut();
            assert_eq!(surface_mut.dims(), (80, 24));

            // A write through the mutable surface is what the next frame shows
            surface_mut.write_str(0, 0, "Test", Rgba::white(), Rgba::black(), Attr::empty());
            assert_eq!(renderer.surface().get(3, 0).ch, 't');
        });
    }

    #[test]
    fn test_stats_management() {
        on_terminal("core::renderer::tests::test_stats_management", || {
            let mut renderer = Renderer::new(80, 24).unwrap();

            // Test clearing stats
            renderer.clear_stats();

            // Test performance check
            assert!(
                !renderer.is_performance_good(),
                "Cleared stats have no measured frame rate"
            );

            // Test latest detailed stats (should be None initially)
            let latest = renderer.latest_detailed_stats();
            assert!(latest.is_none());
        });
    }

    #[test]
    fn test_memory_estimation() {
        on_terminal("core::renderer::tests::test_memory_estimation", || {
            let renderer = Renderer::new(80, 24).unwrap();

            // Memory usage should be reasonable for the given dimensions
            let memory = renderer.estimate_memory_usage();
            assert!(memory > 0);

            // Larger renderer should use more memory
            let large_renderer = Renderer::new(200, 100).unwrap();
            assert!(large_renderer.estimate_memory_usage() > memory);
        });
    }
}
