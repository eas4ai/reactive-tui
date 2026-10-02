//! Bridge between Lumis themes and CSS-based theme system
//!
//! Provides seamless integration between syntax highlighting themes
//! and the reactive-tui CSS theme system.

use crate::core::surface::Rgba;
use crate::theme::{Theme, ThemeVariables};
use lumis::highlight::Style as LumisStyle;
use lumis::themes::Theme as LumisTheme;
use std::collections::HashMap;

/// Syntax theme variables that extend the CSS theme system
pub struct SyntaxThemeVariables {
    /// Maps syntax scope names to CSS variables
    scope_map: HashMap<String, String>,
}

impl SyntaxThemeVariables {
    /// Create syntax theme variables from a Lumis theme.
    ///
    /// Every scope in the theme's highlight map becomes `--syntax-<path>`
    /// variables (dotted scope paths map to dashed names, case-insensitively
    /// so `@`-prefixed captures and capitalized groups land alike). The
    /// `normal` scope additionally feeds `--syntax-text`/`--syntax-background`.
    pub fn from_lumis_theme(theme: &LumisTheme) -> Self {
        let mut scope_map = HashMap::new();

        // Lumis themes use lowercase tree-sitter scopes; `normal` is the
        // base-text scope (there are no Normal/Cursor/Visual groups).
        if let Some(normal) = theme.highlights.get("normal") {
            if let Some(fg) = normal.fg.as_deref() {
                scope_map.insert("--syntax-text".to_string(), fg.to_string());
            }
            if let Some(bg) = normal.bg.as_deref() {
                scope_map.insert("--syntax-background".to_string(), bg.to_string());
            }
        }

        // Process scope styles
        for (scope_name, style) in &theme.highlights {
            let css_var_name = scope_to_css_variable(scope_name);

            if let Some(fg) = style.fg.as_deref() {
                scope_map.insert(format!("{css_var_name}-fg"), fg.to_string());
            }

            if let Some(bg) = style.bg.as_deref() {
                scope_map.insert(format!("{css_var_name}-bg"), bg.to_string());
            }
        }

        Self { scope_map }
    }

    /// Apply syntax theme variables to a theme
    pub fn apply_to_theme(&self, theme: &mut Theme) {
        for (key, value) in &self.scope_map {
            theme.variables = theme.variables.clone().set(key.clone(), value.clone());
        }

        // Also set derived colors for common syntax elements
        self.set_syntax_colors(&mut theme.variables);
    }

    /// Set common syntax highlighting colors as CSS variables
    fn set_syntax_colors(&self, variables: &mut ThemeVariables) {
        // Keywords (control flow, declarations)
        if let Some(keyword_color) = self.scope_map.get("--syntax-keyword-fg") {
            *variables = variables.clone().set("--syntax-keyword", keyword_color);
        }

        // Types and classes
        if let Some(type_color) = self
            .scope_map
            .get("--syntax-type-fg")
            .or_else(|| self.scope_map.get("--syntax-entity-name-type-fg"))
        {
            *variables = variables.clone().set("--syntax-type", type_color);
        }

        // Functions and methods
        if let Some(func_color) = self
            .scope_map
            .get("--syntax-function-fg")
            .or_else(|| self.scope_map.get("--syntax-entity-name-function-fg"))
        {
            *variables = variables.clone().set("--syntax-function", func_color);
        }

        // Variables and parameters
        if let Some(var_color) = self.scope_map.get("--syntax-variable-fg") {
            *variables = variables.clone().set("--syntax-variable", var_color);
        }

        // Strings
        if let Some(string_color) = self.scope_map.get("--syntax-string-fg") {
            *variables = variables.clone().set("--syntax-string", string_color);
        }

        // Comments
        if let Some(comment_color) = self.scope_map.get("--syntax-comment-fg") {
            *variables = variables.clone().set("--syntax-comment", comment_color);
        }

        // Numbers and constants
        if let Some(constant_color) = self
            .scope_map
            .get("--syntax-number-fg")
            .or_else(|| self.scope_map.get("--syntax-constant-numeric-fg"))
            .or_else(|| self.scope_map.get("--syntax-constant-fg"))
        {
            *variables = variables.clone().set("--syntax-number", constant_color);
            *variables = variables.clone().set("--syntax-constant", constant_color);
        }

        // Operators
        if let Some(operator_color) = self
            .scope_map
            .get("--syntax-operator-fg")
            .or_else(|| self.scope_map.get("--syntax-keyword-operator-fg"))
        {
            *variables = variables.clone().set("--syntax-operator", operator_color);
        }
    }
}

/// Convert a highlight scope selector to a CSS variable name.
///
/// Accepts dotted paths (`keyword.control`), `@`-prefixed tree-sitter
/// captures (`@keyword`), and capitalized Neovim groups (`Keyword`);
/// all map case-insensitively with underscores flattened to dashes.
fn scope_to_css_variable(scope: &str) -> String {
    let scope = scope.strip_prefix('@').unwrap_or(scope);
    let parts: Vec<&str> = scope.split('.').collect();
    let mut var_name = "--syntax".to_string();

    for part in parts {
        var_name.push('-');
        var_name.push_str(&part.replace('_', "-").to_lowercase());
    }

    var_name
}

/// Convert hex color string to Rgba. Delegates to the canonical parser in
/// `crate::layout::colors`, so `#rgb`, `#rgba` and `#rrggbbaa` work here
/// exactly as everywhere else.
///
/// Breaking change in 0.1.0: returns `None` on malformed input. The old
/// version panicked on short input and silently returned black on bad
/// digits; callers must fall back explicitly (see `get_syntax_color`).
pub fn hex_to_rgba(hex: &str) -> Option<Rgba> {
    let (r, g, b, a) = crate::layout::colors::parse_hex_bytes(hex)?;
    Some(Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: a as f32 / 255.0,
    })
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

    /// Resolve a Lumis style using theme variables.
    ///
    /// A missing foreground falls back to white and a missing background
    /// to transparent, matching the highlighter's run convention.
    pub fn resolve_style(&self, style: &LumisStyle) -> (Rgba, Rgba) {
        let fg = style
            .fg
            .as_deref()
            .and_then(hex_to_rgba)
            .unwrap_or(Rgba::white());

        let bg = style
            .bg
            .as_deref()
            .and_then(hex_to_rgba)
            .unwrap_or(Rgba::transparent());

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
            if let Some(rgba) = hex_to_rgba(&color) {
                return rgba;
            }
        }
        // Fallback colors
        {
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
    /// Language keywords (if, for, while, etc.)
    Keyword,
    /// Type names and type annotations
    Type,
    /// Function and method names
    Function,
    /// Variable and parameter names
    Variable,
    /// String literals
    String,
    /// Comments and documentation
    Comment,
    /// Numeric literals
    Number,
    /// Operators (+, -, *, etc.)
    Operator,
    /// Constants and literals
    Constant,
}

/// Create a CSS theme with syntax highlighting support
pub fn create_syntax_theme(base_theme: Theme, theme_name: &str) -> Theme {
    use crate::syntax::resources::SYNTAX_RESOURCES;

    let mut theme = base_theme;

    // Load Lumis theme and convert to CSS variables
    match SYNTAX_RESOURCES.read() {
        Ok(resources) => {
            if let Some(lumis_theme) = resources.theme_by_name(theme_name) {
                let syntax_vars = SyntaxThemeVariables::from_lumis_theme(&lumis_theme);
                syntax_vars.apply_to_theme(&mut theme);
            }
        }
        Err(_) => {
            // Lock poisoned, use default theme
            log::warn!("Syntax resources lock poisoned, using default theme");
        }
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
        assert_eq!(scope_to_css_variable("@keyword"), "--syntax-keyword");
        assert_eq!(scope_to_css_variable("Keyword"), "--syntax-keyword");
    }

    #[test]
    fn test_hex_to_rgba() {
        let color = hex_to_rgba("#ff0000").expect("valid hex");
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);

        let color = hex_to_rgba("#00ff00").expect("valid hex");
        assert_eq!(color.r, 0.0);
        assert_eq!(color.g, 1.0);
        assert_eq!(color.b, 0.0);
    }

    #[test]
    fn test_hex_to_rgba_rejects_malformed() {
        // Short input panicked and bad digits silently returned black
        // before the canonical-parser migration.
        for bad in ["", "#", "#ff", "#gg0000", "#ff0000 ", "red"] {
            assert_eq!(hex_to_rgba(bad), None, "input: {bad:?}");
        }
    }

    #[test]
    fn test_themed_syntax_style() {
        let theme = dark_theme();
        let styled = ThemedSyntaxStyle::new(theme);

        // Should return theme colors even without a Lumis style
        let keyword_color = styled.get_syntax_color(SyntaxElement::Keyword);
        assert!(keyword_color.r > 0.0 || keyword_color.g > 0.0 || keyword_color.b > 0.0);
    }

    #[test]
    fn test_create_syntax_theme() {
        let base = dark_theme();
        let theme = create_syntax_theme(base, "onedark");

        // Should have syntax variables added
        assert!(
            theme.get_variable("--syntax-text").is_some()
                || theme.get_variable("--color-text").is_some()
        );
    }
}
