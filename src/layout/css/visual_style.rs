/// Visual style extraction from CSS classes for terminal rendering
use crate::layout::manager::{BoxStyle, TextStyle};

/// Extract visual styles (colors, text decorations) from CSS classes
pub fn extract_visual_style(class: &str) -> TextStyle {
    let mut style = TextStyle::default();

    for token in class.split_whitespace() {
        // Text color utilities
        if let Some(color) = parse_text_color(token) {
            style.fg = Some(color);
        }

        // Background color utilities
        if let Some(color) = parse_bg_color(token) {
            style.bg = Some(color);
        }

        // Text decoration utilities
        match token {
            "font-bold" | "bold" => style.bold = true,
            "italic" => style.italic = true,
            "underline" => style.underline = true,
            _ => {}
        }
    }

    style
}

/// Extract box styles (backgrounds, borders) from CSS classes
pub fn extract_box_style(class: &str) -> BoxStyle {
    let mut style = BoxStyle::default();

    for token in class.split_whitespace() {
        // Background color utilities
        if let Some(color) = parse_bg_color(token) {
            style.bg = Some(color);
        }

        // Border utilities
        if token.starts_with("border") {
            // Parse border-color-* utilities
            if let Some(color_part) = token.strip_prefix("border-") {
                if let Some(color) = parse_border_color(color_part) {
                    style.border_color = Some(color);
                } else {
                    // Default border color if no specific color is specified
                    style.border_color = Some((128, 128, 128)); // Default gray border
                }
            } else {
                style.border_color = Some((128, 128, 128)); // Default gray border
            }

            // Check for border thickness/style
            if token.contains("border-2") || token.contains("border-4") {
                style.double_border = false; // Single border for now
            }
        }
    }

    style
}

/// Parse text color from token like "text-red-500" or "text-white"
fn parse_text_color(token: &str) -> Option<(u8, u8, u8)> {
    if !token.starts_with("text-") {
        return None;
    }

    let color_part = &token[5..]; // Skip "text-"
    parse_color_value(color_part)
}

/// Parse border color from token like "red-500" or "black" (after "border-" prefix)
fn parse_border_color(color_part: &str) -> Option<(u8, u8, u8)> {
    // Handle numeric border utilities (border-2, border-4, etc.)
    if color_part.chars().all(|c| c.is_ascii_digit()) {
        return None; // This is a border width, not a color
    }

    // Handle border style utilities (solid, dashed, etc.)
    if matches!(
        color_part,
        "solid" | "dashed" | "dotted" | "double" | "none"
    ) {
        return None; // This is a border style, not a color
    }

    // Parse color names and shades
    parse_color_value(color_part)
}

/// Parse background color from token like "bg-blue-500" or "bg-black"
fn parse_bg_color(token: &str) -> Option<(u8, u8, u8)> {
    if !token.starts_with("bg-") {
        return None;
    }

    let color_part = &token[3..]; // Skip "bg-"
    parse_color_value(color_part)
}

/// Parse a color value like "red-500", "white", "black", etc.
fn parse_color_value(color: &str) -> Option<(u8, u8, u8)> {
    // Basic named colors
    match color {
        "black" => return Some((0, 0, 0)),
        "white" => return Some((255, 255, 255)),
        "red" => return Some((239, 68, 68)),   // red-500 default
        "green" => return Some((34, 197, 94)), // green-500 default
        "blue" => return Some((59, 130, 246)), // blue-500 default
        "yellow" => return Some((234, 179, 8)), // yellow-500 default
        "gray" => return Some((107, 114, 128)), // gray-500 default
        _ => {}
    }

    // Shaded colors like "red-500", "blue-700", etc.
    if let Some(dash_pos) = color.find('-') {
        let (base, shade) = color.split_at(dash_pos);
        let shade = &shade[1..]; // Skip the dash

        let shade_num = match shade.parse::<u16>() {
            Ok(n) if (50..=950).contains(&n) => n,
            _ => return None,
        };

        // Map color + shade to RGB values (simplified mapping)
        match base {
            "red" => match shade_num {
                50 => Some((254, 242, 242)),
                100 => Some((254, 226, 226)),
                200 => Some((254, 202, 202)),
                300 => Some((252, 165, 165)),
                400 => Some((248, 113, 113)),
                500 => Some((239, 68, 68)),
                600 => Some((220, 38, 38)),
                700 => Some((185, 28, 28)),
                800 => Some((153, 27, 27)),
                900 => Some((127, 29, 29)),
                950 => Some((69, 10, 10)),
                _ => None,
            },
            "green" => match shade_num {
                50 => Some((240, 253, 244)),
                100 => Some((220, 252, 231)),
                200 => Some((187, 247, 208)),
                300 => Some((134, 239, 172)),
                400 => Some((74, 222, 128)),
                500 => Some((34, 197, 94)),
                600 => Some((22, 163, 74)),
                700 => Some((21, 128, 61)),
                800 => Some((22, 101, 52)),
                900 => Some((20, 83, 45)),
                950 => Some((5, 46, 22)),
                _ => None,
            },
            "blue" => match shade_num {
                50 => Some((239, 246, 255)),
                100 => Some((219, 234, 254)),
                200 => Some((191, 219, 254)),
                300 => Some((147, 197, 253)),
                400 => Some((96, 165, 250)),
                500 => Some((59, 130, 246)),
                600 => Some((37, 99, 235)),
                700 => Some((29, 78, 216)),
                800 => Some((30, 64, 175)),
                900 => Some((30, 58, 138)),
                950 => Some((23, 37, 84)),
                _ => None,
            },
            "yellow" => match shade_num {
                50 => Some((254, 252, 232)),
                100 => Some((254, 249, 195)),
                200 => Some((254, 240, 138)),
                300 => Some((253, 224, 71)),
                400 => Some((250, 204, 21)),
                500 => Some((234, 179, 8)),
                600 => Some((202, 138, 4)),
                700 => Some((161, 98, 7)),
                800 => Some((133, 77, 14)),
                900 => Some((113, 63, 18)),
                950 => Some((66, 32, 6)),
                _ => None,
            },
            "gray" | "grey" => match shade_num {
                50 => Some((249, 250, 251)),
                100 => Some((243, 244, 246)),
                200 => Some((229, 231, 235)),
                300 => Some((209, 213, 219)),
                400 => Some((156, 163, 175)),
                500 => Some((107, 114, 128)),
                600 => Some((75, 85, 99)),
                700 => Some((55, 65, 81)),
                800 => Some((31, 41, 55)),
                900 => Some((17, 24, 39)),
                950 => Some((3, 7, 18)),
                _ => None,
            },
            "indigo" => match shade_num {
                50 => Some((238, 242, 255)),
                100 => Some((224, 231, 255)),
                200 => Some((199, 210, 254)),
                300 => Some((165, 180, 252)),
                400 => Some((129, 140, 248)),
                500 => Some((99, 102, 241)),
                600 => Some((79, 70, 229)),
                700 => Some((67, 56, 202)),
                800 => Some((55, 48, 163)),
                900 => Some((49, 46, 129)),
                950 => Some((30, 27, 75)),
                _ => None,
            },
            "purple" => match shade_num {
                50 => Some((250, 245, 255)),
                100 => Some((243, 232, 255)),
                200 => Some((233, 213, 255)),
                300 => Some((216, 180, 254)),
                400 => Some((192, 132, 252)),
                500 => Some((168, 85, 247)),
                600 => Some((147, 51, 234)),
                700 => Some((126, 34, 206)),
                800 => Some((107, 33, 168)),
                900 => Some((88, 28, 135)),
                950 => Some((59, 7, 100)),
                _ => None,
            },
            "pink" => match shade_num {
                50 => Some((253, 242, 248)),
                100 => Some((252, 231, 243)),
                200 => Some((251, 207, 232)),
                300 => Some((249, 168, 212)),
                400 => Some((244, 114, 182)),
                500 => Some((236, 72, 153)),
                600 => Some((219, 39, 119)),
                700 => Some((190, 24, 93)),
                800 => Some((157, 23, 77)),
                900 => Some((131, 24, 67)),
                950 => Some((80, 7, 36)),
                _ => None,
            },
            "cyan" => match shade_num {
                50 => Some((236, 254, 255)),
                100 => Some((207, 250, 254)),
                200 => Some((165, 243, 252)),
                300 => Some((103, 232, 249)),
                400 => Some((34, 211, 238)),
                500 => Some((6, 182, 212)),
                600 => Some((8, 145, 178)),
                700 => Some((14, 116, 144)),
                800 => Some((21, 94, 117)),
                900 => Some((22, 78, 99)),
                950 => Some((8, 51, 68)),
                _ => None,
            },
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_colors() {
        let style = extract_visual_style("text-red-500");
        assert_eq!(style.fg, Some((239, 68, 68)));

        let style = extract_visual_style("text-white");
        assert_eq!(style.fg, Some((255, 255, 255)));

        let style = extract_visual_style("text-green-300");
        assert_eq!(style.fg, Some((134, 239, 172)));
    }

    #[test]
    fn test_extract_bg_colors() {
        let style = extract_visual_style("bg-black");
        assert_eq!(style.bg, Some((0, 0, 0)));

        let style = extract_visual_style("bg-blue-500");
        assert_eq!(style.bg, Some((59, 130, 246)));
    }

    #[test]
    fn test_extract_multiple_styles() {
        let style = extract_visual_style("text-white bg-red-600 font-bold underline");
        assert_eq!(style.fg, Some((255, 255, 255)));
        assert_eq!(style.bg, Some((220, 38, 38)));
        assert!(style.bold);
        assert!(style.underline);
        assert!(!style.italic);
    }

    #[test]
    fn test_absolute_positioning_ignored() {
        // Position utilities shouldn't affect visual style
        let style = extract_visual_style("absolute left-10 top-5 text-green-500");
        assert_eq!(style.fg, Some((34, 197, 94)));
        assert_eq!(style.bg, None);
    }
}
