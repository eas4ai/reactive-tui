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
    fn default() -> Self {
        Self {
            base_classes: "bg-gray-800 text-white".to_string(),
            selected_classes: "bg-blue-600 text-white font-bold".to_string(),
            focused_classes: "bg-cyan-500 text-black font-bold".to_string(),
            disabled_classes: "text-gray-500".to_string(),
            separator_classes: "text-gray-400".to_string(),
            shortcut_classes: "text-yellow-400".to_string(),
            icon_classes: "text-green-400".to_string(),
            border_classes: "border border-gray-600".to_string(),
            show_shadow: true,
            show_icons: true,
            show_shortcuts: true,
            min_width: 10,
            max_width: Some(50),
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

/// Predefined menu themes using CSS utility classes
#[derive(Clone, Debug, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "Preserve the public Custom(MenuStyle) constructor without adding allocation"
)]
pub enum MenuTheme {
    /// Default theme
    Default,
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// High contrast theme
    HighContrast,
    /// Custom theme with specific CSS classes
    Custom(MenuStyle),
}

impl MenuTheme {
    /// Convert theme to menu style
    pub fn to_style(&self) -> MenuStyle {
        match self {
            MenuTheme::Default => MenuStyle::default(),
            MenuTheme::Dark => MenuStyle {
                base_classes: "bg-gray-900 text-gray-100".to_string(),
                selected_classes: "bg-blue-600 text-white font-bold".to_string(),
                focused_classes: "bg-cyan-400 text-black font-bold".to_string(),
                disabled_classes: "text-gray-600".to_string(),
                separator_classes: "text-gray-500".to_string(),
                shortcut_classes: "text-yellow-300".to_string(),
                icon_classes: "text-green-400".to_string(),
                border_classes: "border border-gray-700".to_string(),
                show_shadow: true,
                show_icons: true,
                show_shortcuts: true,
                min_width: 10,
                max_width: Some(50),
                padding: 1,
            },
            MenuTheme::Light => MenuStyle {
                base_classes: "bg-white text-black".to_string(),
                selected_classes: "bg-blue-500 text-white font-bold".to_string(),
                focused_classes: "bg-blue-200 text-black font-bold".to_string(),
                disabled_classes: "text-gray-400".to_string(),
                separator_classes: "text-gray-600".to_string(),
                shortcut_classes: "text-blue-600".to_string(),
                icon_classes: "text-green-600".to_string(),
                border_classes: "border border-gray-300".to_string(),
                show_shadow: true,
                show_icons: true,
                show_shortcuts: true,
                min_width: 10,
                max_width: Some(50),
                padding: 1,
            },
            MenuTheme::HighContrast => MenuStyle {
                base_classes: "bg-black text-white".to_string(),
                selected_classes: "bg-white text-black font-bold".to_string(),
                focused_classes: "bg-yellow-400 text-black font-bold".to_string(),
                disabled_classes: "text-gray-600".to_string(),
                separator_classes: "text-white".to_string(),
                shortcut_classes: "text-yellow-300".to_string(),
                icon_classes: "text-white".to_string(),
                border_classes: "border border-white".to_string(),
                show_shadow: true,
                show_icons: true,
                show_shortcuts: true,
                min_width: 10,
                max_width: Some(50),
                padding: 1,
            },
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
        assert_eq!(style.base_classes, "bg-gray-800 text-white");
        assert_eq!(style.padding, 1);
        assert_eq!(style.selected_classes, "bg-blue-600 text-white font-bold");
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
        let dark_style = MenuTheme::Dark.to_style();
        assert!(dark_style.base_classes.contains("bg-gray-900"));
        assert!(dark_style.base_classes.contains("text-gray-100"));

        let light_style = MenuTheme::Light.to_style();
        assert!(light_style.base_classes.contains("bg-white"));
        assert!(light_style.base_classes.contains("text-black"));

        let high_contrast_style = MenuTheme::HighContrast.to_style();
        assert!(high_contrast_style.base_classes.contains("bg-black"));
        assert!(high_contrast_style.selected_classes.contains("bg-white"));
    }

    #[test]
    fn test_apply_styles() {
        let style = MenuStyle::default();

        // Test that styles can be applied (without theme)
        let base_builder = style.apply_base_style(None);
        let selected_builder = style.apply_selected_style(None);
        let focused_builder = style.apply_focused_style(None);

        // The base style carries the menu padding; selected and focused rows are bold
        assert!(
            base_builder.bg_rgba.is_some(),
            "base classes set a background"
        );
        assert_eq!(selected_builder.text.bold, Some(true));
        assert_eq!(focused_builder.text.bold, Some(true));
        let padding = taffy::style::LengthPercentage::length(f32::from(style.padding));
        let base_style = base_builder.build();
        assert_eq!(base_style.padding.left, padding);
        assert_eq!(base_style.padding.top, padding);
        let _selected_style = selected_builder.build();
        let _focused_style = focused_builder.build();
    }
}
