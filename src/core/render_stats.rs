//! Comprehensive render statistics and performance monitoring
//!
//! This module provides detailed performance monitoring capabilities
//! inspired by OpenTUI's statistics system, including FPS tracking,
//! efficiency metrics, and detailed timing breakdowns.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Detailed frame statistics with timing breakdowns
#[derive(Debug, Clone)]
pub struct DetailedFrameStats {
    /// Total frame time from begin_frame to end_frame
    pub frame_time: Duration,
    /// Time spent calculating diffs
    pub diff_time: Duration,
    /// Time spent writing to terminal
    pub write_time: Duration,
    /// Time spent in surface operations
    pub surface_time: Duration,
    /// Number of cells that changed
    pub cells_updated: u32,
    /// Total number of cells in surface
    pub cells_total: u32,
    /// Number of bytes written to terminal
    pub bytes_written: u64,
    /// Number of spans written
    pub spans_written: u32,
    /// Number of rows that changed
    pub rows_changed: u32,
    /// Timestamp when frame was recorded
    pub timestamp: Instant,
    /// Memory usage estimate in bytes
    pub memory_usage: usize,

    // Enhanced component metrics
    /// Number of components created this frame
    pub components_created: u32,
    /// Number of components destroyed this frame
    pub components_destroyed: u32,
    /// Current number of active components
    pub active_components: u32,
    /// Time spent in component creation
    pub component_creation_time: Duration,
    /// Time spent in component cleanup
    pub component_cleanup_time: Duration,
}

impl Default for DetailedFrameStats {
    fn default() -> Self {
        Self {
            frame_time: Duration::ZERO,
            diff_time: Duration::ZERO,
            write_time: Duration::ZERO,
            surface_time: Duration::ZERO,
            cells_updated: 0,
            cells_total: 0,
            bytes_written: 0,
            spans_written: 0,
            rows_changed: 0,
            timestamp: Instant::now(),
            memory_usage: 0,
            components_created: 0,
            components_destroyed: 0,
            active_components: 0,
            component_creation_time: Duration::ZERO,
            component_cleanup_time: Duration::ZERO,
        }
    }
}

/// Aggregated performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Current frames per second
    pub fps: f32,
    /// Average frame time over sample window
    pub avg_frame_time: Duration,
    /// Minimum frame time in sample window
    pub min_frame_time: Duration,
    /// Maximum frame time in sample window
    pub max_frame_time: Duration,
    /// Efficiency ratio (cells_updated / cells_total)
    pub efficiency_ratio: f32,
    /// Average bytes per frame
    pub avg_bytes_per_frame: f64,
    /// Average write time per frame
    pub avg_write_time: Duration,
    /// Average diff time per frame
    pub avg_diff_time: Duration,
    /// Total frames processed
    pub total_frames: u64,
    /// Estimated memory usage
    pub memory_usage: usize,
    /// Performance stability (low variance in frame times)
    pub stability_score: f32,

    // Enhanced component metrics
    /// Average component creation time
    pub avg_component_creation_time: Duration,
    /// Average component cleanup time
    pub avg_component_cleanup_time: Duration,
    /// Current number of active components
    pub active_components: u32,
    /// Total components created across all frames
    pub total_components_created: u64,
    /// Total components destroyed across all frames
    pub total_components_destroyed: u64,
    /// Component creation rate (components/second)
    pub component_creation_rate: f32,
    /// Component cleanup rate (components/second)
    pub component_cleanup_rate: f32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            avg_frame_time: Duration::ZERO,
            min_frame_time: Duration::ZERO,
            max_frame_time: Duration::ZERO,
            efficiency_ratio: 0.0,
            avg_bytes_per_frame: 0.0,
            avg_write_time: Duration::ZERO,
            avg_diff_time: Duration::ZERO,
            total_frames: 0,
            memory_usage: 0,
            stability_score: 1.0,
            avg_component_creation_time: Duration::ZERO,
            avg_component_cleanup_time: Duration::ZERO,
            active_components: 0,
            total_components_created: 0,
            total_components_destroyed: 0,
            component_creation_rate: 0.0,
            component_cleanup_rate: 0.0,
        }
    }
}

/// Statistics collector with rolling window
pub struct RenderStatsCollector {
    samples: VecDeque<DetailedFrameStats>,
    max_samples: usize,
    start_time: Instant,
    total_frames: u64,
}

impl RenderStatsCollector {
    /// Create a new stats collector
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            start_time: Instant::now(),
            total_frames: 0,
        }
    }

    /// Record a frame's statistics
    pub fn record_frame(&mut self, stats: DetailedFrameStats) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(stats);
        self.total_frames += 1;
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> PerformanceMetrics {
        if self.samples.is_empty() {
            return PerformanceMetrics::default();
        }

        let sample_count = self.samples.len() as f32;

        // Calculate FPS based on recent samples
        let fps = if self.samples.len() >= 2 {
            if let (Some(back), Some(front)) = (self.samples.back(), self.samples.front()) {
                let time_span = back.timestamp.duration_since(front.timestamp);
                if time_span.as_secs_f32() > 0.0 {
                    (self.samples.len() - 1) as f32 / time_span.as_secs_f32()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Aggregate timing statistics
        let total_frame_time: Duration = self.samples.iter().map(|s| s.frame_time).sum();
        let avg_frame_time = total_frame_time / self.samples.len() as u32;

        let min_frame_time = self
            .samples
            .iter()
            .map(|s| s.frame_time)
            .min()
            .unwrap_or(Duration::ZERO);

        let max_frame_time = self
            .samples
            .iter()
            .map(|s| s.frame_time)
            .max()
            .unwrap_or(Duration::ZERO);

        // Calculate efficiency ratio
        let total_updated: u32 = self.samples.iter().map(|s| s.cells_updated).sum();
        let total_cells: u32 = self.samples.iter().map(|s| s.cells_total).sum();
        let efficiency_ratio = if total_cells > 0 {
            total_updated as f32 / total_cells as f32
        } else {
            0.0
        };

        // Average bytes per frame
        let total_bytes: u64 = self.samples.iter().map(|s| s.bytes_written).sum();
        let avg_bytes_per_frame = total_bytes as f64 / sample_count as f64;

        // Average timing breakdowns
        let total_write_time: Duration = self.samples.iter().map(|s| s.write_time).sum();
        let avg_write_time = total_write_time / self.samples.len() as u32;

        let total_diff_time: Duration = self.samples.iter().map(|s| s.diff_time).sum();
        let avg_diff_time = total_diff_time / self.samples.len() as u32;

        // Memory usage (use latest sample)
        let memory_usage = self.samples.back().map(|s| s.memory_usage).unwrap_or(0);

        // Stability score (inverse of coefficient of variation)
        let stability_score = if self.samples.len() > 1 {
            let mean = avg_frame_time.as_secs_f32();
            let variance: f32 = self
                .samples
                .iter()
                .map(|s| {
                    let diff = s.frame_time.as_secs_f32() - mean;
                    diff * diff
                })
                .sum::<f32>()
                / sample_count;
            let std_dev = variance.sqrt();
            let cv = if mean > 0.0 { std_dev / mean } else { 1.0 };
            (1.0 - cv.min(1.0)).max(0.0)
        } else {
            1.0
        };

        // Component metrics calculations
        let total_component_creation_time: Duration =
            self.samples.iter().map(|s| s.component_creation_time).sum();
        let avg_component_creation_time = if sample_count > 0.0 {
            total_component_creation_time / sample_count as u32
        } else {
            Duration::ZERO
        };

        let total_component_cleanup_time: Duration =
            self.samples.iter().map(|s| s.component_cleanup_time).sum();
        let avg_component_cleanup_time = if sample_count > 0.0 {
            total_component_cleanup_time / sample_count as u32
        } else {
            Duration::ZERO
        };

        // Current active components (from latest sample)
        let active_components = self
            .samples
            .back()
            .map(|s| s.active_components)
            .unwrap_or(0);

        // Total components created/destroyed across all samples
        let total_components_created: u64 = self
            .samples
            .iter()
            .map(|s| s.components_created as u64)
            .sum();
        let total_components_destroyed: u64 = self
            .samples
            .iter()
            .map(|s| s.components_destroyed as u64)
            .sum();

        // Component creation/cleanup rates (per second)
        let time_span_secs = if self.samples.len() > 1 {
            if let (Some(back), Some(front)) = (self.samples.back(), self.samples.front()) {
                back.timestamp.duration_since(front.timestamp).as_secs_f32()
            } else {
                1.0 // Fallback if samples are somehow missing
            }
        } else {
            1.0 // Avoid division by zero
        };

        let component_creation_rate = total_components_created as f32 / time_span_secs;
        let component_cleanup_rate = total_components_destroyed as f32 / time_span_secs;

        PerformanceMetrics {
            fps,
            avg_frame_time,
            min_frame_time,
            max_frame_time,
            efficiency_ratio,
            avg_bytes_per_frame,
            avg_write_time,
            avg_diff_time,
            total_frames: self.total_frames,
            memory_usage,
            stability_score,
            avg_component_creation_time,
            avg_component_cleanup_time,
            active_components,
            total_components_created,
            total_components_destroyed,
            component_creation_rate,
            component_cleanup_rate,
        }
    }

    /// Get the most recent frame stats
    pub fn latest_frame(&self) -> Option<&DetailedFrameStats> {
        self.samples.back()
    }

    /// Get all samples in the current window
    pub fn samples(&self) -> &VecDeque<DetailedFrameStats> {
        &self.samples
    }

    /// Clear all collected statistics
    pub fn clear(&mut self) {
        self.samples.clear();
        self.total_frames = 0;
        self.start_time = Instant::now();
    }

    /// Update component metrics for the current frame
    pub fn update_component_metrics(
        &mut self,
        components_created: u32,
        components_destroyed: u32,
        active_components: u32,
        creation_time: Duration,
        cleanup_time: Duration,
    ) {
        if let Some(latest) = self.samples.back_mut() {
            latest.components_created += components_created;
            latest.components_destroyed += components_destroyed;
            latest.active_components = active_components;
            latest.component_creation_time += creation_time;
            latest.component_cleanup_time += cleanup_time;
        }
    }

    /// Get component performance summary
    pub fn component_performance_summary(&self) -> ComponentPerformanceSummary {
        let metrics = self.get_metrics();

        ComponentPerformanceSummary {
            avg_creation_time: metrics.avg_component_creation_time,
            avg_cleanup_time: metrics.avg_component_cleanup_time,
            active_components: metrics.active_components,
            creation_rate: metrics.component_creation_rate,
            cleanup_rate: metrics.component_cleanup_rate,
            total_created: metrics.total_components_created,
            total_destroyed: metrics.total_components_destroyed,
            creation_efficiency: if metrics.avg_component_creation_time.as_nanos() > 0 {
                1_000_000.0 / metrics.avg_component_creation_time.as_nanos() as f32
            // components per millisecond
            } else {
                f32::INFINITY
            },
        }
    }
}

/// Component performance summary
#[derive(Debug, Clone)]
pub struct ComponentPerformanceSummary {
    /// Average time to create a component
    pub avg_creation_time: Duration,
    /// Average time to cleanup a component
    pub avg_cleanup_time: Duration,
    /// Current number of active components
    pub active_components: u32,
    /// Component creation rate (per second)
    pub creation_rate: f32,
    /// Component cleanup rate (per second)
    pub cleanup_rate: f32,
    /// Total components created
    pub total_created: u64,
    /// Total components destroyed
    pub total_destroyed: u64,
    /// Creation efficiency (components per millisecond)
    pub creation_efficiency: f32,
}

impl RenderStatsCollector {
    /// Get uptime since collector was created
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Check if performance is considered good
    pub fn is_performance_good(&self) -> bool {
        let metrics = self.get_metrics();
        metrics.fps >= 30.0
            && metrics.stability_score >= 0.8
            && metrics.avg_frame_time.as_millis() <= 33 // ~30 FPS
    }

    /// Get a performance grade (A-F)
    pub fn performance_grade(&self) -> char {
        let metrics = self.get_metrics();

        if metrics.fps >= 60.0 && metrics.stability_score >= 0.9 {
            'A'
        } else if metrics.fps >= 45.0 && metrics.stability_score >= 0.8 {
            'B'
        } else if metrics.fps >= 30.0 && metrics.stability_score >= 0.7 {
            'C'
        } else if metrics.fps >= 20.0 {
            'D'
        } else {
            'F'
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_collector_basic() {
        let mut collector = RenderStatsCollector::new(10);

        let stats = DetailedFrameStats {
            frame_time: Duration::from_millis(16),
            cells_updated: 100,
            cells_total: 1000,
            bytes_written: 500,
            ..Default::default()
        };

        collector.record_frame(stats);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_frames, 1);
        assert_eq!(metrics.efficiency_ratio, 0.1);
    }

    #[test]
    fn test_fps_calculation() {
        let mut collector = RenderStatsCollector::new(10);
        let start = Instant::now();

        // Add frames with 16ms intervals (should be ~60 FPS)
        for i in 0..5 {
            let stats = DetailedFrameStats {
                frame_time: Duration::from_millis(16),
                timestamp: start + Duration::from_millis(i * 16),
                ..Default::default()
            };
            collector.record_frame(stats);
        }

        let metrics = collector.get_metrics();
        // Should be approximately 60 FPS (allowing for some variance)
        assert!(metrics.fps > 50.0 && metrics.fps < 70.0);
    }
}
