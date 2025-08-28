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
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.variables.get(key).cloned()
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Define color variables
    pub fn colors(self) -> ColorBuilder {
        ColorBuilder { theme: self }
    }

    /// Define spacing variables
    pub fn spacing(self) -> SpacingBuilder {
        SpacingBuilder { theme: self }
    }

    /// Define typography variables
    pub fn typography(self) -> TypographyBuilder {
        TypographyBuilder { theme: self }
    }

    /// Define border variables
    pub fn borders(self) -> BorderBuilder {
        BorderBuilder { theme: self }
    }

    /// Define shadow variables for depth
    pub fn shadows(self) -> ShadowBuilder {
        ShadowBuilder { theme: self }
    }
}

pub struct ColorBuilder {
    theme: ThemeVariables,
}

impl ColorBuilder {
    /// Primary brand color (RGB values)
    pub fn primary(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-primary", rgb_to_hex(r, g, b));
        self.theme.set("--color-primary-dark", darken_rgb(r, g, b));
        self.theme
            .set("--color-primary-light", lighten_rgb(r, g, b));
        self
    }

    /// Secondary accent color
    pub fn secondary(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-secondary", rgb_to_hex(r, g, b));
        self.theme
            .set("--color-secondary-dark", darken_rgb(r, g, b));
        self.theme
            .set("--color-secondary-light", lighten_rgb(r, g, b));
        self
    }

    /// Background colors
    pub fn background(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-background", rgb_to_hex(r, g, b));
        self
    }

    pub fn background_secondary(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme
            .set("--color-background-secondary", rgb_to_hex(r, g, b));
        self
    }

    pub fn background_tertiary(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme
            .set("--color-background-tertiary", rgb_to_hex(r, g, b));
        self
    }

    /// Text colors
    pub fn text(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-text", rgb_to_hex(r, g, b));
        self
    }

    pub fn text_secondary(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme
            .set("--color-text-secondary", rgb_to_hex(r, g, b));
        self
    }

    pub fn text_muted(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-text-muted", rgb_to_hex(r, g, b));
        self
    }

    /// Border colors
    pub fn border(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-border", rgb_to_hex(r, g, b));
        self
    }

    pub fn border_focus(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-border-focus", rgb_to_hex(r, g, b));
        self
    }

    /// Semantic colors
    pub fn success(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-success", rgb_to_hex(r, g, b));
        self
    }

    pub fn warning(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-warning", rgb_to_hex(r, g, b));
        self
    }

    pub fn error(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-error", rgb_to_hex(r, g, b));
        self
    }

    pub fn info(mut self, r: u8, g: u8, b: u8) -> Self {
        self.theme.set("--color-info", rgb_to_hex(r, g, b));
        self
    }

    pub fn build(self) -> ThemeVariables {
        self.theme
    }
}

pub struct SpacingBuilder {
    theme: ThemeVariables,
}

impl SpacingBuilder {
    pub fn xs(mut self, value: u16) -> Self {
        self.theme.set("--spacing-xs", value.to_string());
        self
    }

    pub fn sm(mut self, value: u16) -> Self {
        self.theme.set("--spacing-sm", value.to_string());
        self
    }

    pub fn md(mut self, value: u16) -> Self {
        self.theme.set("--spacing-md", value.to_string());
        self
    }

    pub fn lg(mut self, value: u16) -> Self {
        self.theme.set("--spacing-lg", value.to_string());
        self
    }

    pub fn xl(mut self, value: u16) -> Self {
        self.theme.set("--spacing-xl", value.to_string());
        self
    }

    pub fn xxl(mut self, value: u16) -> Self {
        self.theme.set("--spacing-xxl", value.to_string());
        self
    }

    pub fn build(self) -> ThemeVariables {
        self.theme
    }
}

pub struct TypographyBuilder {
    theme: ThemeVariables,
}

impl TypographyBuilder {
    pub fn size_xs(mut self, value: u16) -> Self {
        self.theme.set("--font-size-xs", value.to_string());
        self
    }

    pub fn size_sm(mut self, value: u16) -> Self {
        self.theme.set("--font-size-sm", value.to_string());
        self
    }

    pub fn size_md(mut self, value: u16) -> Self {
        self.theme.set("--font-size-md", value.to_string());
        self
    }

    pub fn size_lg(mut self, value: u16) -> Self {
        self.theme.set("--font-size-lg", value.to_string());
        self
    }

    pub fn size_xl(mut self, value: u16) -> Self {
        self.theme.set("--font-size-xl", value.to_string());
        self
    }

    pub fn weight_normal(mut self) -> Self {
        self.theme.set("--font-weight-normal", "normal");
        self
    }

    pub fn weight_bold(mut self) -> Self {
        self.theme.set("--font-weight-bold", "bold");
        self
    }

    pub fn build(self) -> ThemeVariables {
        self.theme
    }
}

pub struct BorderBuilder {
    theme: ThemeVariables,
}

impl BorderBuilder {
    pub fn width(mut self, value: u16) -> Self {
        self.theme.set("--border-width", value.to_string());
        self
    }

    pub fn radius_sm(mut self, value: u16) -> Self {
        self.theme.set("--border-radius-sm", value.to_string());
        self
    }

    pub fn radius_md(mut self, value: u16) -> Self {
        self.theme.set("--border-radius-md", value.to_string());
        self
    }

    pub fn radius_lg(mut self, value: u16) -> Self {
        self.theme.set("--border-radius-lg", value.to_string());
        self
    }

    pub fn style(mut self, style: &str) -> Self {
        self.theme.set("--border-style", style);
        self
    }

    pub fn build(self) -> ThemeVariables {
        self.theme
    }
}

pub struct ShadowBuilder {
    theme: ThemeVariables,
}

impl ShadowBuilder {
    pub fn none(mut self) -> Self {
        self.theme.set("--shadow-none", "none");
        self
    }

    pub fn sm(mut self) -> Self {
        self.theme.set("--shadow-sm", "subtle");
        self
    }

    pub fn md(mut self) -> Self {
        self.theme.set("--shadow-md", "medium");
        self
    }

    pub fn lg(mut self) -> Self {
        self.theme.set("--shadow-lg", "strong");
        self
    }

    pub fn xl(mut self) -> Self {
        self.theme.set("--shadow-xl", "heavy");
        self
    }

    pub fn build(self) -> ThemeVariables {
        self.theme
    }
}

fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn darken_rgb(r: u8, g: u8, b: u8) -> String {
    let r = (r as f32 * 0.7) as u8;
    let g = (g as f32 * 0.7) as u8;
    let b = (b as f32 * 0.7) as u8;
    rgb_to_hex(r, g, b)
}

fn lighten_rgb(r: u8, g: u8, b: u8) -> String {
    let r = ((r as f32 * 1.3).min(255.0)) as u8;
    let g = ((g as f32 * 1.3).min(255.0)) as u8;
    let b = ((b as f32 * 1.3).min(255.0)) as u8;
    rgb_to_hex(r, g, b)
}
