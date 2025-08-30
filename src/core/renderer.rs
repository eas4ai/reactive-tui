use crate::core::geometry::Size;
use crate::core::render_stats::{DetailedFrameStats, PerformanceMetrics, RenderStatsCollector};
use crate::core::surface::{DiffWriter, Rgba, Surface};
use crate::core::terminal::Terminal;
use crate::error::{ReactiveError, Result};
use std::time::Instant;

#[derive(Debug, Default, Clone, Copy)]
pub struct FrameStats {
    pub bytes_written: usize,
    pub spans_written: usize,
    pub rows_changed: usize,
    pub frame_ms: f32,
}

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
}

impl Drop for Renderer {
    fn drop(&mut self) {
        // Always restore terminal state, even on panic
        let _ = self.term.exit_modern_mode();
    }
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(ReactiveError::invalid_parameter(
                "Renderer dimensions must be greater than 0",
            ));
        }
        let mut term = Terminal::new().map_err(ReactiveError::Io)?;
        term.capability_gate()?;
        term.enter_modern_mode()?;
        Ok(Self {
            term,
            front: Surface::new(width, height),
            back: Surface::new(width, height),
            diff: DiffWriter::new(),
            last_stats: FrameStats::default(),
            frame_start: None,
            debug_overlay: false,
            stats_collector: RenderStatsCollector::new(120), // Keep 2 seconds at 60fps
            detailed_stats_enabled: false,
            diff_start: None,
            write_start: None,
            surface_start: None,
        })
    }

    /// Create a new renderer with Size
    pub fn with_size(size: Size) -> Result<Self> {
        Self::new(size.width, size.height)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.reinit(width, height);
        self.back.reinit(width, height);
    }

    /// Enable high-performance mode with large output buffers
    pub fn enable_high_performance_mode(&mut self) -> Result<()> {
        self.term.enable_buffered_mode().map_err(ReactiveError::Io)
    }

    /// Disable high-performance mode and return to direct writes
    pub fn disable_high_performance_mode(&mut self) -> Result<()> {
        self.term.disable_buffered_mode().map_err(ReactiveError::Io)
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

    pub fn clear(&mut self, color: Rgba) {
        self.back.clear(color);
    }

    pub fn set_debug_overlay(&mut self, enabled: bool) {
        self.debug_overlay = enabled;
    }

    pub fn begin_frame(&mut self) -> Result<()> {
        self.frame_start = Some(Instant::now());
        self.surface_start = Some(Instant::now());
        self.term.begin_sync().map_err(ReactiveError::Io)
    }

    pub fn end_frame(&mut self) -> Result<()> {
        let surface_time = self
            .surface_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Start diff timing
        self.diff_start = Some(Instant::now());

        // Diff and write
        self.diff.diff(&self.front, &self.back, false);
        let out = self.diff.output();

        let diff_time = self
            .diff_start
            .take()
            .map(|start| start.elapsed())
            .unwrap_or_default();

        // Start write timing
        self.write_start = Some(Instant::now());

        // Try buffered write first, fall back to direct write
        if self.term.write_all_buffered(out).is_err() {
            Terminal::write_all(out).map_err(ReactiveError::Io)?;
        }

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
                    row = h,
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
                    row = h,
                    ms = self.last_stats.frame_ms,
                    b = self.last_stats.bytes_written,
                    s = self.last_stats.spans_written,
                    r = self.last_stats.rows_changed,
                    buf = write_stats.buffer_utilization * 100.0
                )
            };

            // Try buffered write first, fall back to direct write
            if self.term.write_all_buffered(overlay.as_bytes()).is_err() {
                Terminal::write_all(overlay.as_bytes()).map_err(ReactiveError::Io)?;
            }
        }
        // Make front reflect the just-rendered back buffer for next diff
        let (fw, fh) = self.front.dims();
        let (bw, bh) = self.back.dims();
        if (fw, fh) == (bw, bh) {
            self.front.copy_from(&self.back);
        } else {
            self.front = self.back.clone_into_new();
        }

        // Flush buffered output before ending sync
        let _ = self.term.flush_buffered();

        self.term.end_sync().map_err(ReactiveError::Io)
    }

    pub fn shutdown(mut self) -> Result<()> {
        // Ensure all buffered data is flushed before shutdown
        let _ = self.term.disable_buffered_mode();
        self.term.exit_modern_mode().map_err(ReactiveError::Io)
    }

    pub fn frame_stats(&self) -> FrameStats {
        self.last_stats
    }

    pub fn surface_mut(&mut self) -> &mut Surface {
        &mut self.back
    }
    pub fn surface(&self) -> &Surface {
        &self.back
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

    pub fn dims(&self) -> (usize, usize) {
        self.back.dims()
    }
}
