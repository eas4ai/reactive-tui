//! Color parsing cache for CSS utilities
//!
//! Pre-computed color values and LRU caching for dynamic colors

use lru::LruCache;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::num::NonZeroUsize;

/// RGBA color tuple (r, g, b, a) with u8 components
type ColorTuple = (u8, u8, u8, f32);

/// Pre-computed utility color palette
static TAILWIND_COLORS: Lazy<HashMap<&'static str, ColorTuple>> = Lazy::new(|| {
    let mut m = HashMap::with_capacity(300);

    // Basic colors
    m.insert("black", (0, 0, 0, 1.0));
    m.insert("white", (255, 255, 255, 1.0));
    m.insert("transparent", (0, 0, 0, 0.0));

    // Slate palette
    m.insert("slate-50", (248, 250, 252, 1.0));
    m.insert("slate-100", (241, 245, 249, 1.0));
    m.insert("slate-200", (226, 232, 240, 1.0));
    m.insert("slate-300", (203, 213, 225, 1.0));
    m.insert("slate-400", (148, 163, 184, 1.0));
    m.insert("slate-500", (100, 116, 139, 1.0));
    m.insert("slate-600", (71, 85, 105, 1.0));
    m.insert("slate-700", (51, 65, 85, 1.0));
    m.insert("slate-800", (30, 41, 59, 1.0));
    m.insert("slate-900", (15, 23, 42, 1.0));
    m.insert("slate-950", (2, 6, 23, 1.0));

    // Gray palette
    m.insert("gray-50", (249, 250, 251, 1.0));
    m.insert("gray-100", (243, 244, 246, 1.0));
    m.insert("gray-200", (229, 231, 235, 1.0));
    m.insert("gray-300", (209, 213, 219, 1.0));
    m.insert("gray-400", (156, 163, 175, 1.0));
    m.insert("gray-500", (107, 114, 128, 1.0));
    m.insert("gray-600", (75, 85, 99, 1.0));
    m.insert("gray-700", (55, 65, 81, 1.0));
    m.insert("gray-800", (31, 41, 55, 1.0));
    m.insert("gray-900", (17, 24, 39, 1.0));
    m.insert("gray-950", (3, 7, 18, 1.0));

    // Zinc palette
    m.insert("zinc-50", (250, 250, 250, 1.0));
    m.insert("zinc-100", (244, 244, 245, 1.0));
    m.insert("zinc-200", (228, 228, 231, 1.0));
    m.insert("zinc-300", (212, 212, 216, 1.0));
    m.insert("zinc-400", (161, 161, 170, 1.0));
    m.insert("zinc-500", (113, 113, 122, 1.0));
    m.insert("zinc-600", (82, 82, 91, 1.0));
    m.insert("zinc-700", (63, 63, 70, 1.0));
    m.insert("zinc-800", (39, 39, 42, 1.0));
    m.insert("zinc-900", (24, 24, 27, 1.0));
    m.insert("zinc-950", (9, 9, 11, 1.0));

    // Red palette
    m.insert("red-50", (254, 242, 242, 1.0));
    m.insert("red-100", (254, 226, 226, 1.0));
    m.insert("red-200", (254, 202, 202, 1.0));
    m.insert("red-300", (252, 165, 165, 1.0));
    m.insert("red-400", (248, 113, 113, 1.0));
    m.insert("red-500", (239, 68, 68, 1.0));
    m.insert("red-600", (220, 38, 38, 1.0));
    m.insert("red-700", (185, 28, 28, 1.0));
    m.insert("red-800", (153, 27, 27, 1.0));
    m.insert("red-900", (127, 29, 29, 1.0));
    m.insert("red-950", (69, 10, 10, 1.0));

    // Blue palette
    m.insert("blue-50", (239, 246, 255, 1.0));
    m.insert("blue-100", (219, 234, 254, 1.0));
    m.insert("blue-200", (191, 219, 254, 1.0));
    m.insert("blue-300", (147, 197, 253, 1.0));
    m.insert("blue-400", (96, 165, 250, 1.0));
    m.insert("blue-500", (59, 130, 246, 1.0));
    m.insert("blue-600", (37, 99, 235, 1.0));
    m.insert("blue-700", (29, 78, 216, 1.0));
    m.insert("blue-800", (30, 64, 175, 1.0));
    m.insert("blue-900", (30, 58, 138, 1.0));
    m.insert("blue-950", (23, 37, 84, 1.0));

    // Green palette
    m.insert("green-50", (240, 253, 244, 1.0));
    m.insert("green-100", (220, 252, 231, 1.0));
    m.insert("green-200", (187, 247, 208, 1.0));
    m.insert("green-300", (134, 239, 172, 1.0));
    m.insert("green-400", (74, 222, 128, 1.0));
    m.insert("green-500", (34, 197, 94, 1.0));
    m.insert("green-600", (22, 163, 74, 1.0));
    m.insert("green-700", (21, 128, 61, 1.0));
    m.insert("green-800", (22, 101, 52, 1.0));
    m.insert("green-900", (20, 83, 45, 1.0));
    m.insert("green-950", (5, 46, 22, 1.0));

    // Yellow palette
    m.insert("yellow-50", (254, 252, 232, 1.0));
    m.insert("yellow-100", (254, 249, 195, 1.0));
    m.insert("yellow-200", (254, 240, 138, 1.0));
    m.insert("yellow-300", (253, 224, 71, 1.0));
    m.insert("yellow-400", (250, 204, 21, 1.0));
    m.insert("yellow-500", (234, 179, 8, 1.0));
    m.insert("yellow-600", (202, 138, 4, 1.0));
    m.insert("yellow-700", (161, 98, 7, 1.0));
    m.insert("yellow-800", (133, 77, 14, 1.0));
    m.insert("yellow-900", (113, 63, 18, 1.0));
    m.insert("yellow-950", (66, 32, 6, 1.0));

    // Purple palette
    m.insert("purple-50", (250, 245, 255, 1.0));
    m.insert("purple-100", (243, 232, 255, 1.0));
    m.insert("purple-200", (233, 213, 255, 1.0));
    m.insert("purple-300", (216, 180, 254, 1.0));
    m.insert("purple-400", (192, 132, 252, 1.0));
    m.insert("purple-500", (168, 85, 247, 1.0));
    m.insert("purple-600", (147, 51, 234, 1.0));
    m.insert("purple-700", (126, 34, 206, 1.0));
    m.insert("purple-800", (107, 33, 168, 1.0));
    m.insert("purple-900", (88, 28, 135, 1.0));
    m.insert("purple-950", (59, 7, 100, 1.0));

    m
});

thread_local! {
    /// Thread-local LRU cache for dynamically parsed colors (hex codes, rgb(), etc.)
    static DYNAMIC_COLOR_CACHE: RefCell<LruCache<String, ColorTuple>> =
        RefCell::new(LruCache::new(
            NonZeroUsize::new(128).expect("Cache size must be non-zero")
        ));
}

/// Get a color from the static utility palette
#[inline(always)]
pub fn get_utility_color(name: &str) -> Option<ColorTuple> {
    TAILWIND_COLORS.get(name).copied()
}

/// Get a cached dynamic color
pub fn get_cached_color(token: &str) -> Option<ColorTuple> {
    // First check static colors
    if let Some(color) = get_utility_color(token) {
        return Some(color);
    }

    // Then check dynamic cache
    DYNAMIC_COLOR_CACHE.with(|cache| cache.borrow_mut().get(token).copied())
}

/// Cache a dynamically parsed color
pub fn cache_color(token: String, color: ColorTuple) {
    DYNAMIC_COLOR_CACHE.with(|cache| {
        cache.borrow_mut().put(token, color);
    });
}

/// Parse hex color with caching
pub fn parse_hex_cached(hex: &str) -> Option<ColorTuple> {
    // Check cache first
    if let Some(color) = get_cached_color(hex) {
        return Some(color);
    }

    // Parse hex color through the canonical parser; the cache only stores.
    let (r, g, b, a) = crate::layout::colors::parse_hex_bytes(hex)?;
    let (r, g, b, a) = (r, g, b, a as f32 / 255.0);

    let color = (r, g, b, a);
    cache_color(hex.to_string(), color);
    Some(color)
}

/// Parse rgb/rgba color with caching
pub fn parse_rgb_cached(rgb: &str) -> Option<ColorTuple> {
    // Check cache first
    if let Some(color) = get_cached_color(rgb) {
        return Some(color);
    }

    // Parse rgb(r, g, b) or rgba(r, g, b, a)
    let inner = if let Some(inner) = rgb.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        inner
    } else {
        rgb.strip_prefix("rgba(")
            .and_then(|s| s.strip_suffix(')'))?
    };

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
    let color = match parts.len() {
        3 => {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            (r, g, b, 1.0)
        }
        4 => {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            let a = parts[3].parse::<f32>().ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };

    cache_color(rgb.to_string(), color);
    Some(color)
}

/// Clear the dynamic color cache
pub fn clear_color_cache() {
    DYNAMIC_COLOR_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Get cache statistics
#[derive(Debug, Default, Clone)]
pub struct ColorCacheStats {
    /// Current number of cached colors
    pub size: usize,
    /// Maximum capacity of the cache
    pub capacity: usize,
}

/// Get current cache statistics
pub fn get_cache_stats() -> ColorCacheStats {
    DYNAMIC_COLOR_CACHE.with(|cache| {
        let cache = cache.borrow();
        ColorCacheStats {
            size: cache.len(),
            capacity: cache.cap().get(),
        }
    })
}
