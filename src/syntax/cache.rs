//! Line cache for efficient syntax highlighting

use crate::syntax::highlighter::HighlightedLine;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Cache entry with timestamp
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The highlighted line data
    line: HighlightedLine,
    /// When this entry was created
    timestamp: Instant,
    /// Hash of the original content
    hash: u64,
}

/// Line cache for highlighted code
pub struct LineCache {
    /// Cache entries indexed by line number
    entries: HashMap<usize, CacheEntry>,
    /// Maximum age for cache entries
    max_age: Duration,
    /// Maximum number of entries to keep
    max_entries: usize,
}

impl Default for LineCache {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(60))
    }
}

impl LineCache {
    /// Create a new cache with size and age limits
    pub fn new(max_entries: usize, max_age: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(max_entries),
            max_age,
            max_entries,
        }
    }

    /// Get a cached line
    pub fn get(&self, line_number: usize, content_hash: u64) -> Option<&HighlightedLine> {
        self.entries.get(&line_number).and_then(|entry| {
            let age = Instant::now().duration_since(entry.timestamp);
            if age < self.max_age && entry.hash == content_hash {
                Some(&entry.line)
            } else {
                None
            }
        })
    }

    /// Insert a highlighted line into the cache
    pub fn insert(&mut self, line_number: usize, line: HighlightedLine, content_hash: u64) {
        // Evict old entries if at capacity
        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        self.entries.insert(
            line_number,
            CacheEntry {
                line,
                timestamp: Instant::now(),
                hash: content_hash,
            },
        );
    }

    /// Invalidate a range of lines
    pub fn invalidate_range(&mut self, start: usize, end: usize) {
        self.entries
            .retain(|&line_num, _| line_num < start || line_num >= end);
    }

    /// Clear the entire cache
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Evict the oldest entry
    fn evict_oldest(&mut self) {
        if let Some((&oldest_key, _)) = self.entries.iter().min_by_key(|(_, entry)| entry.timestamp)
        {
            self.entries.remove(&oldest_key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let now = Instant::now();
        let mut total_age = Duration::ZERO;
        let mut expired_count = 0;

        for entry in self.entries.values() {
            let age = now.duration_since(entry.timestamp);
            total_age += age;
            if age >= self.max_age {
                expired_count += 1;
            }
        }

        CacheStats {
            entry_count: self.entries.len(),
            expired_count,
            average_age: if self.entries.is_empty() {
                Duration::ZERO
            } else {
                total_age / self.entries.len() as u32
            },
            capacity: self.max_entries,
        }
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    /// Number of entries currently in cache
    pub entry_count: usize,
    /// Number of expired entries removed
    pub expired_count: usize,
    /// Average age of cache entries
    pub average_age: Duration,
    /// Maximum cache capacity
    pub capacity: usize,
}

/// Calculate a hash for line content
pub fn hash_line_content(content: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::styled_text::StyledRun;
    use crate::core::surface::{Attr, Rgba};

    #[test]
    fn test_cache_basic() {
        let mut cache = LineCache::new(10, Duration::from_secs(60));

        let line = HighlightedLine {
            runs: vec![StyledRun::new(
                "test".to_string(),
                Rgba::white(),
                Rgba::black(),
                Attr::empty(),
            )],
            line_number: 0,
        };

        let hash = hash_line_content("test");
        cache.insert(0, line.clone(), hash);

        // Should retrieve with correct hash
        assert!(cache.get(0, hash).is_some());

        // Should not retrieve with wrong hash
        assert!(cache.get(0, hash + 1).is_none());

        // Should not retrieve wrong line number
        assert!(cache.get(1, hash).is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = LineCache::new(10, Duration::from_secs(60));

        // Insert multiple lines
        for i in 0..5 {
            let line = HighlightedLine {
                runs: vec![],
                line_number: i,
            };
            cache.insert(i, line, i as u64);
        }

        // Invalidate middle range
        cache.invalidate_range(1, 4);

        // Should keep lines outside range
        assert!(cache.get(0, 0).is_some());
        assert!(cache.get(4, 4).is_some());

        // Should remove lines in range
        assert!(cache.get(1, 1).is_none());
        assert!(cache.get(2, 2).is_none());
        assert!(cache.get(3, 3).is_none());
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = LineCache::new(3, Duration::from_secs(60));

        // Fill cache to capacity
        for i in 0..3 {
            let line = HighlightedLine {
                runs: vec![],
                line_number: i,
            };
            cache.insert(i, line, i as u64);
        }

        // Insert one more should evict oldest
        let line = HighlightedLine {
            runs: vec![],
            line_number: 3,
        };
        cache.insert(3, line, 3);

        // Should have exactly max_entries
        assert_eq!(cache.entries.len(), 3);
        assert!(cache.get(3, 3).is_some());
    }

    #[test]
    fn test_cache_stats() {
        let mut cache = LineCache::new(10, Duration::from_secs(60));

        for i in 0..5 {
            let line = HighlightedLine {
                runs: vec![],
                line_number: i,
            };
            cache.insert(i, line, i as u64);
        }

        let stats = cache.stats();
        assert_eq!(stats.entry_count, 5);
        assert_eq!(stats.capacity, 10);
        assert_eq!(stats.expired_count, 0);
    }
}
