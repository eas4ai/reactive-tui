use std::collections::HashMap;
use super::Theme;

/// Parser for CSS utility classes that resolves theme variables
/// 
/// Maps Tailwind-style utility classes to actual styles using theme variables.
/// For example: "bg-primary" -> background color from --color-primary
pub struct ThemeParser {
    theme: Theme,
    cache: HashMap<String, ParsedStyle>,
}

#[derive(Debug, Clone)]
pub struct ParsedStyle {
    pub fg: Option<(u8, u8, u8)>,
    pub bg: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
    pub border: Option<BorderStyle>,
    pub padding: Option<Padding>,
    pub margin: Option<Margin>,
}

#[derive(Debug, Clone)]
pub struct BorderStyle {
    pub color: (u8, u8, u8),
    pub width: u16,
    pub style: String,
}

#[derive(Debug, Clone)]
pub struct Padding {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

#[derive(Debug, Clone)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl ThemeParser {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            cache: HashMap::new(),
        }
    }

    /// Parse a string of utility classes into a style
    pub fn parse(&mut self, classes: &str) -> ParsedStyle {
        if let Some(cached) = self.cache.get(classes) {
            return cached.clone();
        }

        let mut style = ParsedStyle::default();
        
        for class in classes.split_whitespace() {
            self.parse_single_class(class, &mut style);
        }
        
        self.cache.insert(classes.to_string(), style.clone());
        style
    }

    fn parse_single_class(&self, class: &str, style: &mut ParsedStyle) {
        // Text colors - map to Tailwind palette or theme variables
        match class {
            // Theme-based colors
            "text-primary" => style.fg = self.get_rgb_from_var("--color-primary"),
            "text-secondary" => style.fg = self.get_rgb_from_var("--color-secondary"),
            "text-muted" => style.fg = self.get_rgb_from_var("--color-text-muted"),
            
            // gray scale
            "text-gray-50" => style.fg = Some((249, 250, 251)),
            "text-gray-100" => style.fg = Some((243, 244, 246)),
            "text-gray-200" => style.fg = Some((229, 231, 235)),
            "text-gray-300" => style.fg = Some((209, 213, 219)),
            "text-gray-400" => style.fg = Some((156, 163, 175)),
            "text-gray-500" => style.fg = Some((107, 114, 128)),
            "text-gray-600" => style.fg = Some((75, 85, 99)),
            "text-gray-700" => style.fg = Some((55, 65, 81)),
            "text-gray-800" => style.fg = Some((31, 41, 55)),
            "text-gray-900" => style.fg = Some((17, 24, 39)),
            "text-gray-950" => style.fg = Some((3, 7, 18)),
            
            // blue scale
            "text-blue-50" => style.fg = Some((239, 246, 255)),
            "text-blue-100" => style.fg = Some((219, 234, 254)),
            "text-blue-200" => style.fg = Some((191, 219, 254)),
            "text-blue-300" => style.fg = Some((147, 197, 253)),
            "text-blue-400" => style.fg = Some((96, 165, 250)),
            "text-blue-500" => style.fg = Some((59, 130, 246)),
            "text-blue-600" => style.fg = Some((37, 99, 235)),
            "text-blue-700" => style.fg = Some((29, 78, 216)),
            "text-blue-800" => style.fg = Some((30, 64, 175)),
            "text-blue-900" => style.fg = Some((30, 58, 138)),
            "text-blue-950" => style.fg = Some((23, 37, 84)),
            
            // Background colors
            "bg-primary" => style.bg = self.get_rgb_from_var("--color-primary"),
            "bg-secondary" => style.bg = self.get_rgb_from_var("--color-secondary"),
            
            // gray backgrounds
            "bg-gray-50" => style.bg = Some((249, 250, 251)),
            "bg-gray-100" => style.bg = Some((243, 244, 246)),
            "bg-gray-200" => style.bg = Some((229, 231, 235)),
            "bg-gray-300" => style.bg = Some((209, 213, 219)),
            "bg-gray-400" => style.bg = Some((156, 163, 175)),
            "bg-gray-500" => style.bg = Some((107, 114, 128)),
            "bg-gray-600" => style.bg = Some((75, 85, 99)),
            "bg-gray-700" => style.bg = Some((55, 65, 81)),
            "bg-gray-800" => style.bg = Some((31, 41, 55)),
            "bg-gray-900" => style.bg = Some((17, 24, 39)),
            "bg-gray-950" => style.bg = Some((3, 7, 18)),
            
            // blue backgrounds
            "bg-blue-50" => style.bg = Some((239, 246, 255)),
            "bg-blue-100" => style.bg = Some((219, 234, 254)),
            "bg-blue-200" => style.bg = Some((191, 219, 254)),
            "bg-blue-300" => style.bg = Some((147, 197, 253)),
            "bg-blue-400" => style.bg = Some((96, 165, 250)),
            "bg-blue-500" => style.bg = Some((59, 130, 246)),
            "bg-blue-600" => style.bg = Some((37, 99, 235)),
            "bg-blue-700" => style.bg = Some((29, 78, 216)),
            "bg-blue-800" => style.bg = Some((30, 64, 175)),
            "bg-blue-900" => style.bg = Some((30, 58, 138)),
            "bg-blue-950" => style.bg = Some((23, 37, 84)),
            
            // Semantic colors
            "text-success" => style.fg = self.get_rgb_from_var("--color-success"),
            "text-warning" => style.fg = self.get_rgb_from_var("--color-warning"),
            "text-error" => style.fg = self.get_rgb_from_var("--color-error"),
            "text-info" => style.fg = self.get_rgb_from_var("--color-info"),
            
            "bg-success" => style.bg = self.get_rgb_from_var("--color-success"),
            "bg-warning" => style.bg = self.get_rgb_from_var("--color-warning"),
            "bg-error" => style.bg = self.get_rgb_from_var("--color-error"),
            "bg-info" => style.bg = self.get_rgb_from_var("--color-info"),
            
            // Text decoration
            "font-bold" | "bold" => style.bold = true,
            "font-normal" => style.bold = false,
            "italic" => style.italic = true,
            "underline" => style.underline = true,
            "dim" => style.dim = true,
            
            // Borders
            "border" => {
                if let Some(color) = self.get_rgb_from_var("--color-border") {
                    style.border = Some(BorderStyle {
                        color,
                        width: 1,
                        style: "solid".to_string(),
                    });
                }
            },
            "border-2" => {
                if let Some(mut border) = style.border.clone() {
                    border.width = 2;
                    style.border = Some(border);
                }
            },
            
            // Padding
            "p-0" => style.padding = Some(Padding { top: 0, right: 0, bottom: 0, left: 0 }),
            "p-1" => style.padding = Some(Padding { top: 1, right: 1, bottom: 1, left: 1 }),
            "p-2" => style.padding = Some(Padding { top: 2, right: 2, bottom: 2, left: 2 }),
            "p-3" => style.padding = Some(Padding { top: 3, right: 3, bottom: 3, left: 3 }),
            "p-4" => style.padding = Some(Padding { top: 4, right: 4, bottom: 4, left: 4 }),
            
            "px-1" => {
                if let Some(ref mut p) = style.padding {
                    p.left = 1;
                    p.right = 1;
                } else {
                    style.padding = Some(Padding { top: 0, right: 1, bottom: 0, left: 1 });
                }
            },
            "px-2" => {
                if let Some(ref mut p) = style.padding {
                    p.left = 2;
                    p.right = 2;
                } else {
                    style.padding = Some(Padding { top: 0, right: 2, bottom: 0, left: 2 });
                }
            },
            "py-1" => {
                if let Some(ref mut p) = style.padding {
                    p.top = 1;
                    p.bottom = 1;
                } else {
                    style.padding = Some(Padding { top: 1, right: 0, bottom: 1, left: 0 });
                }
            },
            "py-2" => {
                if let Some(ref mut p) = style.padding {
                    p.top = 2;
                    p.bottom = 2;
                } else {
                    style.padding = Some(Padding { top: 2, right: 0, bottom: 2, left: 0 });
                }
            },
            
            // Margins
            "m-0" => style.margin = Some(Margin { top: 0, right: 0, bottom: 0, left: 0 }),
            "m-1" => style.margin = Some(Margin { top: 1, right: 1, bottom: 1, left: 1 }),
            "m-2" => style.margin = Some(Margin { top: 2, right: 2, bottom: 2, left: 2 }),
            "m-3" => style.margin = Some(Margin { top: 3, right: 3, bottom: 3, left: 3 }),
            "m-4" => style.margin = Some(Margin { top: 4, right: 4, bottom: 4, left: 4 }),
            
            _ => {
                // Try to parse dynamic classes like hover:bg-blue-700
                if class.starts_with("hover:") {
                    // Store for hover state handling
                } else if class.starts_with("focus:") {
                    // Store for focus state handling
                }
            }
        }
    }
    
    fn get_rgb_from_var(&self, var_name: &str) -> Option<(u8, u8, u8)> {
        self.theme.get_variable(var_name)
            .and_then(|hex| parse_hex_color(&hex))
    }
}

impl Default for ParsedStyle {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
            border: None,
            padding: None,
            margin: None,
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    
    Some((r, g, b))
}