/// ANSI color conversion utilities
///
/// Provides conversion from RGB/Hex colors to ANSI 256-color palette
/// and basic 16-color palette for terminal compatibility
use std::cmp;

/// Convert RGB to ANSI 256 color
pub fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    // Check for grayscale
    if r == g && g == b {
        // Use grayscale ramp (colors 232-255)
        if r < 8 {
            return 16; // Black
        }
        if r > 248 {
            return 231; // White
        }

        // Map to 24 grayscale colors (232-255)
        // Grayscale values: 8, 18, 28, ..., 238 (increments of 10)
        // So for a given gray value, find nearest: round((value - 8) / 10)
        let gray = ((r.saturating_sub(8)) as f32 / 10.0).round() as u8;
        return 232 + gray.min(23); // Ensure we don't exceed 255
    }

    // Use 6x6x6 color cube (colors 16-231)
    let r_idx = if r < 48 {
        0
    } else {
        cmp::min(5, (r - 35) / 40)
    };
    let g_idx = if g < 48 {
        0
    } else {
        cmp::min(5, (g - 35) / 40)
    };
    let b_idx = if b < 48 {
        0
    } else {
        cmp::min(5, (b - 35) / 40)
    };

    16 + 36 * r_idx + 6 * g_idx + b_idx
}

/// Convert hex string to ANSI 256 color
pub fn hex_to_ansi256(hex: &str) -> Option<u8> {
    let (r, g, b) = hex_to_rgb(hex)?;
    Some(rgb_to_ansi256(r, g, b))
}

/// Convert RGB to basic ANSI 16 color
pub fn rgb_to_ansi16(r: u8, g: u8, b: u8) -> u8 {
    // Find dominant color channel and brightness
    let max = cmp::max(r, cmp::max(g, b));
    let is_bright = max > 192; // Only consider truly bright colors
    let threshold = 64;

    // Black/Gray/White detection
    if max < threshold {
        return if is_bright { 8 } else { 0 }; // Dark gray or black
    }
    if r > 192 && g > 192 && b > 192 {
        return 15; // White
    }
    if r > 96 && g > 96 && b > 96 {
        return 7; // Light gray
    }

    // Color detection with brightness
    let base = if r == max && g < threshold && b < threshold {
        1 // Red
    } else if g == max && r < threshold && b < threshold {
        2 // Green
    } else if b == max && r < threshold && g < threshold {
        4 // Blue
    } else if r == max && g == max && b < threshold {
        3 // Yellow
    } else if r == max && b == max && g < threshold {
        5 // Magenta
    } else if g == max && b == max && r < threshold {
        6 // Cyan
    } else if r > 128 && g > 64 && b < 64 {
        3 // Yellow-ish
    } else if r > 128 && b > 64 && g < 64 {
        5 // Magenta-ish
    } else if g > 128 && b > 64 && r < 64 {
        6 // Cyan-ish
    } else {
        7 // Default to gray
    };

    if is_bright && base < 8 {
        base + 8 // Bright version
    } else {
        base
    }
}

/// Convert hex to RGB. Accepts `#rgb` and `#rrggbb`; anything else
/// (including `#rgba`/`#rrggbbaa`) is `None`. Delegates to the canonical
/// parser in `crate::layout::colors`.
pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    // Historical contract (see test_hex_to_rgb): a missing '#' is tolerated.
    let owned;
    let hex = if hex.starts_with('#') {
        hex
    } else {
        owned = format!("#{hex}");
        &owned
    };
    let digits = hex.trim_start_matches('#');
    match digits.len() {
        3 | 6 => {
            let (r, g, b, _) = crate::layout::colors::parse_hex_bytes(hex)?;
            Some((r, g, b))
        }
        _ => None,
    }
}

/// ANSI 256 color to RGB approximation
pub fn ansi256_to_rgb(color: u8) -> (u8, u8, u8) {
    match color {
        // Standard 16 colors
        0 => (0, 0, 0),        // Black
        1 => (128, 0, 0),      // Red
        2 => (0, 128, 0),      // Green
        3 => (128, 128, 0),    // Yellow
        4 => (0, 0, 128),      // Blue
        5 => (128, 0, 128),    // Magenta
        6 => (0, 128, 128),    // Cyan
        7 => (192, 192, 192),  // White
        8 => (128, 128, 128),  // Bright Black
        9 => (255, 0, 0),      // Bright Red
        10 => (0, 255, 0),     // Bright Green
        11 => (255, 255, 0),   // Bright Yellow
        12 => (0, 0, 255),     // Bright Blue
        13 => (255, 0, 255),   // Bright Magenta
        14 => (0, 255, 255),   // Bright Cyan
        15 => (255, 255, 255), // Bright White

        // 216 color cube (16-231)
        16..=231 => {
            let idx = color - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            (r, g, b)
        }

        // Grayscale (232-255)
        232..=255 => {
            let gray = 8 + (color - 232) * 10;
            (gray, gray, gray)
        }
    }
}

/// Color depth support levels for ANSI terminals
pub enum ColorDepth {
    /// Basic 16 colors (0-15)
    Ansi16,
    /// Extended 256 colors (0-255)
    Ansi256,
    /// Full RGB support (24-bit color)
    TrueColor,
}

/// Convert RGB values to ANSI color based on color depth
///
/// # Arguments
/// * `r` - Red component (0-255)
/// * `g` - Green component (0-255)
/// * `b` - Blue component (0-255)
/// * `depth` - Target color depth
///
/// # Returns
/// Appropriate ANSI color for the given depth
pub fn rgb_to_ansi(r: u8, g: u8, b: u8, depth: ColorDepth) -> AnsiColor {
    match depth {
        ColorDepth::Ansi16 => AnsiColor::Basic(rgb_to_ansi16(r, g, b)),
        ColorDepth::Ansi256 => AnsiColor::Extended(rgb_to_ansi256(r, g, b)),
        ColorDepth::TrueColor => AnsiColor::Rgb(r, g, b),
    }
}

/// ANSI color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiColor {
    /// Basic ANSI color (0-15)
    Basic(u8),
    /// Extended ANSI color (0-255)
    Extended(u8),
    /// True color RGB values
    Rgb(u8, u8, u8),
}

impl AnsiColor {
    /// Convert to ANSI escape sequence
    pub fn to_fg_escape(&self) -> String {
        match self {
            AnsiColor::Basic(n) if *n < 8 => format!("\x1b[3{n}m"),
            AnsiColor::Basic(n) => format!("\x1b[9{}m", n - 8),
            AnsiColor::Extended(n) => format!("\x1b[38;5;{n}m"),
            AnsiColor::Rgb(r, g, b) => format!("\x1b[38;2;{r};{g};{b}m"),
        }
    }

    /// Convert to background color escape sequence
    ///
    /// # Returns
    /// ANSI escape sequence for setting background color
    pub fn to_bg_escape(&self) -> String {
        match self {
            AnsiColor::Basic(n) if *n < 8 => format!("\x1b[4{n}m"),
            AnsiColor::Basic(n) => format!("\x1b[10{}m", n - 8),
            AnsiColor::Extended(n) => format!("\x1b[48;5;{n}m"),
            AnsiColor::Rgb(r, g, b) => format!("\x1b[48;2;{r};{g};{b}m"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_ansi256() {
        // Pure colors
        assert_eq!(rgb_to_ansi256(255, 0, 0), 196); // Red
        assert_eq!(rgb_to_ansi256(0, 255, 0), 46); // Green
        assert_eq!(rgb_to_ansi256(0, 0, 255), 21); // Blue

        // Grayscale
        assert_eq!(rgb_to_ansi256(0, 0, 0), 16); // Black
        assert_eq!(rgb_to_ansi256(255, 255, 255), 231); // White
        assert_eq!(rgb_to_ansi256(128, 128, 128), 244); // Mid gray
    }

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#FF0000"), Some((255, 0, 0)));
        assert_eq!(hex_to_rgb("#00FF00"), Some((0, 255, 0)));
        assert_eq!(hex_to_rgb("#0000FF"), Some((0, 0, 255)));
        assert_eq!(hex_to_rgb("FF0"), Some((255, 255, 0)));
        assert_eq!(hex_to_rgb("#FFF"), Some((255, 255, 255)));
    }

    #[test]
    fn test_rgb_to_ansi16() {
        assert_eq!(rgb_to_ansi16(255, 0, 0), 9); // Bright red
        assert_eq!(rgb_to_ansi16(0, 128, 0), 2); // Green
        assert_eq!(rgb_to_ansi16(0, 0, 0), 0); // Black
        assert_eq!(rgb_to_ansi16(255, 255, 255), 15); // White
    }

    #[test]
    fn test_hex_to_rgb_rejects_without_panicking() {
        // Widths outside #rgb/#rrggbb stay None, even though the
        // canonical parser understands #rgba/#rrggbbaa.
        for bad in [
            "",
            "#",
            "#ff",
            "#ffff",
            "#ff0000ff",
            "#gg0000",
            "#ff0000 ",
            "red",
            "#é",
            // 3 bytes but 2 chars: indexed chars[2] and panicked before.
            "#éx",
        ] {
            assert_eq!(hex_to_rgb(bad), None, "input: {bad:?}");
        }
    }
}
