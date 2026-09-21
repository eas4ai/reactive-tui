//! Display widget builders
//!
//! This module provides builders for display components including progress bars,
//! tree views, images, and other visual display elements.

use crate::builder::specialized::{ImageBuilder, TreeBuilder};
use crate::component::Element;

/// Create a Progress Bar builder
///
/// Returns a `ProgressBarBuilder` for creating progress bars with customizable
/// values, labels, colors, and animations.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::progress_bar;
///
/// let progress = progress_bar()
///     .value(75.0)
///     .label("Loading")
///     .build();
/// ```
pub fn progress_bar() -> ProgressBarBuilder {
    ProgressBarBuilder::new()
}

// /// Create a Tree view builder
// ///
// /// Returns a `TreeBuilder` for creating hierarchical tree view components.
// pub fn tree() -> TreeBuilder {
//     TreeBuilder::new()
// }

// /// Create an Image display builder
// ///
// /// Returns an `ImageBuilder` for creating image display components.
// pub fn image() -> ImageBuilder {
//     ImageBuilder::new()
// }

/// Create a Tree builder
///
/// Returns a `TreeBuilder` for creating hierarchical tree view components
/// with expandable/collapsible nodes and custom rendering.
pub fn tree() -> TreeBuilder {
    TreeBuilder::new()
}

/// Create an Image builder
///
/// Returns an `ImageBuilder` for creating image display components
/// with support for various formats and terminal rendering.
pub fn image() -> ImageBuilder {
    ImageBuilder::new()
}

/// Create a Popover builder
///
/// Returns a `PopoverBuilder` for creating popover elements that display
/// content when triggered by user interaction.
pub fn popover() -> PopoverBuilder {
    PopoverBuilder::new()
}

/// Builder for ProgressBar components
///
/// Provides a fluent API for creating progress bars with customizable
/// values, labels, colors, and animations.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::progress_bar;
///
/// let progress = progress_bar()
///     .value(75.0)
///     .max_value(100.0)
///     .label("Loading")
///     .animated(true)
///     .color("#3b82f6")
///     .build();
/// ```
pub struct ProgressBarBuilder {
    value: f64,
    max_value: f64,
    label: Option<String>,
    show_percentage: bool,
    animated: bool,
    color: Option<String>,
    width: Option<u16>,
    class: Option<String>,
}

impl ProgressBarBuilder {
    /// Create a new ProgressBarBuilder with default values
    fn new() -> Self {
        Self {
            value: 0.0,
            max_value: 100.0,
            label: None,
            show_percentage: true,
            animated: false,
            color: None,
            width: None,
            class: None,
        }
    }

    /// Set the current progress value
    ///
    /// # Arguments
    /// * `value` - The current progress value (typically 0.0 to max_value)
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Set the maximum progress value
    ///
    /// # Arguments
    /// * `max_value` - The maximum value representing 100% completion
    pub fn max_value(mut self, max_value: f64) -> Self {
        self.max_value = max_value;
        self
    }

    /// Set the progress bar label
    ///
    /// # Arguments
    /// * `label` - Text label to display with the progress bar
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Configure whether to show percentage text
    ///
    /// # Arguments
    /// * `show` - Whether to display the percentage value
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_percentage = show;
        self
    }

    /// Enable or disable progress bar animation
    ///
    /// # Arguments
    /// * `animated` - Whether the progress bar should animate
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }

    /// Set the progress bar color
    ///
    /// # Arguments
    /// * `color` - CSS color value (hex, rgb, named color, etc.)
    pub fn color(mut self, color: &str) -> Self {
        self.color = Some(color.to_string());
        self
    }

    /// Set the progress bar width
    ///
    /// # Arguments
    /// * `width` - Width in characters
    pub fn width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the progress bar
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the ProgressBar element
    ///
    /// Creates a progress bar element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the progress bar
    pub fn build(self) -> Element {
        let props = crate::widgets::display::ProgressBarProps {
            value: self.value,
            max_value: self.max_value,
            label: self.label,
            show_percentage: self.show_percentage,
            animated: self.animated,
            color: self
                .color
                .or_else(|| crate::widgets::display::ProgressBarProps::default().color),
            width: self.width,
            style: self.class,
            ..Default::default()
        };
        Element::component_with_props("ProgressBar", props)
    }
}

impl From<ProgressBarBuilder> for Element {
    fn from(builder: ProgressBarBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Popover components
///
/// This builder provides a fluent API for creating popover elements that display
/// content when triggered by user interaction.
///
/// # Example
/// ```rust,ignore
/// use reactive_tui::builder::popover;
/// use reactive_tui::component::Element;
///
/// let popover = popover()
///     .content(Element::text("Helpful information"))
///     .trigger(Element::text("Help"))
///     .build();
/// ```
pub struct PopoverBuilder {
    content: Vec<Element>,
    trigger: Option<Element>,
    class: Option<String>,
}

impl Default for PopoverBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PopoverBuilder {
    /// Create a new PopoverBuilder with default values
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            trigger: None,
            class: None,
        }
    }

    /// Add content to the popover
    ///
    /// # Arguments
    /// * `element` - Element to display in the popover content area
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Set the trigger element for the popover
    ///
    /// # Arguments
    /// * `element` - Element that triggers the popover when interacted with
    pub fn trigger(mut self, element: Element) -> Self {
        self.trigger = Some(element);
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the popover
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Popover element
    ///
    /// Creates a popover element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the popover
    pub fn build(self) -> Element {
        let props = crate::widgets::display::popover::PopoverProps {
            trigger_element: self.trigger.unwrap_or_else(Element::empty),
            content: Element::layout(crate::component::LayoutType::Flex)
                .with_children(self.content),
            ..Default::default()
        };
        let mut element = Element::component_with_props("Popover", props);
        element.class = self.class;
        element
    }
}

impl From<PopoverBuilder> for Element {
    fn from(builder: PopoverBuilder) -> Self {
        builder.build()
    }
}

// Note: Placeholder builders (TreeBuilder, ImageBuilder)
// are now provided by the placeholders module
