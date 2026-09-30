use crate::layout::css::apply_utility_classes_with_theme;
use crate::layout::style::StyleBuilder;
use crate::theme::Theme;

/// Menu styling configuration using CSS utility classes and themes
///
/// This follows the reactive-tui architecture by leveraging the CSS utility system
/// and theme variables instead of custom styling structures.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuStyle {
    /// CSS classes for base menu items
    pub base_classes: String,
    /// CSS classes for selected items
    pub selected_classes: String,
    /// CSS classes for focused items
    pub focused_classes: String,
    /// CSS classes for disabled items
    pub disabled_classes: String,
    /// CSS classes for separators
    pub separator_classes: String,
    /// CSS classes for shortcuts
    pub shortcut_classes: String,
    /// CSS classes for icons
    pub icon_classes: String,
    /// CSS classes for menu borders
    pub border_classes: String,
    /// CSS classes for the shadow under a panel
    pub shadow_classes: String,
    /// CSS classes for the veil behind a dialog menu
    pub veil_classes: String,
    /// Whether to show shadows for popup menus
    pub show_shadow: bool,
    /// Whether to show icons
    pub show_icons: bool,
    /// Whether to show shortcuts
    pub show_shortcuts: bool,
    /// Minimum width for menu items
    pub min_width: u16,
    /// Maximum width for menu items
    pub max_width: Option<u16>,
    /// Padding inside menu containers
    pub padding: u16,
}

impl Default for MenuStyle {
    /// Every color is a role of the active theme (MNU-001): the panel is
    /// the theme's surface, the current row its selection while the menu
    /// holds the focus and its hover color while it does not.
    fn default() -> Self {
        Self {
            base_classes: "bg-surface text-foreground".to_string(),
            selected_classes: "bg-hover text-foreground".to_string(),
            focused_classes: "bg-selection text-selection-foreground font-bold".to_string(),
            disabled_classes: "text-muted".to_string(),
            separator_classes: "text-muted".to_string(),
            shortcut_classes: "text-muted".to_string(),
            icon_classes: String::new(),
            border_classes: "border border-border".to_string(),
            shadow_classes: "bg-shadow".to_string(),
            veil_classes: "bg-overlay".to_string(),
            show_shadow: true,
            show_icons: true,
            show_shortcuts: true,
            min_width: 10,
            max_width: None,
            padding: 1,
        }
    }
}

impl MenuStyle {
    /// Create a new menu style with custom CSS classes
    pub fn new() -> Self {
        Self::default()
    }

    /// Set base menu item classes
    pub fn base_classes(mut self, classes: impl Into<String>) -> Self {
        self.base_classes = classes.into();
        self
    }

    /// Set selected item classes
    pub fn selected_classes(mut self, classes: impl Into<String>) -> Self {
        self.selected_classes = classes.into();
        self
    }

    /// Set focused item classes
    pub fn focused_classes(mut self, classes: impl Into<String>) -> Self {
        self.focused_classes = classes.into();
        self
    }

    /// Set disabled item classes
    pub fn disabled_classes(mut self, classes: impl Into<String>) -> Self {
        self.disabled_classes = classes.into();
        self
    }

    /// Set separator classes
    pub fn separator_classes(mut self, classes: impl Into<String>) -> Self {
        self.separator_classes = classes.into();
        self
    }

    /// Set shortcut classes
    pub fn shortcut_classes(mut self, classes: impl Into<String>) -> Self {
        self.shortcut_classes = classes.into();
        self
    }

    /// Set icon classes
    pub fn icon_classes(mut self, classes: impl Into<String>) -> Self {
        self.icon_classes = classes.into();
        self
    }

    /// Set border classes
    pub fn border_classes(mut self, classes: impl Into<String>) -> Self {
        self.border_classes = classes.into();
        self
    }

    /// Set the classes of the shadow under a panel
    pub fn shadow_classes(mut self, classes: impl Into<String>) -> Self {
        self.shadow_classes = classes.into();
        self
    }

    /// Set the classes of the veil behind a dialog menu
    pub fn veil_classes(mut self, classes: impl Into<String>) -> Self {
        self.veil_classes = classes.into();
        self
    }

    /// The default style with every color taken from `theme` instead of
    /// the active theme: the menu keeps these colors when the application
    /// changes its theme.
    pub fn of(theme: &Theme) -> Self {
        let color = |role: &str| theme.hex(role);
        Self {
            base_classes: format!("bg-{} text-{}", color("surface"), color("foreground")),
            selected_classes: format!("bg-{} text-{}", color("hover"), color("foreground")),
            focused_classes: format!(
                "bg-{} text-{} font-bold",
                color("selection"),
                color("selection-foreground")
            ),
            disabled_classes: format!("text-{}", color("text-muted")),
            separator_classes: format!("text-{}", color("text-muted")),
            shortcut_classes: format!("text-{}", color("text-muted")),
            border_classes: format!("border border-{}", color("border")),
            shadow_classes: format!("bg-{}", color("shadow")),
            veil_classes: format!("bg-{}", color("overlay")),
            ..Self::default()
        }
    }

    /// Set whether to show shadows
    pub fn show_shadow(mut self, show: bool) -> Self {
        self.show_shadow = show;
        self
    }

    /// Set whether to show icons
    pub fn show_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    /// Set whether to show shortcuts
    pub fn show_shortcuts(mut self, show: bool) -> Self {
        self.show_shortcuts = show;
        self
    }

    /// Set minimum width
    pub fn min_width(mut self, width: u16) -> Self {
        self.min_width = width;
        self
    }

    /// Set maximum width
    pub fn max_width(mut self, width: Option<u16>) -> Self {
        self.max_width = width;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: u16) -> Self {
        self.padding = padding;
        self
    }

    /// Apply base menu item styling with optional theme
    pub fn apply_base_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(
            &self.base_classes,
            StyleBuilder::new().padding_all_px(f32::from(self.padding)),
            theme,
        )
    }

    /// Apply selected item styling with optional theme
    pub fn apply_selected_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.selected_classes, StyleBuilder::new(), theme)
    }

    /// Apply focused item styling with optional theme
    pub fn apply_focused_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.focused_classes, StyleBuilder::new(), theme)
    }

    /// Apply disabled item styling with optional theme
    pub fn apply_disabled_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.disabled_classes, StyleBuilder::new(), theme)
    }

    /// Apply separator styling with optional theme
    pub fn apply_separator_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.separator_classes, StyleBuilder::new(), theme)
    }

    /// Apply shortcut styling with optional theme
    pub fn apply_shortcut_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.shortcut_classes, StyleBuilder::new(), theme)
    }

    /// Apply icon styling with optional theme
    pub fn apply_icon_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.icon_classes, StyleBuilder::new(), theme)
    }

    /// Apply border styling with optional theme
    pub fn apply_border_style(&self, theme: Option<&Theme>) -> StyleBuilder {
        apply_utility_classes_with_theme(&self.border_classes, StyleBuilder::new(), theme)
    }
}

/// The looks a menu can name. `Default` follows the application's theme;
/// `Dark`, `Light` and `HighContrast` keep the colors of the built-in
/// preset of that name under any theme.
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "Preserve the public Custom(MenuStyle) constructor without adding allocation"
)]
pub enum MenuTheme {
    /// The roles of the active theme
    Default,
    /// The roles of the built-in dark preset
    Dark,
    /// The roles of the built-in light preset
    Light,
    /// The roles of the built-in high contrast preset
    HighContrast,
    /// Custom theme with specific CSS classes
    Custom(MenuStyle),
}

impl MenuTheme {
    /// Convert theme to menu style
    pub fn to_style(&self) -> MenuStyle {
        match self {
            MenuTheme::Default => MenuStyle::default(),
            MenuTheme::Dark => MenuStyle::of(&crate::theme::dark_theme()),
            MenuTheme::Light => MenuStyle::of(&crate::theme::light_theme()),
            MenuTheme::HighContrast => MenuStyle::of(&crate::theme::high_contrast_theme()),
            MenuTheme::Custom(style) => style.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_style_default() {
        let style = MenuStyle::default();
        assert_eq!(style.base_classes, "bg-surface text-foreground");
        assert_eq!(style.padding, 1);
        assert_eq!(style.selected_classes, "bg-hover text-foreground");
        assert_eq!(style.max_width, None);
        assert!(style.show_shadow);
        assert!(style.show_icons);
        assert!(style.show_shortcuts);
    }

    #[test]
    fn test_menu_style_builder() {
        let style = MenuStyle::new()
            .base_classes("bg-red-500 text-white")
            .selected_classes("bg-red-700 text-yellow-200")
            .show_shadow(false)
            .min_width(20);

        assert_eq!(style.base_classes, "bg-red-500 text-white");
        assert_eq!(style.selected_classes, "bg-red-700 text-yellow-200");
        assert!(!style.show_shadow);
        assert_eq!(style.min_width, 20);
    }

    #[test]
    fn test_menu_theme_to_style() {
        use crate::theme::{dark_theme, high_contrast_theme, light_theme};
        for (look, preset) in [
            (MenuTheme::Dark, dark_theme()),
            (MenuTheme::Light, light_theme()),
            (MenuTheme::HighContrast, high_contrast_theme()),
        ] {
            let style = look.to_style();
            let surface = preset.get_variable("--color-surface").unwrap();
            let selection = preset.get_variable("--color-selection").unwrap();
            assert_eq!(
                style.base_classes.split_whitespace().next(),
                Some(format!("bg-{surface}").as_str()),
                "{look:?}"
            );
            assert!(
                style.focused_classes.contains(&format!("bg-{selection}")),
                "{look:?}: {}",
                style.focused_classes
            );
            assert_eq!(style.shadow_classes, "bg-#00000066", "{look:?}");
        }
        assert_eq!(MenuTheme::Default.to_style(), MenuStyle::default());
    }

    #[test]
    fn test_apply_styles() {
        let style = MenuStyle::default();

        // Test that styles can be applied (without theme)
        let base_builder = style.apply_base_style(None);
        let selected_builder = style.apply_selected_style(None);
        let focused_builder = style.apply_focused_style(None);

        // The base style carries the menu padding; the current row of a
        // menu that holds the focus is bold
        assert!(
            base_builder.bg_rgba.is_some(),
            "base classes set a background"
        );
        assert!(
            selected_builder.bg_rgba.is_some(),
            "the current row of a menu without the focus has a background"
        );
        assert_eq!(focused_builder.text.bold, Some(true));
        let padding = taffy::style::LengthPercentage::length(f32::from(style.padding));
        let base_style = base_builder.build();
        assert_eq!(base_style.padding.left, padding);
        assert_eq!(base_style.padding.top, padding);
        let _selected_style = selected_builder.build();
        let _focused_style = focused_builder.build();
    }
}
