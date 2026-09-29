//! Common parsing functions for CSS utilities

/// Parse pixel values from tokens like "w-4px" or "p-8"
/// Protected against DoS attacks with input length limits
pub fn parse_px(token: &str, prefix: &str) -> Option<f32> {
    // Protect against DoS attacks - limit input length
    const MAX_TOKEN_LENGTH: usize = 64;
    const MAX_PREFIX_LENGTH: usize = 16;

    if token.len() > MAX_TOKEN_LENGTH || prefix.len() > MAX_PREFIX_LENGTH {
        return None;
    }

    token.strip_prefix(prefix).and_then(|n| {
        // Additional length check on the remaining part
        if n.len() > 32 {
            return None;
        }
        let trimmed = n.strip_suffix("px").unwrap_or(n);
        trimmed.parse::<f32>().ok()
    })
}

/// The largest number a spacing class takes, in cells.
pub const MAX_SPACING: f32 = 512.0;

/// The cells the number of a spacing class asks for: the number itself, and
/// for a number with a fraction the next whole number, so that `0.25` is one
/// cell; `px` is one cell. `None` for anything else than a number from 0 to
/// [`MAX_SPACING`] written in digits.
pub fn spacing_cells(number: &str) -> Option<f32> {
    if number == "px" {
        return Some(1.0);
    }
    // Digits and a point only: no sign, no exponent, no `inf` and no `nan`.
    if number.is_empty()
        || number.len() > 16
        || !number
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return None;
    }
    let cells = number.parse::<f32>().ok()?.ceil();
    (cells <= MAX_SPACING).then_some(cells)
}

/// Parse the number of a padding, margin, gap or space class as a count of
/// cells, as the width and height classes count: `p-1` is one cell and
/// `gap-2` two (see [`spacing_cells`]).
/// Protected against DoS attacks with input length limits
pub fn parse_spacing(token: &str, prefix: &str) -> Option<f32> {
    // Protect against DoS attacks - limit input length
    const MAX_TOKEN_LENGTH: usize = 64;
    const MAX_PREFIX_LENGTH: usize = 16;

    if token.len() > MAX_TOKEN_LENGTH || prefix.len() > MAX_PREFIX_LENGTH {
        return None;
    }

    token.strip_prefix(prefix).and_then(spacing_cells)
}

/// Parse spacing values for terminal dimensions (width/height)
/// In terminal context, we want direct character cell mapping
pub fn parse_spacing_terminal(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        // Always treat as character cells for terminal dimensions
        n.parse::<f32>().ok()
    })
}

/// Parse percentage values from tokens like "w-1/2" or "w-full"
pub fn parse_percentage(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        match n {
            "full" => Some(100.0),
            "1/2" => Some(50.0),
            "1/3" => Some(33.333333),
            "2/3" => Some(66.666667),
            "1/4" => Some(25.0),
            "3/4" => Some(75.0),
            "1/5" => Some(20.0),
            "2/5" => Some(40.0),
            "3/5" => Some(60.0),
            "4/5" => Some(80.0),
            "1/6" => Some(16.666667),
            "5/6" => Some(83.333333),
            "1/12" => Some(8.333333),
            "5/12" => Some(41.666667),
            "7/12" => Some(58.333333),
            "11/12" => Some(91.666667),
            _ => {
                // Try to parse as direct percentage like "50"
                n.parse::<f32>().ok()
            }
        }
    })
}

/// Parse grid column/row values
pub fn parse_grid_value(token: &str, prefix: &str) -> Option<u16> {
    token.strip_prefix(prefix).and_then(|n| {
        match n {
            "none" => Some(0),
            "subgrid" => Some(0), // Special handling needed
            _ => n.parse::<u16>().ok(),
        }
    })
}

/// Parse z-index values with common presets
pub fn parse_z_index(token: &str) -> Option<i32> {
    match token {
        "z-auto" => Some(0),
        "z-0" => Some(0),
        "z-10" => Some(10),
        "z-20" => Some(20),
        "z-30" => Some(30),
        "z-40" => Some(40),
        "z-50" => Some(50),
        _ => token.strip_prefix("z-").and_then(|n| n.parse::<i32>().ok()),
    }
}

/// Parse opacity values (0-100)
pub fn parse_opacity(token: &str) -> Option<f32> {
    token.strip_prefix("opacity-").and_then(|n| {
        n.parse::<u8>()
            .ok()
            .map(|opacity| (opacity as f32 / 100.0).clamp(0.0, 1.0))
    })
}

/// Parse fraction values like "1/2", "3/4", etc.
pub fn parse_fraction(token: &str, prefix: &str) -> Option<f32> {
    token.strip_prefix(prefix).and_then(|n| {
        if let Some((num_str, den_str)) = n.split_once('/') {
            let num = num_str.parse::<f32>().ok()?;
            let den = den_str.parse::<f32>().ok()?;
            if den != 0.0 {
                Some(num / den)
            } else {
                None
            }
        } else {
            None
        }
    })
}

/// Parse font weight values
pub fn parse_font_weight(token: &str) -> Option<u16> {
    match token {
        "font-thin" => Some(100),
        "font-extralight" => Some(200),
        "font-light" => Some(300),
        "font-normal" => Some(400),
        "font-medium" => Some(500),
        "font-semibold" => Some(600),
        "font-bold" => Some(700),
        "font-extrabold" => Some(800),
        "font-black" => Some(900),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_spacing() {
        // The number is a count of cells
        assert_eq!(parse_spacing("p-0", "p-"), Some(0.0));
        assert_eq!(parse_spacing("p-16", "p-"), Some(16.0));
        assert_eq!(parse_spacing("m-32", "m-"), Some(32.0));
        assert_eq!(parse_spacing("gap-48", "gap-"), Some(48.0));
        assert_eq!(parse_spacing("gap-13", "gap-"), Some(13.0));
        assert_eq!(parse_spacing("p-512", "p-"), Some(512.0));
        assert_eq!(parse_spacing("p-px", "p-"), Some(1.0));
        assert_eq!(parse_spacing("p-invalid", "p-"), None);
    }

    #[test]
    fn a_fraction_is_the_next_whole_cell() {
        assert_eq!(parse_spacing("gap-0.25", "gap-"), Some(1.0));
        assert_eq!(parse_spacing("p-1.5", "p-"), Some(2.0));
        assert_eq!(parse_spacing("m-2.0", "m-"), Some(2.0));
    }

    #[test]
    fn a_spacing_number_is_digits_from_0_to_512() {
        for number in [
            "513",
            "512.5",
            "-1",
            "+1",
            "1e2",
            "inf",
            "nan",
            "NaN",
            "",
            ".",
            "1.2.3",
            "0x10",
            "99999999999999999",
        ] {
            assert_eq!(spacing_cells(number), None, "{number:?}");
        }
    }

    #[test]
    fn test_parse_spacing_terminal() {
        // Terminal spacing for width/height (direct character cells)
        assert_eq!(parse_spacing_terminal("w-0", "w-"), Some(0.0));
        assert_eq!(parse_spacing_terminal("w-5", "w-"), Some(5.0));
        assert_eq!(parse_spacing_terminal("h-10", "h-"), Some(10.0));
        assert_eq!(parse_spacing_terminal("w-20", "w-"), Some(20.0));
        assert_eq!(parse_spacing_terminal("w-invalid", "w-"), None);
    }

    #[test]
    fn test_parse_percentage() {
        assert_eq!(parse_percentage("w-full", "w-"), Some(100.0));
        assert_eq!(parse_percentage("w-1/2", "w-"), Some(50.0));
        assert_eq!(parse_percentage("w-1/3", "w-"), Some(33.333333));
        assert_eq!(parse_percentage("w-3/4", "w-"), Some(75.0));
    }

    #[test]
    fn test_parse_z_index() {
        assert_eq!(parse_z_index("z-auto"), Some(0));
        assert_eq!(parse_z_index("z-10"), Some(10));
        assert_eq!(parse_z_index("z-50"), Some(50));
        assert_eq!(parse_z_index("z-999"), Some(999));
        assert_eq!(parse_z_index("z--10"), Some(-10));
    }

    #[test]
    fn test_parse_opacity() {
        assert_eq!(parse_opacity("opacity-0"), Some(0.0));
        assert_eq!(parse_opacity("opacity-50"), Some(0.5));
        assert_eq!(parse_opacity("opacity-100"), Some(1.0));
        assert_eq!(parse_opacity("opacity-150"), Some(1.0)); // Clamped
    }

    #[test]
    fn test_parse_fraction() {
        assert_eq!(parse_fraction("w-1/2", "w-"), Some(0.5));
        assert_eq!(parse_fraction("w-3/4", "w-"), Some(0.75));
        assert_eq!(parse_fraction("w-2/3", "w-"), Some(0.6666667));
        assert_eq!(parse_fraction("w-invalid", "w-"), None);
        assert_eq!(parse_fraction("w-1/0", "w-"), None); // Division by zero
    }

    #[test]
    fn test_parse_font_weight() {
        assert_eq!(parse_font_weight("font-normal"), Some(400));
        assert_eq!(parse_font_weight("font-bold"), Some(700));
        assert_eq!(parse_font_weight("font-black"), Some(900));
        assert_eq!(parse_font_weight("font-invalid"), None);
    }
}
