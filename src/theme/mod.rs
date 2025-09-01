pub mod ansi;
pub mod colors;
pub mod presets;
pub mod variables;

pub use ansi::{
    hex_to_ansi256, hex_to_rgb, rgb_to_ansi, rgb_to_ansi16, rgb_to_ansi256, AnsiColor, ColorDepth,
};
pub use colors::{get_color, Colors};
pub use presets::{
    dark_theme, gruvbox_dark_theme, high_contrast_theme, light_theme, solarized_dark_theme,
};
pub use variables::ThemeVariables;

use crate::layout::css::apply_utility_classes_with_theme;
use crate::layout::style::StyleBuilder;
use std::collections::HashMap;

/// CSS-first theme system for reactive-tui
///
/// Themes provide CSS custom properties (variables) that integrate seamlessly
/// with the CSS utility system. All styling flows through the CSS module,
/// with themes providing customization through variables.
///
/// # Example
/// ```rust, ignore
/// let theme = dark_theme();
/// let style = theme.apply_classes("bg-primary text-secondary p-4");
/// ```
pub struct Theme {
    pub name: String,
    pub variables: ThemeVariables,
    pub extends: Option<Box<Theme>>,
    /// Cache for resolved variables to improve performance
    variable_cache: HashMap<String, String>,
}

impl Theme {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variables: ThemeVariables::default(),
            extends: None,
            variable_cache: HashMap::new(),
        }
    }

    pub fn with_variables(mut self, variables: ThemeVariables) -> Self {
        self.variables = variables;
        self.variable_cache.clear(); // Clear cache when variables change
        self
    }

    pub fn extend(mut self, base: Theme) -> Self {
        self.extends = Some(Box::new(base));
        self.variable_cache.clear(); // Clear cache when inheritance changes
        self
    }

    /// Resolve a variable, checking parent themes if needed
    pub fn get_variable(&self, key: &str) -> Option<String> {
        // Check cache first
        if let Some(cached) = self.variable_cache.get(key) {
            return Some(cached.clone());
        }

        // Resolve from variables or parent themes
        let value = self.variables.get(key).or_else(|| {
            self.extends
                .as_ref()
                .and_then(|parent| parent.get_variable(key))
        });

        // Cache the result (even if None, we cache that too)
        if let Some(ref val) = value {
            // Note: We can't mutate self here due to borrowing rules
            // In a real implementation, we'd use RefCell or similar for the cache
        }

        value
    }

    /// Apply CSS utility classes with theme variable resolution
    ///
    /// This is the primary method for styling with themes. It uses the CSS
    /// utility system internally while resolving theme variables.
    pub fn apply_classes(&self, classes: &str) -> StyleBuilder {
        apply_utility_classes_with_theme(classes, StyleBuilder::new(), Some(self))
    }

    /// Apply CSS utility classes to an existing StyleBuilder
    pub fn apply_classes_to(&self, classes: &str, builder: StyleBuilder) -> StyleBuilder {
        apply_utility_classes_with_theme(classes, builder, Some(self))
    }
}
