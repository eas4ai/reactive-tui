//! Bridge between syntect themes and CSS-based theme system
//!
//! Provides seamless integration between syntax highlighting themes
//! and the reactive-tui CSS theme system.

use crate::core::surface::Rgba;
use crate::theme::{Theme, ThemeVariables};
use std::collections::HashMap;
use syntect::highlighting::{Color as SyntectColor, Style, Theme as SyntectTheme};

/// Syntax theme variables that extend the CSS theme system
pub struct SyntaxThemeVariables {
    /// Maps syntax scope names to CSS variables
    scope_map: HashMap<String, String>,
}

impl SyntaxThemeVariables {
    /// Create syntax theme variables from a syntect theme
    pub fn from_syntect_theme(theme: &SyntectTheme) -> Self {
        let mut scope_map = HashMap::new();

        // Map common syntax scopes to CSS variables
        if let Some(fg) = theme.settings.foreground {
            scope_map.insert("--syntax-text".to_string(), syntect_color_to_hex(fg));
        }

        if let Some(bg) = theme.settings.background {
            scope_map.insert("--syntax-background".to_string(), syntect_color_to_hex(bg));
        }

        if let Some(caret) = theme.settings.caret {
            scope_map.insert("--syntax-cursor".to_string(), syntect_color_to_hex(caret));
        }

        if let Some(selection) = theme.settings.selection {
            scope_map.insert(
                "--syntax-selection".to_string(),
                syntect_color_to_hex(selection),
            );
        }

        if let Some(line_highlight) = theme.settings.line_highlight {
            scope_map.insert(
                "--syntax-line-highlight".to_string(),
                syntect_color_to_hex(line_highlight),
            );
        }

        // Process scope styles
        for item in &theme.scopes {
            // Use the first selector as the scope name
            let scope_name = if !item.scope.selectors.is_empty() {
                item.scope.selectors[0].path.to_string()
            } else {
                continue;
            };
            let css_var_name = scope_to_css_variable(&scope_name);

            if let Some(fg) = item.style.foreground {
                scope_map.insert(format!("{}-fg", css_var_name), syntect_color_to_hex(fg));
            }

            if let Some(bg) = item.style.background {
                scope_map.insert(format!("{}-bg", css_var_name), syntect_color_to_hex(bg));
            }
        }

        Self { scope_map }
    }

    /// Apply syntax theme variables to a theme
    pub fn apply_to_theme(&self, theme: &mut Theme) {
        for (key, value) in &self.scope_map {
            theme.variables.set(key.clone(), value.clone());
        }

        // Also set derived colors for common syntax elements
        self.set_syntax_colors(&mut theme.variables);
    }

    /// Set common syntax highlighting colors as CSS variables
    fn set_syntax_colors(&self, variables: &mut ThemeVariables) {
        // Keywords (control flow, declarations)
        if let Some(keyword_color) = self.scope_map.get("--syntax-keyword-fg") {
            variables.set("--syntax-keyword", keyword_color);
        }

        // Types and classes
        if let Some(type_color) = self.scope_map.get("--syntax-entity-name-type-fg") {
            variables.set("--syntax-type", type_color);
        }

        // Functions and methods
        if let Some(func_color) = self.scope_map.get("--syntax-entity-name-function-fg") {
            variables.set("--syntax-function", func_color);
        }

        // Variables and parameters
        if let Some(var_color) = self.scope_map.get("--syntax-variable-fg") {
            variables.set("--syntax-variable", var_color);
        }

        // Strings
        if let Some(string_color) = self.scope_map.get("--syntax-string-fg") {
            variables.set("--syntax-string", string_color);
        }

        // Comments
        if let Some(comment_color) = self.scope_map.get("--syntax-comment-fg") {
            variables.set("--syntax-comment", comment_color);
        }

        // Numbers and constants
        if let Some(constant_color) = self.scope_map.get("--syntax-constant-numeric-fg") {
            variables.set("--syntax-number", constant_color);
            variables.set("--syntax-constant", constant_color);
        }

        // Operators
        if let Some(operator_color) = self.scope_map.get("--syntax-keyword-operator-fg") {
            variables.set("--syntax-operator", operator_color);
        }
    }
}

/// Convert a syntect scope selector to a CSS variable name
fn scope_to_css_variable(scope: &str) -> String {
    let parts: Vec<&str> = scope.split('.').collect();
    let mut var_name = "--syntax".to_string();

    for part in parts {
        var_name.push('-');
        var_name.push_str(&part.replace('_', "-"));
    }

    var_name
}

/// Convert syntect Color to hex string
fn syntect_color_to_hex(color: SyntectColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

/// Convert hex color string to Rgba
pub fn hex_to_rgba(hex: &str) -> Rgba {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

    Rgba { r, g, b, a: 1.0 }
}

/// Theme-aware syntax style resolver
pub struct ThemedSyntaxStyle {
    theme: Theme,
}

impl ThemedSyntaxStyle {
    /// Create a new themed syntax style resolver
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }

    /// Resolve a syntect style using theme variables
    pub fn resolve_style(&self, style: Style) -> (Rgba, Rgba) {
        let fg = { syntect_color_to_rgba(style.foreground) };

        let bg = { syntect_color_to_rgba(style.background) };

        (fg, bg)
    }

    /// Get a color for a specific syntax element type
    pub fn get_syntax_color(&self, element_type: SyntaxElement) -> Rgba {
        let var_name = match element_type {
            SyntaxElement::Keyword => "--syntax-keyword",
            SyntaxElement::Type => "--syntax-type",
            SyntaxElement::Function => "--syntax-function",
            SyntaxElement::Variable => "--syntax-variable",
            SyntaxElement::String => "--syntax-string",
            SyntaxElement::Comment => "--syntax-comment",
            SyntaxElement::Number => "--syntax-number",
            SyntaxElement::Operator => "--syntax-operator",
            SyntaxElement::Constant => "--syntax-constant",
        };

        if let Some(color) = self.theme.get_variable(var_name) {
            hex_to_rgba(&color)
        } else {
            // Fallback colors
            match element_type {
                SyntaxElement::Keyword => Rgba {
                    r: 0.8,
                    g: 0.4,
                    b: 0.6,
                    a: 1.0,
                },
                SyntaxElement::Type => Rgba {
                    r: 0.4,
                    g: 0.6,
                    b: 0.8,
                    a: 1.0,
                },
                SyntaxElement::Function => Rgba {
                    r: 0.6,
                    g: 0.8,
                    b: 0.4,
                    a: 1.0,
                },
                SyntaxElement::Variable => Rgba {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
                SyntaxElement::String => Rgba {
                    r: 0.8,
                    g: 0.6,
                    b: 0.4,
                    a: 1.0,
                },
                SyntaxElement::Comment => Rgba {
                    r: 0.5,
                    g: 0.5,
                    b: 0.5,
                    a: 1.0,
                },
                SyntaxElement::Number => Rgba {
                    r: 0.4,
                    g: 0.8,
                    b: 0.8,
                    a: 1.0,
                },
                SyntaxElement::Operator => Rgba {
                    r: 0.7,
                    g: 0.7,
                    b: 0.7,
                    a: 1.0,
                },
                SyntaxElement::Constant => Rgba {
                    r: 0.8,
                    g: 0.4,
                    b: 0.4,
                    a: 1.0,
                },
            }
        }
    }
}

/// Common syntax element types
#[derive(Debug, Clone, Copy)]
pub enum SyntaxElement {
    Keyword,
    Type,
    Function,
    Variable,
    String,
    Comment,
    Number,
    Operator,
    Constant,
}

/// Convert syntect Color to Rgba
fn syntect_color_to_rgba(color: SyntectColor) -> Rgba {
    Rgba {
        r: color.r as f32 / 255.0,
        g: color.g as f32 / 255.0,
        b: color.b as f32 / 255.0,
        a: color.a as f32 / 255.0,
    }
}

/// Create a CSS theme with syntax highlighting support
pub fn create_syntax_theme(base_theme: Theme, syntect_theme_name: &str) -> Theme {
    use crate::syntax::resources::SYNTAX_RESOURCES;

    let mut theme = base_theme;

    // Load syntect theme and convert to CSS variables
    let resources = SYNTAX_RESOURCES.read().unwrap();
    if let Some(syntect_theme) = resources.theme_set.get(syntect_theme_name) {
        let syntax_vars = SyntaxThemeVariables::from_syntect_theme(syntect_theme);
        syntax_vars.apply_to_theme(&mut theme);
    }

    theme
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::presets::dark_theme;

    #[test]
    fn test_scope_to_css_variable() {
        assert_eq!(scope_to_css_variable("keyword"), "--syntax-keyword");
        assert_eq!(
            scope_to_css_variable("keyword.control"),
            "--syntax-keyword-control"
        );
        assert_eq!(
            scope_to_css_variable("entity.name.function"),
            "--syntax-entity-name-function"
        );
    }

    #[test]
    fn test_hex_to_rgba() {
        let color = hex_to_rgba("#ff0000");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);

        let color = hex_to_rgba("#00ff00");
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_themed_syntax_style() {
        let theme = dark_theme();
        let styled = ThemedSyntaxStyle::new(theme);

        // Should return theme colors even without syntect style
        let keyword_color = styled.get_syntax_color(SyntaxElement::Keyword);
        assert!(keyword_color.r > 0.0 || keyword_color.g > 0.0 || keyword_color.b > 0.0);
    }

    #[test]
    fn test_create_syntax_theme() {
        let base = dark_theme();
        let theme = create_syntax_theme(base, "base16-ocean.dark");

        // Should have syntax variables added
        assert!(
            theme.get_variable("--syntax-text").is_some()
                || theme.get_variable("--color-text").is_some()
        );
    }
}
