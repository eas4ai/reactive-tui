//! The separator builder (DIS-001 to DIS-004): a line that fills its parent's width,
//! or its height when vertical, with an optional label.

use crate::component::Element;
use crate::widgets::display::pieces::separator::{Separator, SeparatorProps, SeparatorStyle};

/// Create a separator builder: a horizontal line that fills its parent's width.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::widgets::pieces::separator::separator;
///
/// let line = separator().label("Results").build();
/// ```
pub fn separator() -> SeparatorBuilder {
    SeparatorBuilder {
        props: SeparatorProps::default(),
    }
}

/// Builder for a separator. The element it builds is the one the separator's
/// props make.
#[derive(Clone, Debug, Default)]
pub struct SeparatorBuilder {
    props: SeparatorProps,
}

impl SeparatorBuilder {
    /// Set the line the separator draws.
    pub fn style(mut self, style: SeparatorStyle) -> Self {
        self.props.style = style;
        self
    }

    /// Run the separator down the height instead of across the width.
    pub fn vertical(mut self) -> Self {
        self.props.vertical = true;
        self
    }

    /// Set the label drawn in the middle of a horizontal separator.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.props.label = Some(label.into());
        self
    }

    /// Add classes to the separator. A `w-`, `h-` class sets its size.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Set the name the screen reader speaks, in place of the label.
    pub fn aria_label(mut self, label: impl Into<String>) -> Self {
        self.props.aria_label = Some(label.into());
        self
    }

    /// Build the separator's element.
    pub fn build(self) -> Element {
        Element::typed::<Separator>(self.props)
    }
}
