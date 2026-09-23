//! Stagger animation system for delayed sequences
//!
//! Provides anime.js-inspired stagger animations with multiple origin points,
//! 2D grid support, easing, and range modifiers.

use super::EasingFunction;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Stagger animation configuration for creating delayed animation sequences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StaggerConfig {
    /// Base delay between elements
    pub delay: Duration,
    /// Starting point for stagger
    pub from: StaggerOrigin,
    /// Direction of stagger
    pub direction: StaggerDirection,
    /// Easing for delay calculation
    pub ease: Option<EasingFunction>,
    /// Grid dimensions (for 2D stagger)
    pub grid: Option<(usize, usize)>,
    /// Range multiplier for delays
    pub range: Option<(f32, f32)>,
}

impl Default for StaggerConfig {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(100),
            from: StaggerOrigin::First,
            direction: StaggerDirection::Normal,
            ease: None,
            grid: None,
            range: None,
        }
    }
}

/// Starting point for stagger animations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerOrigin {
    /// Start from first element
    First,
    /// Start from last element
    Last,
    /// Start from center element
    Center,
    /// Random starting point
    Random,
    /// Start from specific index
    Index(usize),
    /// Start from specific position (for 2D grid)
    Position(i16, i16),
}

/// Direction of stagger progression
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StaggerDirection {
    /// Normal progression (forward)
    Normal,
    /// Reverse progression (backward)
    Reverse,
    /// Random order
    Random,
}

impl StaggerConfig {
    /// Calculate delays for a list of elements
    pub fn calculate_delays(&self, target_count: usize, positions: &[(i16, i16)]) -> Vec<Duration> {
        let mut delays = match self.from {
            StaggerOrigin::First => self.calculate_linear_delays(target_count, false),
            StaggerOrigin::Last => self.calculate_linear_delays(target_count, true),
            StaggerOrigin::Center => self.calculate_center_delays(target_count),
            StaggerOrigin::Random => self.calculate_random_delays(target_count),
            StaggerOrigin::Index(start_index) => {
                self.calculate_index_delays(target_count, start_index)
            }
            StaggerOrigin::Position(x, y) => {
                if positions.is_empty() {
                    self.calculate_linear_delays(target_count, false)
                } else {
                    self.calculate_position_delays(positions, x, y)
                }
            }
        };

        // Apply direction
        match self.direction {
            StaggerDirection::Reverse => delays.reverse(),
            StaggerDirection::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                // Create a deterministic but pseudo-random shuffle
                let mut indices: Vec<usize> = (0..delays.len()).collect();
                for i in 0..indices.len() {
                    let mut hasher = DefaultHasher::new();
                    i.hash(&mut hasher);
                    let hash = hasher.finish();
                    let j = (hash as usize) % indices.len();
                    indices.swap(i, j);
                }

                let original_delays = delays.clone();
                for (i, &original_index) in indices.iter().enumerate() {
                    delays[i] = original_delays[original_index];
                }
            }
            StaggerDirection::Normal => {} // Already in normal order
        }

        // Apply easing if specified
        if let Some(ease) = &self.ease {
            let max_delay = delays.iter().max().cloned().unwrap_or(Duration::ZERO);
            if max_delay > Duration::ZERO {
                for delay in &mut delays {
                    let t = delay.as_secs_f32() / max_delay.as_secs_f32();
                    let eased_t = ease.apply(t);
                    *delay = Duration::from_secs_f32(max_delay.as_secs_f32() * eased_t);
                }
            }
        }

        // Apply range if specified
        if let Some((min_range, max_range)) = self.range {
            let base_delay = self.delay.as_secs_f32();
            for delay in &mut delays {
                let factor = min_range
                    + (max_range - min_range) * (delay.as_secs_f32() / base_delay).clamp(0.0, 1.0);
                *delay = Duration::from_secs_f32(base_delay * factor);
            }
        }

        delays
    }

    /// Calculate delays for 2D grid layout
    pub fn calculate_grid_delays(&self, grid_width: usize, grid_height: usize) -> Vec<Duration> {
        let total_elements = grid_width * grid_height;
        let mut delays = Vec::with_capacity(total_elements);

        if let Some((gw, gh)) = self.grid {
            // Use provided grid dimensions
            let grid_w = gw.min(grid_width);
            let grid_h = gh.min(grid_height);

            for y in 0..grid_h {
                for x in 0..grid_w {
                    let delay = match self.from {
                        StaggerOrigin::First => Duration::from_secs_f32(
                            self.delay.as_secs_f32() * (y * grid_w + x) as f32,
                        ),
                        StaggerOrigin::Center => {
                            let center_x = grid_w as f32 / 2.0;
                            let center_y = grid_h as f32 / 2.0;
                            let distance = ((x as f32 - center_x).powi(2)
                                + (y as f32 - center_y).powi(2))
                            .sqrt();
                            Duration::from_secs_f32(self.delay.as_secs_f32() * distance)
                        }
                        StaggerOrigin::Position(px, py) => {
                            let distance = ((x as i16 - px).pow(2) + (y as i16 - py).pow(2)) as f32;
                            Duration::from_secs_f32(
                                self.delay.as_secs_f32() * distance.sqrt() / 10.0,
                            )
                        }
                        _ => Duration::from_secs_f32(
                            self.delay.as_secs_f32() * (y * grid_w + x) as f32,
                        ),
                    };
                    delays.push(delay);
                }
            }
        } else {
            // Fallback to linear delays
            delays = self.calculate_linear_delays(total_elements, false);
        }

        // Apply direction
        match self.direction {
            StaggerDirection::Reverse => delays.reverse(),
            StaggerDirection::Random => {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                // Create a deterministic but pseudo-random shuffle
                let mut indices: Vec<usize> = (0..delays.len()).collect();
                for i in 0..indices.len() {
                    let mut hasher = DefaultHasher::new();
                    i.hash(&mut hasher);
                    let hash = hasher.finish();
                    let j = (hash as usize) % indices.len();
                    indices.swap(i, j);
                }

                let original_delays = delays.clone();
                for (i, &original_index) in indices.iter().enumerate() {
                    delays[i] = original_delays[original_index];
                }
            }
            StaggerDirection::Normal => {} // Already in normal order
        }

        // Apply easing if specified
        if let Some(ease) = &self.ease {
            let max_delay = delays.iter().max().cloned().unwrap_or(Duration::ZERO);
            if max_delay > Duration::ZERO {
                for delay in &mut delays {
                    let t = delay.as_secs_f32() / max_delay.as_secs_f32();
                    let eased_t = ease.apply(t);
                    *delay = Duration::from_secs_f32(max_delay.as_secs_f32() * eased_t);
                }
            }
        }

        // Apply range if specified
        if let Some((min_range, max_range)) = self.range {
            let base_delay = self.delay.as_secs_f32();
            for delay in &mut delays {
                let factor = min_range
                    + (max_range - min_range) * (delay.as_secs_f32() / base_delay).clamp(0.0, 1.0);
                *delay = Duration::from_secs_f32(base_delay * factor);
            }
        }

        delays
    }

    // Helper methods for delay calculation
    fn calculate_linear_delays(&self, count: usize, reverse: bool) -> Vec<Duration> {
        let mut delays = Vec::with_capacity(count);
        for i in 0..count {
            let index = if reverse { count - 1 - i } else { i };
            delays.push(Duration::from_secs_f32(
                self.delay.as_secs_f32() * index as f32,
            ));
        }
        delays
    }

    fn calculate_center_delays(&self, count: usize) -> Vec<Duration> {
        let center = count as f32 / 2.0;
        (0..count)
            .map(|i| {
                let distance = (i as f32 - center).abs();
                Duration::from_secs_f32(self.delay.as_secs_f32() * distance)
            })
            .collect()
    }

    fn calculate_random_delays(&self, count: usize) -> Vec<Duration> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        (0..count)
            .map(|i| {
                let mut hasher = DefaultHasher::new();
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let random_factor = (hash % 1000) as f32 / 1000.0; // 0.0 to 1.0
                Duration::from_secs_f32(self.delay.as_secs_f32() * random_factor * count as f32)
            })
            .collect()
    }

    fn calculate_index_delays(&self, count: usize, start_index: usize) -> Vec<Duration> {
        let start = start_index.min(count.saturating_sub(1));
        (0..count)
            .map(|i| {
                let distance = i.abs_diff(start);
                Duration::from_secs_f32(self.delay.as_secs_f32() * distance as f32)
            })
            .collect()
    }

    fn calculate_position_delays(
        &self,
        positions: &[(i16, i16)],
        start_x: i16,
        start_y: i16,
    ) -> Vec<Duration> {
        positions
            .iter()
            .map(|(x, y)| {
                let distance = (((x - start_x).pow(2) + (y - start_y).pow(2)) as f32).sqrt();
                Duration::from_secs_f32(self.delay.as_secs_f32() * distance / 100.0)
            })
            .collect()
    }
}

/// Builder for creating complex stagger configurations
pub struct StaggerBuilder {
    config: StaggerConfig,
}

impl StaggerBuilder {
    /// Create a new stagger builder with delay in milliseconds
    pub fn new(delay_ms: u64) -> Self {
        Self {
            config: StaggerConfig {
                delay: Duration::from_millis(delay_ms),
                ..Default::default()
            },
        }
    }

    /// Set the origin point for the stagger animation
    pub fn from(mut self, origin: StaggerOrigin) -> Self {
        self.config.from = origin;
        self
    }

    /// Set the direction of the stagger animation
    pub fn direction(mut self, direction: StaggerDirection) -> Self {
        self.config.direction = direction;
        self
    }

    /// Set the easing function for the stagger animation
    pub fn ease(mut self, easing: EasingFunction) -> Self {
        self.config.ease = Some(easing);
        self
    }

    /// Set grid dimensions for grid-based stagger animations
    pub fn grid(mut self, width: usize, height: usize) -> Self {
        self.config.grid = Some((width, height));
        self
    }

    /// Set the range of values for the stagger animation
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.config.range = Some((min, max));
        self
    }

    /// Build the final stagger configuration
    pub fn build(self) -> StaggerConfig {
        self.config
    }
}

// Convenience functions for creating stagger configurations

/// Create a basic stagger with delay
pub fn stagger(delay_ms: u64) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        ..Default::default()
    }
}

/// Create a stagger starting from center
pub fn stagger_from_center(delay_ms: u64) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        from: StaggerOrigin::Center,
        ..Default::default()
    }
}

/// Create a stagger starting from last element
pub fn stagger_from_last(delay_ms: u64) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        from: StaggerOrigin::Last,
        ..Default::default()
    }
}

/// Create a stagger starting from specific index
pub fn stagger_from_index(delay_ms: u64, index: usize) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        from: StaggerOrigin::Index(index),
        ..Default::default()
    }
}

/// Create a stagger starting from specific position (for 2D layouts)
pub fn stagger_from_position(delay_ms: u64, x: i16, y: i16) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        from: StaggerOrigin::Position(x, y),
        ..Default::default()
    }
}

/// Create a random stagger
pub fn stagger_random(delay_ms: u64) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        from: StaggerOrigin::Random,
        direction: StaggerDirection::Random,
        ..Default::default()
    }
}

/// Create a 2D grid stagger
pub fn stagger_grid(delay_ms: u64, width: usize, height: usize) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        grid: Some((width, height)),
        from: StaggerOrigin::First,
        ..Default::default()
    }
}

/// Create a 2D grid stagger from center
pub fn stagger_grid_center(delay_ms: u64, width: usize, height: usize) -> StaggerConfig {
    StaggerConfig {
        delay: Duration::from_millis(delay_ms),
        grid: Some((width, height)),
        from: StaggerOrigin::Center,
        ..Default::default()
    }
}

/// Create a stagger builder
pub fn stagger_builder(delay_ms: u64) -> StaggerBuilder {
    StaggerBuilder::new(delay_ms)
}
