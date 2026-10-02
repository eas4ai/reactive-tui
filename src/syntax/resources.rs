//! Global syntax resources manager
//!
//! Lazy-loaded Lumis themes with support for custom themes. Grammars are
//! compiled in through Lumis `lang-*` Cargo features (see the root
//! `Cargo.toml`); there is no runtime syntax-definition loading.

use lumis::languages::Language;
use lumis::themes::{self, Theme};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

/// Default active theme (a dark Neovim theme bundled with Lumis).
pub const DEFAULT_THEME: &str = "onedark";

/// Global syntax resources instance
pub static SYNTAX_RESOURCES: Lazy<RwLock<SyntaxResources>> =
    Lazy::new(|| RwLock::new(SyntaxResources::default()));

/// Container for the active theme and custom themes
pub struct SyntaxResources {
    /// Currently active theme name
    active_theme: String,
    /// Caller-supplied themes shadowing the built-ins
    theme_set: ThemeSet,
}

impl Default for SyntaxResources {
    fn default() -> Self {
        Self {
            active_theme: String::from(DEFAULT_THEME),
            theme_set: ThemeSet::default(),
        }
    }
}

impl SyntaxResources {
    /// Load a custom theme from a JSON file (Lumis theme format).
    ///
    /// This replaces the old `.tmTheme`/plist loader: Sublime Text theme
    /// files are no longer accepted.
    pub fn load_theme_from_file(&mut self, name: String, path: &Path) -> Result<(), String> {
        let theme =
            themes::from_file(path).map_err(|error| format!("cannot load theme: {error}"))?;
        self.theme_set.add_theme(name, theme);
        Ok(())
    }

    /// Set the active theme
    pub fn set_active_theme(&mut self, name: &str) -> Result<(), String> {
        if self.theme_set.get(name).is_some() || themes::get(name).is_ok() {
            self.active_theme = name.to_string();
            Ok(())
        } else {
            Err(format!("Theme '{name}' not found"))
        }
    }

    /// Look up a theme by name: custom themes first, then Lumis built-ins.
    pub fn theme_by_name(&self, name: &str) -> Option<Theme> {
        if let Some(theme) = self.theme_set.get(name) {
            return Some(theme.clone());
        }
        themes::get(name).ok()
    }

    /// Get the active theme.
    ///
    /// Falls back to the default theme when the configured name stops
    /// resolving, and to `None` only when no theme resolves at all.
    pub fn active_theme(&self) -> Option<Theme> {
        if let Some(theme) = self.theme_by_name(&self.active_theme) {
            return Some(theme);
        }
        if self.active_theme != DEFAULT_THEME {
            return self.theme_by_name(DEFAULT_THEME);
        }
        None
    }

    /// Find the compiled-in language for a file name.
    ///
    /// Matches against each language's glob patterns (`*.rs`, `PKGBUILD`,
    /// ...). Unknown files fall back to plain text, mirroring the old
    /// syntect behavior.
    pub fn find_language_for_file(&self, file_name: &str) -> Language {
        let extension = Path::new(file_name)
            .extension()
            .and_then(|extension| extension.to_str());
        for language in Language::iter() {
            for glob in language.globs() {
                if Self::glob_matches(glob, file_name, extension) {
                    return language;
                }
            }
        }
        Language::PlainText
    }

    /// Find a compiled-in language by display name (`Rust`, `Python`).
    ///
    /// Matching is exact and case-sensitive, mirroring the old syntect
    /// lookup: an unknown name yields `None` so callers fall back to
    /// plain rendering instead of guessing.
    pub fn find_language_by_name(&self, name: &str) -> Option<Language> {
        Language::iter().find(|language| language.name() == name)
    }

    /// Get list of available theme names (custom themes first).
    pub fn available_themes(&self) -> Vec<String> {
        let mut names = self.theme_set.names();
        for theme in themes::available_themes() {
            if !names.contains(&theme.name) {
                names.push(theme.name.clone());
            }
        }
        names
    }

    /// Get list of supported language names (compiled-in grammars).
    pub fn supported_languages(&self) -> Vec<String> {
        Language::iter()
            .map(|language| language.name().to_string())
            .collect()
    }

    /// Match a Lumis glob pattern against a file name.
    fn glob_matches(glob: &str, file_name: &str, extension: Option<&str>) -> bool {
        if let Some(pattern) = glob.strip_prefix("*.") {
            // Extension glob: compare case-insensitively, with or without
            // a directory prefix on the file name.
            return extension.is_some_and(|extension| extension.eq_ignore_ascii_case(pattern))
                || file_name
                    .rsplit('/')
                    .next()
                    .is_some_and(|base| base.eq_ignore_ascii_case(pattern));
        }
        if let Some(pattern) = glob.strip_prefix('*') {
            return file_name
                .rsplit('/')
                .next()
                .is_some_and(|base| base.ends_with(pattern));
        }
        // Exact file name (e.g. `PKGBUILD`, `Dockerfile`).
        file_name
            .rsplit('/')
            .next()
            .is_some_and(|base| base == glob)
    }
}

/// Collection of caller-supplied themes shadowing the Lumis built-ins
#[derive(Default)]
pub struct ThemeSet {
    themes: HashMap<String, Theme>,
}

impl ThemeSet {
    /// Add a theme to the set
    pub fn add_theme(&mut self, name: String, theme: Theme) {
        self.themes.insert(name, theme);
    }

    /// Get a theme by name
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }

    /// Get list of custom theme names
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

        // Should have compiled-in grammars
        assert!(!resources.supported_languages().is_empty());

        // Should have themes (built-ins at minimum)
        assert!(!resources.available_themes().is_empty());

        // The default theme must resolve
        assert!(resources.active_theme().is_some());
    }

    #[test]
    fn test_find_language() {
        let resources = SyntaxResources::default();

        // Should find Rust by file and by name
        assert_eq!(
            resources.find_language_for_file("main.rs").name(),
            Language::Rust.name()
        );
        assert!(resources.find_language_by_name("Rust").is_some());

        // Should find Python by file and by name
        assert!(resources.find_language_by_name("Python").is_some());
        assert_ne!(
            resources.find_language_for_file("script.py"),
            Language::PlainText
        );

        // Unknown files fall back to plain text
        assert_eq!(
            resources.find_language_for_file("unknown.xyz"),
            Language::PlainText
        );
    }

    #[test]
    fn test_theme_management() {
        let mut resources = SyntaxResources::default();
        let themes = resources.available_themes();

        assert!(!themes.is_empty());

        // Should be able to set a theme that exists
        if let Some(theme_name) = themes.first().cloned() {
            assert!(resources.set_active_theme(&theme_name).is_ok());
        }

        // Should fail for non-existent theme
        assert!(resources.set_active_theme("non-existent-theme").is_err());
    }
}
