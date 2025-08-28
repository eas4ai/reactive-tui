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
        let gray = ((r - 8) as f32 / 247.0 * 24.0) as u8;
        return 232 + gray;
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
    // Calculate brightness
    let brightness = (r as u16 + g as u16 + b as u16) / 3;
    let is_bright = brightness > 127;

    // Find dominant color channel
    let max = cmp::max(r, cmp::max(g, b));
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

/// Convert hex to RGB
pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');

    // Handle 3-char hex (e.g., #FFF -> #FFFFFF)
    let hex = if hex.len() == 3 {
        let chars: Vec<char> = hex.chars().collect();
        format!(
            "{}{}{}{}{}{}",
            chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
        )
    } else {
        hex.to_string()
    };

    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
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

/// Get the best ANSI color for a given RGB value with specified color depth
pub enum ColorDepth {
    Ansi16,    // Basic 16 colors
    Ansi256,   // Extended 256 colors
    TrueColor, // Full RGB support
}

pub fn rgb_to_ansi(r: u8, g: u8, b: u8, depth: ColorDepth) -> AnsiColor {
    match depth {
        ColorDepth::Ansi16 => AnsiColor::Basic(rgb_to_ansi16(r, g, b)),
        ColorDepth::Ansi256 => AnsiColor::Extended(rgb_to_ansi256(r, g, b)),
        ColorDepth::TrueColor => AnsiColor::Rgb(r, g, b),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnsiColor {
    Basic(u8),    // 0-15
    Extended(u8), // 0-255
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
}
