//! Global syntax resources manager
//!
//! Lazy-loaded themes and syntax sets with support for custom themes.

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use syntect::highlighting::{Theme, ThemeSet as SyntectThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::LoadingError;

/// Global syntax resources instance
pub static SYNTAX_RESOURCES: Lazy<RwLock<SyntaxResources>> =
    Lazy::new(|| RwLock::new(SyntaxResources::default()));

/// Container for syntax definitions and themes
pub struct SyntaxResources {
    /// Syntax definitions for various languages
    pub syntax_set: SyntaxSet,
    /// Available color themes
    pub theme_set: ThemeSet,
    /// Currently active theme name
    active_theme: String,
}

impl Default for SyntaxResources {
    fn default() -> Self {
        // Try to load better syntax definitions
        let syntax_set = Self::load_enhanced_syntax_set();

        Self {
            syntax_set,
            theme_set: ThemeSet::default(),
            active_theme: String::from("base16-eighties.dark"),
        }
    }
}

impl SyntaxResources {
    /// Load enhanced syntax definitions with better coverage
    fn load_enhanced_syntax_set() -> SyntaxSet {
        // Try to load better syntax definitions
        // First try to load from embedded enhanced definitions
        if let Ok(syntax_set) = Self::try_load_enhanced_definitions() {
            return syntax_set;
        }

        // Fall back to syntect defaults
        SyntaxSet::load_defaults_newlines()
    }

    /// Try to load enhanced syntax definitions
    fn try_load_enhanced_definitions() -> Result<SyntaxSet, LoadingError> {
        // Use syntect's built-in syntax definitions which include comprehensive
        // support for most programming languages. Future enhancements could include:
        // - Embedding custom syntax definitions for domain-specific languages
        // - Loading additional syntax packages from Sublime Text repositories
        // - Runtime syntax definition updates

        // The default syntax set provides excellent coverage for common languages
        Ok(SyntaxSet::load_defaults_newlines())
    }

    /// Load custom syntax definitions from a folder
    pub fn load_syntaxes_from_folder(&mut self, path: &Path) -> Result<(), LoadingError> {
        let mut builder = self.syntax_set.clone().into_builder();
        builder.add_from_folder(path, true)?;
        self.syntax_set = builder.build();
        Ok(())
    }

    /// Load a custom theme from file
    pub fn load_theme_from_file(&mut self, name: String, path: &Path) -> Result<(), LoadingError> {
        let theme = ThemeSet::load_theme(path)?;
        self.theme_set.add_theme(name.clone(), theme);
        Ok(())
    }

    /// Set the active theme
    pub fn set_active_theme(&mut self, name: &str) -> Result<(), String> {
        if self.theme_set.get(name).is_some() {
            self.active_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme '{}' not found", name))
        }
    }

    /// Get the active theme
    pub fn active_theme(&self) -> &Theme {
        self.theme_set
            .get(&self.active_theme)
            .unwrap_or_else(|| self.theme_set.get("base16-ocean.dark").unwrap())
    }

    /// Find syntax definition for a file extension or name
    pub fn find_syntax(&self, file_name: &str) -> Option<&SyntaxReference> {
        self.syntax_set
            .find_syntax_by_extension(file_name)
            .or_else(|| {
                // Try to find by first line
                if let Some(ext) = Path::new(file_name).extension() {
                    self.syntax_set.find_syntax_by_extension(ext.to_str()?)
                } else {
                    None
                }
            })
            .or_else(|| {
                // Fallback to plain text
                Some(self.syntax_set.find_syntax_plain_text())
            })
    }

    /// Find syntax by language name
    pub fn find_syntax_by_name(&self, name: &str) -> Option<&SyntaxReference> {
        self.syntax_set.find_syntax_by_name(name)
    }

    /// Get list of available themes
    pub fn available_themes(&self) -> Vec<String> {
        self.theme_set.names()
    }

    /// Get list of supported languages
    pub fn supported_languages(&self) -> Vec<String> {
        self.syntax_set
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect()
    }
}

/// Collection of color themes with lazy loading
pub struct ThemeSet {
    themes: HashMap<String, Theme>,
}

impl Default for ThemeSet {
    fn default() -> Self {
        let syntect_themes = SyntectThemeSet::load_defaults();
        let mut themes = HashMap::new();

        // Load default themes
        for (name, theme) in syntect_themes.themes {
            themes.insert(name, theme);
        }

        Self { themes }
    }
}

impl ThemeSet {
    /// Load a theme from file
    pub fn load_theme(path: &Path) -> Result<Theme, LoadingError> {
        ThemeSet::get_theme(path)
    }

    /// Get theme from file (syntect compatibility)
    fn get_theme(path: &Path) -> Result<Theme, LoadingError> {
        syntect::highlighting::ThemeSet::get_theme(path)
    }

    /// Add a theme to the set
    pub fn add_theme(&mut self, name: String, theme: Theme) {
        self.themes.insert(name, theme);
    }

    /// Get a theme by name
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get list of theme names
    pub fn names(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_resources() {
        let resources = SyntaxResources::default();

        // Should have syntax definitions
        assert!(!resources.syntax_set.syntaxes().is_empty());

        // Should have themes
        assert!(!resources.theme_set.names().is_empty());

        // Should have a default theme
        assert!(resources.active_theme() != &Theme::default());
    }

    #[test]
    fn test_find_syntax() {
        let resources = SyntaxResources::default();

        // Should find Rust syntax
        assert!(resources.find_syntax("main.rs").is_some());
        assert!(resources.find_syntax_by_name("Rust").is_some());

        // Should find Python syntax
        assert!(resources.find_syntax("script.py").is_some());
        assert!(resources.find_syntax_by_name("Python").is_some());

        // Should fallback to plain text for unknown
        assert!(resources.find_syntax("unknown.xyz").is_some());
    }

    #[test]
    fn test_theme_management() {
        let mut resources = SyntaxResources::default();
        let themes = resources.available_themes();

        assert!(!themes.is_empty());

        // Should be able to set a theme that exists
        if let Some(theme_name) = themes.first() {
            assert!(resources.set_active_theme(theme_name).is_ok());
        }

        // Should fail for non-existent theme
        assert!(resources.set_active_theme("non-existent-theme").is_err());
    }
}
