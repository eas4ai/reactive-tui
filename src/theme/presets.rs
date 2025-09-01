use super::{Theme, ThemeVariables};

/// Dark theme preset - optimized for low-light environments
/// Uses CSS custom properties that integrate with utility classes
pub fn dark_theme() -> Theme {
    let variables = ThemeVariables::new()
        // Core brand colors
        .set("--color-primary", "#3b82f6") // Blue-500
        .set("--color-secondary", "#8b5cf6") // Violet-500
        .set("--color-accent", "#06b6d4") // Cyan-500
        // Background colors
        .set("--color-background", "#111827") // Gray-900
        .set("--color-surface", "#1f2937") // Gray-800
        .set("--color-foreground", "#f9fafb") // Gray-50
        // Text colors
        .set("--color-text-muted", "#9ca3af") // Gray-400
        // Border colors
        .set("--color-border", "#4b5563") // Gray-600
        // Semantic colors
        .set("--color-success", "#22c55e") // Green-500
        .set("--color-warning", "#f59e0b") // Amber-500
        .set("--color-error", "#ef4444") // Red-500
        .set("--color-info", "#3b82f6") // Blue-500
        // Spacing scale (in terminal cells)
        .set("--spacing-xs", "1")
        .set("--spacing-sm", "2")
        .set("--spacing-md", "4")
        .set("--spacing-lg", "6")
        .set("--spacing-xl", "8")
        .set("--spacing-2xl", "12")
        .clone();

    Theme::new("dark").with_variables(variables)
}

/// Light theme preset - optimized for bright environments
pub fn light_theme() -> Theme {
    let variables = ThemeVariables::new()
        // Core brand colors
        .set("--color-primary", "#2563eb") // Blue-600
        .set("--color-secondary", "#9333ea") // Purple-600
        .set("--color-accent", "#0891b2") // Cyan-600
        // Background colors
        .set("--color-background", "#ffffff") // White
        .set("--color-surface", "#f9fafb") // Gray-50
        .set("--color-foreground", "#111827") // Gray-900
        // Text colors
        .set("--color-text-muted", "#6b7280") // Gray-500
        // Border colors
        .set("--color-border", "#e5e7eb") // Gray-200
        // Semantic colors
        .set("--color-success", "#16a34a") // Green-600
        .set("--color-warning", "#d97706") // Amber-600
        .set("--color-error", "#dc2626") // Red-600
        .set("--color-info", "#2563eb") // Blue-600
        // Spacing scale (in terminal cells)
        .set("--spacing-xs", "1")
        .set("--spacing-sm", "2")
        .set("--spacing-md", "4")
        .set("--spacing-lg", "6")
        .set("--spacing-xl", "8")
        .set("--spacing-2xl", "12")
        .clone();

    Theme::new("light").with_variables(variables)
}

/// High contrast theme - optimized for accessibility
pub fn high_contrast_theme() -> Theme {
    let variables = ThemeVariables::new()
        // High contrast brand colors
        .set("--color-primary", "#007fff") // Bright blue
        .set("--color-secondary", "#ff00ff") // Bright magenta
        .set("--color-accent", "#00ffff") // Bright cyan
        // Maximum contrast backgrounds
        .set("--color-background", "#000000") // Pure black
        .set("--color-surface", "#141414") // Near black
        .set("--color-foreground", "#ffffff") // Pure white
        // High contrast text
        .set("--color-text-muted", "#c8c8c8") // Light gray
        // High visibility borders
        .set("--color-border", "#ffffff") // White borders
        // High contrast semantic colors
        .set("--color-success", "#00ff00") // Bright green
        .set("--color-warning", "#ffff00") // Bright yellow
        .set("--color-error", "#ff0000") // Bright red
        .set("--color-info", "#00ffff") // Bright cyan
        // Spacing scale (same as other themes)
        .set("--spacing-xs", "1")
        .set("--spacing-sm", "2")
        .set("--spacing-md", "4")
        .set("--spacing-lg", "6")
        .set("--spacing-xl", "8")
        .set("--spacing-2xl", "12")
        .clone();

    Theme::new("high_contrast").with_variables(variables)
}

/// Solarized Dark theme - classic developer theme
pub fn solarized_dark_theme() -> Theme {
    let variables = ThemeVariables::new()
        // Solarized brand colors
        .set("--color-primary", "#268bd2") // Solarized blue
        .set("--color-secondary", "#6c71c4") // Solarized violet
        .set("--color-accent", "#2aa198") // Solarized cyan
        // Solarized backgrounds
        .set("--color-background", "#002b36") // Base03
        .set("--color-surface", "#073642") // Base02
        .set("--color-foreground", "#fdf6e3") // Base3
        // Solarized text
        .set("--color-text-muted", "#93a1a1") // Base1
        // Solarized borders
        .set("--color-border", "#657b83") // Base00
        // Solarized semantic colors
        .set("--color-success", "#859900") // Green
        .set("--color-warning", "#b58900") // Yellow
        .set("--color-error", "#d30102") // Red
        .set("--color-info", "#2aa198") // Cyan
        // Spacing scale
        .set("--spacing-xs", "1")
        .set("--spacing-sm", "2")
        .set("--spacing-md", "4")
        .set("--spacing-lg", "6")
        .set("--spacing-xl", "8")
        .set("--spacing-2xl", "12")
        .clone();

    Theme::new("solarized_dark").with_variables(variables)
}

/// Gruvbox Dark theme - warm retro developer theme
pub fn gruvbox_dark_theme() -> Theme {
    let variables = ThemeVariables::new()
        // Gruvbox brand colors
        .set("--color-primary", "#83a598") // Gruvbox blue
        .set("--color-secondary", "#d3869b") // Gruvbox purple
        .set("--color-accent", "#8ec07c") // Gruvbox aqua
        // Gruvbox backgrounds
        .set("--color-background", "#282828") // bg0
        .set("--color-surface", "#3c3836") // bg1
        .set("--color-foreground", "#fbf1c7") // fg0
        // Gruvbox text
        .set("--color-text-muted", "#bdae93") // fg3
        // Gruvbox borders
        .set("--color-border", "#7c6f64") // bg4
        // Gruvbox semantic colors
        .set("--color-success", "#b8bb26") // Green
        .set("--color-warning", "#fabd2f") // Yellow
        .set("--color-error", "#fb4934") // Red
        .set("--color-info", "#83a598") // Blue
        // Spacing scale
        .set("--spacing-xs", "1")
        .set("--spacing-sm", "2")
        .set("--spacing-md", "4")
        .set("--spacing-lg", "6")
        .set("--spacing-xl", "8")
        .set("--spacing-2xl", "12")
        .clone();

    Theme::new("gruvbox_dark").with_variables(variables)
}
