//! Accordion widget builder
//!
//! Provides fluent API for creating accordion widgets with sections,
//! animations, and accessibility features.

use crate::component::Element;
use crate::widgets::layout::{AccordionBuilder, AccordionMode, AccordionSection};

/// Create an accordion widget builder
///
/// Returns an `AccordionBuilder` for creating accordion widgets with
/// expandable/collapsible sections, animations, and keyboard navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::accordion;
/// use reactive_tui::component::Element;
/// use reactive_tui::widgets::{AccordionMode, AccordionSection};
///
/// let accordion = accordion()
///     .section(AccordionSection::new("general", "General Settings")
///         .content(Element::text("General application settings..."))
///         .expanded(true))
///     .section(AccordionSection::new("privacy", "Privacy & Security")
///         .content(Element::text("Privacy and security options..."))
///         .icon("🔒"))
///     .mode(AccordionMode::Single)
///     .animated(true)
///     .build();
/// ```
pub fn accordion() -> AccordionBuilder {
    AccordionBuilder::new()
}

/// Create a simple accordion with text sections
///
/// A convenience function for creating accordions with simple text content.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::simple_accordion;
///
/// let accordion = simple_accordion(vec![
///     ("intro", "Introduction", "Welcome to our application!"),
///     ("features", "Features", "Here are the main features..."),
///     ("help", "Help", "Need help? Contact support..."),
/// ]);
/// ```
pub fn simple_accordion(sections: Vec<(&str, &str, &str)>) -> Element {
    let mut builder = AccordionBuilder::new();

    for (id, title, content) in sections {
        let section = AccordionSection::new(id, title).content(Element::text(content));
        builder = builder.section(section);
    }

    builder.build()
}

/// Create a settings-style accordion
///
/// Creates an accordion optimized for settings/configuration interfaces
/// with single-expand mode and smooth animations.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::settings_accordion;
/// use reactive_tui::component::Element;
/// use reactive_tui::widgets::AccordionSection;
///
/// let settings = settings_accordion()
///     .section(AccordionSection::new("general", "General")
///         .content(Element::text("General settings..."))
///         .icon("⚙️"))
///     .section(AccordionSection::new("privacy", "Privacy")
///         .content(Element::text("Privacy settings..."))
///         .icon("🔒"))
///     .build();
/// ```
pub fn settings_accordion() -> AccordionBuilder {
    AccordionBuilder::new()
        .mode(AccordionMode::Single)
        .animated(true)
        .keyboard_navigation(true)
        .class("settings-accordion")
}

/// Create a FAQ-style accordion
///
/// Creates an accordion optimized for FAQ interfaces with multiple
/// sections that can be expanded simultaneously.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::faq_accordion;
/// use reactive_tui::component::Element;
/// use reactive_tui::widgets::AccordionSection;
///
/// let faq = faq_accordion()
///     .section(AccordionSection::new("q1", "How do I get started?")
///         .content(Element::text("To get started, simply...")))
///     .section(AccordionSection::new("q2", "What are the requirements?")
///         .content(Element::text("The requirements are...")))
///     .build();
/// ```
pub fn faq_accordion() -> AccordionBuilder {
    AccordionBuilder::new()
        .mode(AccordionMode::Multiple)
        .animated(true)
        .icons("❓", "✅")
        .class("faq-accordion")
}

/// Create a navigation accordion
///
/// Creates an accordion optimized for navigation menus with always-one
/// mode to ensure a section is always visible.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::navigation_accordion;
/// use reactive_tui::component::Element;
/// use reactive_tui::widgets::AccordionSection;
///
/// let nav = navigation_accordion()
///     .section(AccordionSection::new("dashboard", "Dashboard")
///         .content(Element::text("Dashboard content..."))
///         .expanded(true))
///     .section(AccordionSection::new("reports", "Reports")
///         .content(Element::text("Reports content...")))
///     .build();
/// ```
pub fn navigation_accordion() -> AccordionBuilder {
    AccordionBuilder::new()
        .mode(AccordionMode::AlwaysOne)
        .animated(true)
        .icons("▶", "▼")
        .class("navigation-accordion")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::AccordionSection;

    #[test]
    fn test_accordion_builder() {
        let accordion = accordion()
            .section(
                AccordionSection::new("test", "Test Section")
                    .content(Element::text("Test content")),
            )
            .build();

        // Should create a component element
        assert!(accordion.is_component());
        assert_eq!(accordion.component_name(), Some("Accordion"));
    }

    #[test]
    fn test_simple_accordion() {
        let accordion = simple_accordion(vec![
            ("intro", "Introduction", "Welcome!"),
            ("help", "Help", "Need help?"),
        ]);

        assert!(accordion.is_component());
    }

    #[test]
    fn test_settings_accordion() {
        let settings = settings_accordion()
            .section(
                AccordionSection::new("general", "General").content(Element::text("Settings...")),
            )
            .build();

        assert!(settings.is_component());
    }

    #[test]
    fn test_faq_accordion() {
        let faq = faq_accordion()
            .section(AccordionSection::new("q1", "Question 1").content(Element::text("Answer 1")))
            .build();

        assert!(faq.is_component());
    }

    #[test]
    fn test_navigation_accordion() {
        let nav = navigation_accordion()
            .section(
                AccordionSection::new("dashboard", "Dashboard")
                    .content(Element::text("Dashboard"))
                    .expanded(true),
            )
            .build();

        assert!(nav.is_component());
    }
}
