use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Region that needs re-rendering
#[derive(Debug, Clone, PartialEq)]
pub struct DirtyRegion {
    /// X coordinate of the dirty region
    pub x: u16,
    /// Y coordinate of the dirty region
    pub y: u16,
    /// Width of the dirty region
    pub width: u16,
    /// Height of the dirty region
    pub height: u16,
}

impl DirtyRegion {
    /// Create a new dirty region
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if this region contains a point
    pub fn contains_point(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Check if this region intersects another
    pub fn intersects(&self, other: &DirtyRegion) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Merge with another region to create bounding box
    pub fn merge(&self, other: &DirtyRegion) -> DirtyRegion {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.width).max(other.x + other.width);
        let y2 = (self.y + self.height).max(other.y + other.height);

        DirtyRegion::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Calculate area of the region
    pub fn area(&self) -> u32 {
        self.width as u32 * self.height as u32
    }
}

/// Manager for tracking dirty regions
pub struct DirtyRegionManager {
    regions: Vec<DirtyRegion>,
    screen_width: u16,
    screen_height: u16,
}

impl DirtyRegionManager {
    /// Create a new dirty region manager
    pub fn new(screen_width: u16, screen_height: u16) -> Self {
        Self {
            regions: Vec::new(),
            screen_width,
            screen_height,
        }
    }

    /// Add a dirty region
    pub fn add_region(&mut self, region: DirtyRegion) {
        // Clip to screen bounds
        let clipped = DirtyRegion {
            x: region.x.min(self.screen_width),
            y: region.y.min(self.screen_height),
            width: region.width.min(self.screen_width.saturating_sub(region.x)),
            height: region
                .height
                .min(self.screen_height.saturating_sub(region.y)),
        };

        if clipped.width > 0 && clipped.height > 0 {
            self.regions.push(clipped);
            self.optimize_regions();
        }
    }

    /// Mark entire screen as dirty
    pub fn invalidate_all(&mut self) {
        self.regions.clear();
        self.regions.push(DirtyRegion::new(
            0,
            0,
            self.screen_width,
            self.screen_height,
        ));
    }

    /// Get all dirty regions
    pub fn get_regions(&self) -> &[DirtyRegion] {
        &self.regions
    }

    /// Clear all dirty regions
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Check if a point is in any dirty region
    pub fn is_dirty(&self, x: u16, y: u16) -> bool {
        self.regions.iter().any(|r| r.contains_point(x, y))
    }

    /// Optimize regions by merging overlapping ones
    fn optimize_regions(&mut self) {
        if self.regions.len() < 2 {
            return;
        }

        // Simple optimization: merge overlapping regions
        let mut optimized = Vec::new();
        let mut merged = vec![false; self.regions.len()];

        for (i, region) in self.regions.iter().enumerate() {
            if merged[i] {
                continue;
            }

            let mut current = region.clone();

            for (j, other) in self.regions.iter().enumerate().skip(i + 1) {
                if merged[j] {
                    continue;
                }

                if current.intersects(other) {
                    // Check if merging is beneficial
                    let merged_area = current.merge(other).area();
                    let separate_area = current.area() + other.area();

                    // Merge if it doesn't waste too much space
                    if merged_area <= separate_area * 2 {
                        current = current.merge(other);
                        merged[j] = true;
                    }
                }
            }

            optimized.push(current);
        }

        self.regions = optimized;
    }
}

/// Cache key for rendered content
#[derive(Debug, Clone)]
struct CacheKey {
    component_id: String,
    props_hash: u64,
    state_hash: u64,
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.component_id.hash(state);
        self.props_hash.hash(state);
        self.state_hash.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.component_id == other.component_id
            && self.props_hash == other.props_hash
            && self.state_hash == other.state_hash
    }
}

impl Eq for CacheKey {}

/// Cached render output
#[derive(Clone)]
pub struct CachedRender {
    /// Rendered content as bytes
    pub content: Vec<u8>,
    /// Width of the rendered content
    pub width: u16,
    /// Height of the rendered content
    pub height: u16,
    /// When this render was cached
    pub timestamp: std::time::Instant,
}

/// Render cache for avoiding re-rendering unchanged components
pub struct RenderCache {
    cache: HashMap<CacheKey, CachedRender>,
    max_entries: usize,
    max_age: std::time::Duration,
    hits: usize,
    misses: usize,
}

impl RenderCache {
    /// Create a new render cache
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
            max_age: std::time::Duration::from_secs(60),
            hits: 0,
            misses: 0,
        }
    }

    /// Store rendered content in cache
    pub fn store(
        &mut self,
        component_id: String,
        props_hash: u64,
        state_hash: u64,
        content: Vec<u8>,
        width: u16,
        height: u16,
    ) {
        let key = CacheKey {
            component_id,
            props_hash,
            state_hash,
        };

        let entry = CachedRender {
            content,
            width,
            height,
            timestamp: std::time::Instant::now(),
        };

        // Evict old entries if at capacity
        if self.cache.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.cache.insert(key, entry);
    }

    /// Get cached content if available
    pub fn get(
        &mut self,
        component_id: &str,
        props_hash: u64,
        state_hash: u64,
    ) -> Option<CachedRender> {
        let key = CacheKey {
            component_id: component_id.to_string(),
            props_hash,
            state_hash,
        };

        // Check if entry exists and is valid
        let should_remove = if let Some(entry) = self.cache.get(&key) {
            entry.timestamp.elapsed() >= self.max_age
        } else {
            false
        };

        if should_remove {
            self.cache.remove(&key);
            self.misses += 1;
            return None;
        }

        if let Some(entry) = self.cache.get(&key) {
            self.hits += 1;
            Some(entry.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> String {
        let hit_rate = if self.hits + self.misses > 0 {
            (self.hits as f64 / (self.hits + self.misses) as f64) * 100.0
        } else {
            0.0
        };

        format!(
            "RenderCache: {} entries, {} hits, {} misses ({:.1}% hit rate)",
            self.cache.len(),
            self.hits,
            self.misses,
            hit_rate
        )
    }

    /// Evict oldest entry
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .cache
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(key, _)| key.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }
}

/// Incremental rendering optimizer
pub struct IncrementalRenderer {
    last_frame: Vec<Vec<char>>,
    current_frame: Vec<Vec<char>>,
    width: usize,
    height: usize,
}

impl IncrementalRenderer {
    /// Create a new incremental renderer
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            last_frame: vec![vec![' '; width]; height],
            current_frame: vec![vec![' '; width]; height],
            width,
            height,
        }
    }

    /// Set a character at position
    pub fn set_char(&mut self, x: usize, y: usize, ch: char) {
        if x < self.width && y < self.height {
            self.current_frame[y][x] = ch;
        }
    }

    /// Get the differences between last and current frame
    pub fn get_diff(&self) -> Vec<(usize, usize, char)> {
        let mut diff = Vec::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if self.current_frame[y][x] != self.last_frame[y][x] {
                    diff.push((x, y, self.current_frame[y][x]));
                }
            }
        }

        diff
    }

    /// Commit current frame as last frame
    pub fn commit(&mut self) {
        self.last_frame = self.current_frame.clone();
    }

    /// Clear current frame
    pub fn clear(&mut self) {
        for row in &mut self.current_frame {
            row.fill(' ');
        }
    }

    /// Reset both frames
    pub fn reset(&mut self) {
        self.clear();
        self.last_frame = vec![vec![' '; self.width]; self.height];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirty_region_merge() {
        let r1 = DirtyRegion::new(0, 0, 10, 10);
        let r2 = DirtyRegion::new(5, 5, 10, 10);

        let merged = r1.merge(&r2);
        assert_eq!(merged.x, 0);
        assert_eq!(merged.y, 0);
        assert_eq!(merged.width, 15);
        assert_eq!(merged.height, 15);
    }

    #[test]
    fn test_dirty_region_manager() {
        let mut manager = DirtyRegionManager::new(100, 100);

        manager.add_region(DirtyRegion::new(10, 10, 20, 20));
        manager.add_region(DirtyRegion::new(25, 25, 20, 20));

        // Should merge overlapping regions
        assert_eq!(manager.get_regions().len(), 1);

        assert!(manager.is_dirty(15, 15));
        assert!(manager.is_dirty(30, 30));
        assert!(!manager.is_dirty(0, 0));
    }

    #[test]
    fn test_render_cache() {
        let mut cache = RenderCache::new(10);

        cache.store("component1".to_string(), 12345, 67890, vec![1, 2, 3], 10, 5);

        // Cache hit
        assert!(cache.get("component1", 12345, 67890).is_some());
        assert_eq!(cache.hits, 1);

        // Cache miss
        assert!(cache.get("component2", 12345, 67890).is_none());
        assert_eq!(cache.misses, 1);
    }

    #[test]
    fn test_incremental_renderer() {
        let mut renderer = IncrementalRenderer::new(5, 3);

        renderer.set_char(1, 1, 'X');
        renderer.set_char(3, 2, 'O');

        let diff = renderer.get_diff();
        assert_eq!(diff.len(), 2);
        assert!(diff.contains(&(1, 1, 'X')));
        assert!(diff.contains(&(3, 2, 'O')));

        renderer.commit();

        // No changes after commit
        let diff2 = renderer.get_diff();
        assert_eq!(diff2.len(), 0);
    }
}
