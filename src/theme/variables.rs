use std::collections::HashMap;

/// CSS variable definitions for theming
///
/// Maps CSS custom properties to their values, following standard naming conventions:
/// - Colors: --color-{name} (e.g., --color-primary, --color-background)
/// - Spacing: --spacing-{size} (e.g., --spacing-sm, --spacing-lg)
/// - Typography: --font-{property} (e.g., --font-size-sm, --font-weight-bold)
/// - Borders: --border-{property} (e.g., --border-width, --border-radius)
/// - Shadows: --shadow-{size} (e.g., --shadow-sm, --shadow-lg)
#[derive(Debug, Clone, Default)]
pub struct ThemeVariables {
    variables: HashMap<String, String>,
}

impl ThemeVariables {
    /// Create a new theme variables collection
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Get a theme variable value by key
    pub fn get(&self, key: &str) -> Option<String> {
        self.variables.get(key).cloned()
    }

    /// Set a CSS custom property
    pub fn set(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Insert all variables from another ThemeVariables instance
    pub fn extend(mut self, other: ThemeVariables) -> Self {
        self.variables.extend(other.variables);
        self
    }

    /// Get all variables as a HashMap
    pub fn all(&self) -> &HashMap<String, String> {
        &self.variables
    }
}
