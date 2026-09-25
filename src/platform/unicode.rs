//! Unicode handling and grapheme clustering
//!
//! Advanced Unicode support with grapheme clustering and width calculation

use std::collections::HashMap;

/// Width calculation methods
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidthMethod {
    /// Unicode standard width calculation
    Unicode,
    /// wcwidth() system function
    Wcwidth,
    /// No width calculation (assume 1)
    NoOp,
}

/// Grapheme cluster with display width
#[derive(Debug, Clone, PartialEq)]
pub struct Grapheme {
    /// The grapheme cluster string
    pub cluster: String,
    /// Display width in terminal cells
    pub width: usize,
    /// Byte length of the cluster
    pub len: usize,
}

impl Grapheme {
    /// Create a new grapheme cluster
    pub fn new(cluster: String, width: usize) -> Self {
        let len = cluster.len();
        Self {
            cluster,
            width,
            len,
        }
    }

    /// Create from a single character
    pub fn from_char(ch: char) -> Self {
        let cluster = ch.to_string();
        let width = char_width(ch);
        let len = cluster.len();
        Self {
            cluster,
            width,
            len,
        }
    }
}

/// Grapheme cache for performance optimization
pub struct GraphemeCache {
    /// Circular buffer of cached graphemes
    cache: Vec<Option<Grapheme>>,
    /// Current position in the circular buffer
    head: usize,
    /// Cache capacity
    capacity: usize,
    /// Cache hit statistics
    hits: u64,
    /// Cache miss statistics
    misses: u64,
}

impl GraphemeCache {
    /// Create new grapheme cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        let mut cache = Vec::with_capacity(capacity);
        cache.resize_with(capacity, || None);

        Self {
            cache,
            head: 0,
            capacity,
            hits: 0,
            misses: 0,
        }
    }

    /// Look up grapheme in cache
    pub fn get(&mut self, text: &str) -> Option<&Grapheme> {
        for grapheme in self.cache.iter().flatten() {
            if grapheme.cluster == text {
                self.hits += 1;
                return Some(grapheme);
            }
        }
        self.misses += 1;
        None
    }

    /// Insert grapheme into cache
    pub fn insert(&mut self, grapheme: Grapheme) {
        self.cache[self.head] = Some(grapheme);
        self.head = (self.head + 1) % self.capacity;
    }

    /// Get cache statistics
    pub fn stats(&self) -> (u64, u64, f64) {
        let total = self.hits + self.misses;
        let hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
        (self.hits, self.misses, hit_rate)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        for item in &mut self.cache {
            *item = None;
        }
        self.head = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

/// Unicode handler with caching and multiple width methods
pub struct UnicodeHandler {
    /// Grapheme cache for performance
    cache: GraphemeCache,
    /// Width calculation method
    width_method: WidthMethod,
    /// Custom width overrides
    width_overrides: HashMap<char, usize>,
}

impl UnicodeHandler {
    /// Create new Unicode handler
    pub fn new(width_method: WidthMethod, cache_size: usize) -> Self {
        Self {
            cache: GraphemeCache::new(cache_size),
            width_method,
            width_overrides: HashMap::new(),
        }
    }

    /// Add custom width override for a character
    pub fn set_width_override(&mut self, ch: char, width: usize) {
        self.width_overrides.insert(ch, width);
    }

    /// Get grapheme cluster at position in string
    pub fn grapheme_at(&mut self, text: &str, pos: usize) -> Option<Grapheme> {
        if pos >= text.len() {
            return None;
        }

        // Find the start of the grapheme cluster
        let start = find_grapheme_start(text, pos);
        let cluster_text = &text[start..];

        // Check cache first
        if let Some(cached) = self.cache.get(cluster_text) {
            return Some(cached.clone());
        }

        // Extract the grapheme cluster
        let cluster = extract_grapheme_cluster(cluster_text);
        let width = self.calculate_width(&cluster);

        let grapheme = Grapheme::new(cluster, width);
        self.cache.insert(grapheme.clone());

        Some(grapheme)
    }

    /// Calculate display width of a string
    pub fn string_width(&mut self, text: &str) -> usize {
        let mut width = 0;
        let mut pos = 0;

        while pos < text.len() {
            if let Some(grapheme) = self.grapheme_at(text, pos) {
                width += grapheme.width;
                pos += grapheme.len;
            } else {
                break;
            }
        }

        width
    }

    /// Calculate width of a grapheme cluster
    fn calculate_width(&self, cluster: &str) -> usize {
        if cluster.is_empty() {
            return 0;
        }

        // Get the first character for width calculation
        let first_char = cluster.chars().next().unwrap();

        // Check for custom overrides first
        if let Some(&width) = self.width_overrides.get(&first_char) {
            return width;
        }

        match self.width_method {
            WidthMethod::Unicode => char_width(first_char),
            WidthMethod::Wcwidth => wcwidth_char(first_char),
            WidthMethod::NoOp => 1,
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (u64, u64, f64) {
        self.cache.stats()
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for UnicodeHandler {
    fn default() -> Self {
        Self::new(WidthMethod::Unicode, 256)
    }
}

/// Calculate character width using Unicode standard
pub fn char_width(ch: char) -> usize {
    let code = ch as u32;

    // Control characters
    if code < 32 || (0x7F..0xA0).contains(&code) {
        return 0;
    }

    // ASCII printable characters
    if code < 0x7F {
        return 1;
    }

    // Wide characters (simplified East Asian width detection)
    if is_wide_char(ch) {
        return 2;
    }

    // Zero-width characters
    if is_zero_width_char(ch) {
        return 0;
    }

    // Default to 1 for most characters
    1
}

/// Check if character is wide (East Asian)
fn is_wide_char(ch: char) -> bool {
    let code = ch as u32;

    // CJK ranges (simplified)
    matches!(code,
        0x1100..=0x115F |   // Hangul Jamo
        0x2329..=0x232A |   // Left/Right-Pointing Angle Brackets
        0x2E80..=0x2EFF |   // CJK Radicals Supplement
        0x2F00..=0x2FDF |   // Kangxi Radicals
        0x2FF0..=0x2FFF |   // Ideographic Description Characters
        0x3000..=0x303E |   // CJK Symbols and Punctuation
        0x3041..=0x3096 |   // Hiragana
        0x3099..=0x30FF |   // Katakana
        0x3105..=0x312F |   // Bopomofo
        0x3131..=0x318E |   // Hangul Compatibility Jamo
        0x3190..=0x31E3 |   // CJK Strokes
        0x31F0..=0x321E |   // Katakana Phonetic Extensions
        0x3220..=0x3247 |   // Enclosed CJK Letters and Months
        0x3250..=0x4DBF |   // CJK Extension A
        0x4E00..=0x9FFF |   // CJK Unified Ideographs
        0xA960..=0xA97F |   // Hangul Jamo Extended-A
        0xAC00..=0xD7A3 |   // Hangul Syllables
        0xD7B0..=0xD7FF |   // Hangul Jamo Extended-B
        0xF900..=0xFAFF |   // CJK Compatibility Ideographs
        0xFE10..=0xFE19 |   // Vertical Forms
        0xFE30..=0xFE6F |   // CJK Compatibility Forms
        0xFF00..=0xFF60 |   // Fullwidth Forms
        0xFFE0..=0xFFE6 |   // Fullwidth Forms
        0x20000..=0x2FFFD | // CJK Extension B, C, D, E
        0x30000..=0x3FFFD   // CJK Extension F
    )
}

/// Check if character is zero-width
fn is_zero_width_char(ch: char) -> bool {
    let code = ch as u32;

    // Combining marks and other zero-width characters
    matches!(code,
        0x0300..=0x036F |   // Combining Diacritical Marks
        0x0483..=0x0489 |   // Combining Cyrillic
        0x0591..=0x05BD |   // Hebrew combining marks
        0x05BF..=0x05BF |   // Hebrew point
        0x05C1..=0x05C2 |   // Hebrew points
        0x05C4..=0x05C5 |   // Hebrew marks
        0x05C7..=0x05C7 |   // Hebrew point
        0x0610..=0x061A |   // Arabic combining marks
        0x064B..=0x065F |   // Arabic combining marks
        0x0670..=0x0670 |   // Arabic letter superscript alef
        0x06D6..=0x06DC |   // Arabic small high marks
        0x06DF..=0x06E4 |   // Arabic small high marks
        0x06E7..=0x06E8 |   // Arabic small high marks
        0x06EA..=0x06ED |   // Arabic small high marks
        0x0711..=0x0711 |   // Syriac letter superscript alaph
        0x0730..=0x074A |   // Syriac pointing
        0x07A6..=0x07B0 |   // Thaana combining marks
        0x07EB..=0x07F3 |   // NKo combining marks
        0x0816..=0x0819 |   // Samaritan combining marks
        0x081B..=0x0823 |   // Samaritan combining marks
        0x0825..=0x0827 |   // Samaritan combining marks
        0x0829..=0x082D |   // Samaritan combining marks
        0x0859..=0x085B |   // Mandaic combining marks
        0x200B..=0x200F |   // Zero width space, etc.
        0x202A..=0x202E |   // Directional formatting
        0x2060..=0x2064 |   // Word joiner, etc.
        0x2066..=0x206F |   // Directional formatting
        0xFEFF..=0xFEFF     // Zero width no-break space
    )
}

/// Calculate character width using wcwidth (system function)
fn wcwidth_char(ch: char) -> usize {
    // This would call the system wcwidth function
    // For now, fall back to Unicode method
    char_width(ch)
}

/// Find the start of a grapheme cluster containing the given position
fn find_grapheme_start(text: &str, pos: usize) -> usize {
    // Simplified implementation - in a full version, this would use
    // proper grapheme cluster boundary detection
    let mut start = pos;
    while start > 0 && !text.is_char_boundary(start) {
        start -= 1;
    }
    start
}

/// Extract a single grapheme cluster from the start of the text
fn extract_grapheme_cluster(text: &str) -> String {
    // Simplified implementation - in a full version, this would use
    // proper grapheme cluster segmentation rules
    if let Some(ch) = text.chars().next() {
        ch.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_char_width() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('中'), 2);
        assert_eq!(char_width('\u{0300}'), 0); // Combining grave accent
    }

    #[test]
    fn test_grapheme_cache() {
        let mut cache = GraphemeCache::new(4);
        let grapheme = Grapheme::new("a".to_string(), 1);

        cache.insert(grapheme);
        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_none());

        let (hits, misses, _) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }

    #[test]
    fn test_unicode_handler() {
        let mut handler = UnicodeHandler::new(WidthMethod::Unicode, 16);

        assert_eq!(handler.string_width("hello"), 5);
        assert_eq!(handler.string_width("中文"), 4);

        // Test custom override
        handler.set_width_override('x', 3);
        assert_eq!(handler.string_width("x"), 3);
    }
}
