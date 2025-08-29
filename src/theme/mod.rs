pub mod ansi;
pub mod colors;
pub mod parser;
pub mod presets;
pub mod variables;

pub use ansi::{
    AnsiColor, ColorDepth, hex_to_ansi256, hex_to_rgb, rgb_to_ansi, rgb_to_ansi16, rgb_to_ansi256,
};
pub use colors::{Colors, get_color};
pub use parser::{ParsedStyle, ThemeParser};
pub use presets::{
    dark_theme, gruvbox_dark_theme, high_contrast_theme, light_theme, solarized_dark_theme,
};
pub use variables::ThemeVariables;

/// CSS-based theme system for reactive-tui
///
/// Themes are defined using CSS custom properties (variables) that map to
/// utility classes. This allows for easy customization and consistency
/// across components while maintaining our utility-first CSS approach.
///
/// # Example
/// ```rust, ignore
/// let theme = dark_theme();
/// let parser = ThemeParser::new(theme);
/// let style = parser.parse_class("bg-primary text-secondary");
/// ```
pub struct Theme {
    pub name: String,
    pub variables: ThemeVariables,
    pub extends: Option<Box<Theme>>,
}

impl Theme {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variables: ThemeVariables::default(),
            extends: None,
        }
    }

    pub fn with_variables(mut self, variables: ThemeVariables) -> Self {
        self.variables = variables;
        self
    }

    pub fn extend(mut self, base: Theme) -> Self {
        self.extends = Some(Box::new(base));
        self
    }

    /// Resolve a variable, checking parent themes if needed
    pub fn get_variable(&self, key: &str) -> Option<String> {
        self.variables.get(key).or_else(|| {
            self.extends
                .as_ref()
                .and_then(|parent| parent.get_variable(key))
        })
    }
}
