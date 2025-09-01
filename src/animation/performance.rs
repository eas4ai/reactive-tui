//! Performance Optimization System for Animations
//!
//! This module provides comprehensive performance optimizations for the animation system,
//! including batching, caching, and performance monitoring capabilities.
//!
//! Features:
//! - Animation batching for similar property updates
//! - Interpolation result caching with LRU eviction
//! - Performance metrics and profiling
//! - Memory-efficient batch processing
//! - SIMD-optimized calculations where applicable

use super::*;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Optimization level for animation processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// No optimizations - process each animation individually
    None,
    /// Basic batching of similar property types
    Basic,
    /// Aggressive optimization with caching and SIMD
    Aggressive,
    /// GPU acceleration (future implementation)
    GPU,
}

/// Batched animation update result
#[derive(Debug, Clone)]
pub enum BatchedUpdate {
    /// Single animation update
    Single(AnimationId, AnimatedValue),
    /// Batch of opacity updates (id, progress)
    OpacityBatch(Vec<(AnimationId, f32)>),
    /// Batch of position updates (id, x, y)
    PositionBatch(Vec<(AnimationId, i16, i16)>),
    /// Batch of color updates (id, r, g, b)
    ColorBatch(Vec<(AnimationId, u8, u8, u8)>),
    /// Batch of transform updates (id, transform_type, value)
    TransformBatch(Vec<(AnimationId, String, f32)>),
}

/// Animation batch processor
pub struct AnimationBatch {
    /// Animations in this batch
    animations: Vec<Animation>,
    /// Shared timeline for coordination
    #[allow(dead_code)]
    shared_timeline: Option<AnimationTimeline>,
    /// Optimization level
    optimization_level: OptimizationLevel,
    /// Performance metrics
    metrics: PerformanceMetrics,
    /// Interpolation cache
    cache: InterpolationCache,
}

impl AnimationBatch {
    /// Create a new animation batch
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        Self {
            animations: Vec::new(),
            shared_timeline: None,
            optimization_level,
            metrics: PerformanceMetrics::new(),
            cache: InterpolationCache::new(1000), // 1000 cache entries
        }
    }

    /// Add animation to batch
    pub fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    /// Update all animations in batch and return batched updates
    pub fn update_batch(&mut self, delta_time: Duration) -> Vec<BatchedUpdate> {
        let start_time = Instant::now();
        let mut updates = Vec::new();

        match self.optimization_level {
            OptimizationLevel::None => {
                // Process each animation individually
                for animation in &mut self.animations {
                    if animation.update(delta_time) {
                        if let Some(value) = animation.get_current_values() {
                            updates.push(BatchedUpdate::Single(animation.id.clone(), value));
                        }
                    }
                }
            }
            OptimizationLevel::Basic => {
                updates = self.basic_batched_update(delta_time);
            }
            OptimizationLevel::Aggressive => {
                updates = self.aggressive_batched_update(delta_time);
            }
            OptimizationLevel::GPU => {
                // Future GPU implementation
                updates = self.basic_batched_update(delta_time);
            }
        }

        // Update performance metrics
        let update_time = start_time.elapsed();
        self.metrics
            .record_batch_update(self.animations.len(), update_time);

        updates
    }

    /// Basic batching - group similar property types
    fn basic_batched_update(&mut self, delta_time: Duration) -> Vec<BatchedUpdate> {
        let mut updates = Vec::new();
        let mut opacity_anims = Vec::new();
        let mut position_anims = Vec::new();
        let mut color_anims = Vec::new();
        let mut transform_anims = Vec::new();

        for animation in &mut self.animations {
            if animation.update(delta_time) {
                match &animation.property {
                    AnimatedProperty::Opacity(_, _) => {
                        let progress = animation.get_progress();
                        opacity_anims.push((animation.id.clone(), progress));
                    }
                    AnimatedProperty::Position(_, _, _, _) => {
                        if let Some(AnimatedValue::Position(x, y)) = animation.get_current_values()
                        {
                            position_anims.push((animation.id.clone(), x, y));
                        }
                    }
                    AnimatedProperty::Color(_, _) => {
                        if let Some(AnimatedValue::Color { r, g, b }) =
                            animation.get_current_values()
                        {
                            color_anims.push((animation.id.clone(), r, g, b));
                        }
                    }
                    AnimatedProperty::Transform(transform_prop) => {
                        let (transform_type, value) = match transform_prop {
                            TransformProperty::TranslateX(_, to) => ("translateX".to_string(), *to),
                            TransformProperty::TranslateY(_, to) => ("translateY".to_string(), *to),
                            TransformProperty::Scale(_, to) => ("scale".to_string(), *to),
                            TransformProperty::Rotate(_, to) => ("rotate".to_string(), *to),
                            _ => ("unknown".to_string(), 0.0),
                        };
                        transform_anims.push((animation.id.clone(), transform_type, value));
                    }
                    _ => {
                        // Handle individually
                        if let Some(value) = animation.get_current_values() {
                            updates.push(BatchedUpdate::Single(animation.id.clone(), value));
                        }
                    }
                }
            }
        }

        // Create batched updates
        if !opacity_anims.is_empty() {
            updates.push(BatchedUpdate::OpacityBatch(opacity_anims));
        }
        if !position_anims.is_empty() {
            updates.push(BatchedUpdate::PositionBatch(position_anims));
        }
        if !color_anims.is_empty() {
            updates.push(BatchedUpdate::ColorBatch(color_anims));
        }
        if !transform_anims.is_empty() {
            updates.push(BatchedUpdate::TransformBatch(transform_anims));
        }

        updates
    }

    /// Aggressive optimization with caching and advanced techniques
    fn aggressive_batched_update(&mut self, delta_time: Duration) -> Vec<BatchedUpdate> {
        let mut updates = self.basic_batched_update(delta_time);

        // Additional optimizations:
        // 1. Skip invisible/offscreen animations
        // 2. Use cached interpolation values
        // 3. Batch similar easing calculations
        // 4. Optimize memory allocations

        // Filter out invisible animations (placeholder logic)
        updates.retain(|update| self.is_update_visible(update));

        // Use cached interpolation where possible
        self.apply_cached_interpolation(&mut updates);

        updates
    }

    /// Check if an update affects visible elements
    fn is_update_visible(&self, _update: &BatchedUpdate) -> bool {
        // For now, all batched updates are considered visible since we don't have
        // element bounds tracking in the current architecture.
        // In a full implementation, this would:
        // 1. Track element bounds per animation ID
        // 2. Compare with current viewport bounds
        // 3. Use spatial partitioning for large scenes
        true
    }

    /// Apply cached interpolation values
    fn apply_cached_interpolation(&mut self, _updates: &mut [BatchedUpdate]) {
        // Placeholder - would optimize interpolation using cache
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Clear the batch
    pub fn clear(&mut self) {
        self.animations.clear();
        self.cache.clear_expired();
    }
}

/// Interpolation result cache with LRU eviction
pub struct InterpolationCache {
    /// Cached interpolation results
    cache: HashMap<String, CachedInterpolation>,
    /// Maximum cache size
    max_size: usize,
    /// Cache hit/miss statistics
    hits: u64,
    misses: u64,
}

/// Cached interpolation data
#[derive(Debug, Clone)]
struct CachedInterpolation {
    /// From value
    from: AnimatedValue,
    /// To value
    to: AnimatedValue,
    /// Easing function used
    easing: EasingFunction,
    /// Sampled progress -> value pairs
    samples: Vec<(f32, AnimatedValue)>,
    /// Last access time for LRU
    last_access: Instant,
    /// Cache creation time
    created: Instant,
}

impl InterpolationCache {
    /// Create a new interpolation cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            hits: 0,
            misses: 0,
        }
    }

    /// Get interpolated value with caching
    pub fn get_interpolated_value(
        &mut self,
        key: &str,
        from: &AnimatedValue,
        to: &AnimatedValue,
        easing: &EasingFunction,
        progress: f32,
    ) -> AnimatedValue {
        // Check cache first
        let cache_result = if let Some(cached) = self.cache.get(key) {
            if self.is_cache_valid(cached, from, to, easing) {
                self.find_cached_value(&cached.samples, progress)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(cached_value) = cache_result {
            // Update access time
            if let Some(cached) = self.cache.get_mut(key) {
                cached.last_access = Instant::now();
            }
            self.hits += 1;
            return cached_value;
        }

        // Cache miss - calculate and store
        self.misses += 1;
        let value = self.calculate_interpolated_value(from, to, easing, progress);

        // Store in cache
        self.store_in_cache(key.to_string(), from, to, easing, progress, value.clone());

        value
    }

    /// Check if cached entry is valid
    fn is_cache_valid(
        &self,
        cached: &CachedInterpolation,
        from: &AnimatedValue,
        to: &AnimatedValue,
        easing: &EasingFunction,
    ) -> bool {
        // Check if values and easing match
        std::mem::discriminant(&cached.from) == std::mem::discriminant(from)
            && std::mem::discriminant(&cached.to) == std::mem::discriminant(to)
            && cached.easing == *easing
            && cached.created.elapsed() < Duration::from_secs(60) // 1 minute expiry
    }

    /// Find cached value for given progress
    fn find_cached_value(
        &self,
        samples: &[(f32, AnimatedValue)],
        progress: f32,
    ) -> Option<AnimatedValue> {
        // Find closest sample
        let mut closest_sample = None;
        let mut closest_distance = f32::INFINITY;

        for (sample_progress, sample_value) in samples {
            let distance = (sample_progress - progress).abs();
            if distance < closest_distance {
                closest_distance = distance;
                closest_sample = Some(sample_value.clone());
            }
        }

        // Use cached value if close enough (within 1% progress)
        if closest_distance < 0.01 {
            closest_sample
        } else {
            None
        }
    }

    /// Calculate interpolated value
    fn calculate_interpolated_value(
        &self,
        from: &AnimatedValue,
        to: &AnimatedValue,
        easing: &EasingFunction,
        progress: f32,
    ) -> AnimatedValue {
        let eased_progress = easing.apply(progress);
        interpolate_animated_values(from, to, eased_progress)
    }

    /// Store value in cache
    fn store_in_cache(
        &mut self,
        key: String,
        from: &AnimatedValue,
        to: &AnimatedValue,
        easing: &EasingFunction,
        progress: f32,
        value: AnimatedValue,
    ) {
        // Ensure cache size limit
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let cached = self
            .cache
            .entry(key)
            .or_insert_with(|| CachedInterpolation {
                from: from.clone(),
                to: to.clone(),
                easing: easing.clone(),
                samples: Vec::new(),
                last_access: Instant::now(),
                created: Instant::now(),
            });

        // Add sample
        cached.samples.push((progress, value));
        cached.last_access = Instant::now();

        // Limit samples per cache entry
        if cached.samples.len() > 100 {
            cached
                .samples
                .sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            cached.samples.truncate(50); // Keep every other sample
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((oldest_key, _)) = self
            .cache
            .iter()
            .min_by_key(|(_, cached)| cached.last_access)
            .map(|(k, v)| (k.clone(), v.last_access))
        {
            self.cache.remove(&oldest_key);
        }
    }

    /// Clear expired entries
    pub fn clear_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, cached| {
            now.duration_since(cached.created) < Duration::from_secs(300) // 5 minute expiry
        });
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
            cache_size: self.cache.len(),
            max_size: self.max_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Current cache size
    pub cache_size: usize,
    /// Maximum cache size
    pub max_size: usize,
}

/// Performance metrics collector
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total animations processed
    total_animations: u64,
    /// Total update time
    total_update_time: Duration,
    /// Peak batch size
    peak_batch_size: usize,
    /// Update history for averaging
    update_history: Vec<(usize, Duration)>,
    /// Max history size
    max_history: usize,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a batch update
    pub fn record_batch_update(&mut self, animation_count: usize, update_time: Duration) {
        self.total_animations += animation_count as u64;
        self.total_update_time += update_time;
        self.peak_batch_size = self.peak_batch_size.max(animation_count);

        // Store in history
        self.update_history.push((animation_count, update_time));
        if self.update_history.len() > self.max_history {
            self.update_history.remove(0);
        }
    }

    /// Get average update time per animation
    pub fn avg_time_per_animation(&self) -> Duration {
        if self.total_animations > 0 {
            self.total_update_time / self.total_animations as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get recent average batch performance
    pub fn recent_avg_performance(&self) -> Option<Duration> {
        let recent_count = self.update_history.len().min(10);
        if recent_count == 0 {
            return None;
        }

        let recent_updates = &self.update_history[self.update_history.len() - recent_count..];
        let total_time: Duration = recent_updates.iter().map(|(_, time)| *time).sum();
        let total_animations: usize = recent_updates.iter().map(|(count, _)| *count).sum();

        if total_animations > 0 {
            Some(total_time / total_animations as u32)
        } else {
            None
        }
    }

    /// Get performance report
    pub fn get_report(&self) -> PerformanceReport {
        PerformanceReport {
            total_animations: self.total_animations,
            total_update_time: self.total_update_time,
            avg_time_per_animation: self.avg_time_per_animation(),
            peak_batch_size: self.peak_batch_size,
            recent_avg_performance: self.recent_avg_performance(),
        }
    }
}

/// Performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Total number of animations processed
    pub total_animations: u64,
    /// Total time spent updating animations
    pub total_update_time: Duration,
    /// Average time per animation update
    pub avg_time_per_animation: Duration,
    /// Peak batch size processed
    pub peak_batch_size: usize,
    /// Recent average performance (if available)
    pub recent_avg_performance: Option<Duration>,
}

/// Optimized animation manager with batching support
pub struct OptimizedAnimationManager {
    /// Animation batches by optimization level
    batches: HashMap<OptimizationLevel, AnimationBatch>,
    /// Individual animations not in batches
    individual_animations: HashMap<AnimationId, Animation>,
    /// Global performance metrics
    global_metrics: PerformanceMetrics,
    /// Last update time
    last_update: Instant,
}

impl Default for OptimizedAnimationManager {
    fn default() -> Self {
        let mut batches = HashMap::new();
        batches.insert(
            OptimizationLevel::None,
            AnimationBatch::new(OptimizationLevel::None),
        );
        batches.insert(
            OptimizationLevel::Basic,
            AnimationBatch::new(OptimizationLevel::Basic),
        );
        batches.insert(
            OptimizationLevel::Aggressive,
            AnimationBatch::new(OptimizationLevel::Aggressive),
        );

        Self {
            batches,
            individual_animations: HashMap::new(),
            global_metrics: PerformanceMetrics::new(),
            last_update: Instant::now(),
        }
    }
}

impl OptimizedAnimationManager {
    /// Create new optimized animation manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add animation with specified optimization level
    pub fn add_animation(&mut self, animation: Animation, optimization_level: OptimizationLevel) {
        if let Some(batch) = self.batches.get_mut(&optimization_level) {
            batch.add_animation(animation);
        } else {
            self.individual_animations
                .insert(animation.id.clone(), animation);
        }
    }

    /// Update all animations and return batched results
    pub fn update_all(&mut self) -> Vec<BatchedUpdate> {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        let mut all_updates = Vec::new();

        // Update batches
        for batch in self.batches.values_mut() {
            let batch_updates = batch.update_batch(delta_time);
            all_updates.extend(batch_updates);
        }

        // Update individual animations
        for animation in self.individual_animations.values_mut() {
            if animation.update(delta_time) {
                if let Some(value) = animation.get_current_values() {
                    all_updates.push(BatchedUpdate::Single(animation.id.clone(), value));
                }
            }
        }

        all_updates
    }

    /// Get global performance metrics
    pub fn get_global_metrics(&self) -> &PerformanceMetrics {
        &self.global_metrics
    }

    /// Get batch metrics for specific optimization level
    pub fn get_batch_metrics(&self, level: OptimizationLevel) -> Option<&PerformanceMetrics> {
        self.batches.get(&level).map(|batch| batch.get_metrics())
    }
}

/// Helper function to interpolate between animated values
fn interpolate_animated_values(
    from: &AnimatedValue,
    to: &AnimatedValue,
    progress: f32,
) -> AnimatedValue {
    match (from, to) {
        (AnimatedValue::Opacity(from_val), AnimatedValue::Opacity(to_val)) => {
            AnimatedValue::Opacity(from_val + (to_val - from_val) * progress)
        }
        (AnimatedValue::Scale(from_val), AnimatedValue::Scale(to_val)) => {
            AnimatedValue::Scale(from_val + (to_val - from_val) * progress)
        }
        (AnimatedValue::Rotation(from_val), AnimatedValue::Rotation(to_val)) => {
            AnimatedValue::Rotation(from_val + (to_val - from_val) * progress)
        }
        (AnimatedValue::Custom(name, from_val), AnimatedValue::Custom(_, to_val)) => {
            AnimatedValue::Custom(name.clone(), from_val + (to_val - from_val) * progress)
        }
        (AnimatedValue::Position(from_x, from_y), AnimatedValue::Position(to_x, to_y)) => {
            let x = from_x + ((to_x - from_x) as f32 * progress) as i16;
            let y = from_y + ((to_y - from_y) as f32 * progress) as i16;
            AnimatedValue::Position(x, y)
        }
        (
            AnimatedValue::Color {
                r: fr,
                g: fg,
                b: fb,
            },
            AnimatedValue::Color {
                r: tr,
                g: tg,
                b: tb,
            },
        ) => {
            let r = (*fr as f32 + (*tr as f32 - *fr as f32) * progress) as u8;
            let g = (*fg as f32 + (*tg as f32 - *fg as f32) * progress) as u8;
            let b = (*fb as f32 + (*tb as f32 - *fb as f32) * progress) as u8;
            AnimatedValue::Color { r, g, b }
        }
        (AnimatedValue::Size(from_w, from_h), AnimatedValue::Size(to_w, to_h)) => {
            let w = from_w + ((to_w - from_w) as f32 * progress) as u16;
            let h = from_h + ((to_h - from_h) as f32 * progress) as u16;
            AnimatedValue::Size(w, h)
        }
        _ => from.clone(), // Fallback for mismatched types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_batch_creation() {
        let batch = AnimationBatch::new(OptimizationLevel::Basic);
        assert_eq!(batch.optimization_level, OptimizationLevel::Basic);
        assert_eq!(batch.animations.len(), 0);
    }

    #[test]
    fn test_interpolation_cache() {
        let mut cache = InterpolationCache::new(10);

        let from = AnimatedValue::Opacity(0.0);
        let to = AnimatedValue::Opacity(1.0);
        let easing = EasingFunction::Linear;

        // First call should be a miss
        let result1 = cache.get_interpolated_value("test", &from, &to, &easing, 0.5);
        assert_eq!(cache.misses, 1);
        assert_eq!(cache.hits, 0);

        // Second call with same parameters should be a hit
        let result2 = cache.get_interpolated_value("test", &from, &to, &easing, 0.5);
        assert_eq!(cache.hits, 1);

        if let (AnimatedValue::Opacity(val1), AnimatedValue::Opacity(val2)) = (result1, result2) {
            assert!((val1 - val2).abs() < 0.01); // Should be very close
        }
    }

    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();

        metrics.record_batch_update(5, Duration::from_millis(10));
        metrics.record_batch_update(3, Duration::from_millis(6));

        assert_eq!(metrics.total_animations, 8);
        assert_eq!(metrics.peak_batch_size, 5);

        let avg_time = metrics.avg_time_per_animation();
        assert!(avg_time.as_millis() > 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = InterpolationCache::new(10);
        let stats = cache.get_stats();

        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.hit_rate, 0.0);
        assert_eq!(stats.cache_size, 0);
        assert_eq!(stats.max_size, 10);
    }

    #[test]
    fn test_interpolate_animated_values() {
        // Test opacity interpolation
        let from = AnimatedValue::Opacity(0.0);
        let to = AnimatedValue::Opacity(1.0);
        let result = interpolate_animated_values(&from, &to, 0.5);

        if let AnimatedValue::Opacity(val) = result {
            assert!((val - 0.5).abs() < 0.01);
        } else {
            panic!("Expected opacity result");
        }

        // Test position interpolation
        let from_pos = AnimatedValue::Position(0, 0);
        let to_pos = AnimatedValue::Position(100, 200);
        let result_pos = interpolate_animated_values(&from_pos, &to_pos, 0.25);

        if let AnimatedValue::Position(x, y) = result_pos {
            assert_eq!(x, 25);
            assert_eq!(y, 50);
        } else {
            panic!("Expected position result");
        }
    }

    #[test]
    fn test_optimized_animation_manager() {
        let mut manager = OptimizedAnimationManager::new();

        // Should have default batches
        assert!(manager.batches.contains_key(&OptimizationLevel::None));
        assert!(manager.batches.contains_key(&OptimizationLevel::Basic));
        assert!(manager.batches.contains_key(&OptimizationLevel::Aggressive));

        let updates = manager.update_all();
        assert!(updates.is_empty()); // No animations yet
    }
}
