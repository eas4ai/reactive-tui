/// ANSI color support and conversion utilities
pub mod ansi;
/// Color definitions and palettes
pub mod colors;
/// Pre-built theme presets
pub mod presets;
/// Theme variable system
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
use std::sync::{Arc, OnceLock, RwLock};

/// How many times the active theme has changed. A cache that keeps colors
/// resolved through the active theme, on any thread, compares this with the
/// count it filled under and empties itself when they differ.
static GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn active_slot() -> &'static RwLock<Arc<Theme>> {
    static ACTIVE: OnceLock<RwLock<Arc<Theme>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(Arc::new(presets::dark_theme())))
}

/// CSS-first theme system for reactive-tui
///
/// Themes provide CSS custom properties (variables) that integrate seamlessly
/// with the CSS utility system. All styling flows through the CSS module,
/// with themes providing customization through variables.
///
/// # Example
/// ```rust, ignore
/// use reactive_tui::theme::dark_theme;
///
/// let theme = dark_theme();
/// let style = theme.apply_classes("bg-primary text-secondary p-4");
/// ```
#[derive(Debug, Clone)]
pub struct Theme {
    /// Name of the theme
    pub name: String,
    /// Theme variables and their values
    pub variables: ThemeVariables,
    /// Parent theme to inherit from
    pub extends: Option<Box<Theme>>,
}

impl Theme {
    /// Create a new theme with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            variables: ThemeVariables::default(),
            extends: None,
        }
    }

    /// Set the theme variables
    pub fn with_variables(mut self, variables: ThemeVariables) -> Self {
        self.variables = variables;
        self
    }

    /// Extend this theme from a base theme
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

    /// The theme the application currently uses. Components read it through
    /// [`crate::reactive::hooks::use_theme`]; the dark preset applies until
    /// [`Theme::set_active`] or [`crate::app::App::set_theme`] replaces it.
    pub fn active() -> Arc<Theme> {
        active_slot()
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Make `theme` the active theme for every component and utility class.
    pub fn set_active(theme: Theme) -> Arc<Theme> {
        let theme = Arc::new(theme);
        *active_slot().write().unwrap_or_else(|e| e.into_inner()) = theme.clone();
        // After the new theme is in place, so a thread that sees the new
        // count also sees the new theme.
        GENERATION.fetch_add(1, std::sync::atomic::Ordering::Release);
        crate::layout::css::cache::clear_color_cache();
        theme
    }

    /// How many times [`Theme::set_active`] has run in this process.
    pub(crate) fn generation() -> u64 {
        GENERATION.load(std::sync::atomic::Ordering::Acquire)
    }

    /// The variable name a color token refers to: `primary` and
    /// `--color-primary` both name `--color-primary`; `muted` names
    /// `--color-text-muted`.
    pub fn color_variable(token: &str) -> String {
        match token {
            "muted" => "--color-text-muted".to_string(),
            t if t.starts_with("--") => t.to_string(),
            t => format!("--color-{t}"),
        }
    }

    /// Resolve a token that names one of this theme's color variables.
    pub fn resolve_variable(&self, token: &str) -> Option<(f32, f32, f32, f32)> {
        let value = self.get_variable(&Self::color_variable(token))?;
        crate::layout::colors::parse_color_literal(&value).or_else(|| {
            (value != token)
                .then(|| self.resolve_variable(&value))
                .flatten()
        })
    }

    /// The one color resolver (CHT-017): a theme variable name such as
    /// `primary` or `chart-1`, a palette name such as `blue-500`, or hex.
    /// Utility classes and chart colors both resolve through here.
    pub fn resolve_color(&self, token: &str) -> Option<(f32, f32, f32, f32)> {
        self.resolve_variable(token)
            .or_else(|| crate::layout::colors::parse_color_literal(token))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api019_child_theme_inherits_and_overrides_without_cross_instance_state() {
        let base = Theme::new("base").with_variables(
            ThemeVariables::new()
                .set("--color-primary", "#112233")
                .set("--spacing-md", "4"),
        );
        let child = Theme::new("child")
            .with_variables(ThemeVariables::new().set("--color-primary", "#abcdef"))
            .extend(base);
        let independent = Theme::new("independent")
            .with_variables(ThemeVariables::new().set("--color-primary", "#010203"));

        assert_eq!(child.get_variable("--spacing-md").as_deref(), Some("4"));
        assert_eq!(
            child.get_variable("--color-primary").as_deref(),
            Some("#abcdef")
        );
        assert_eq!(
            independent.get_variable("--color-primary").as_deref(),
            Some("#010203")
        );
        assert_eq!(independent.get_variable("--spacing-md"), None);

        assert_eq!(
            child.apply_classes("text-primary").fg_rgba,
            Some((171.0 / 255.0, 205.0 / 255.0, 239.0 / 255.0, 1.0))
        );
        assert_eq!(
            independent.apply_classes("text-primary").fg_rgba,
            Some((1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 1.0))
        );
    }
}
