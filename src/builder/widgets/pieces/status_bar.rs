//! The status bar builder (DIS-001 to DIS-004): one row of left, center and right text
//! on `surface`, cut to fit its width.

use crate::component::Element;
use crate::widgets::display::pieces::status_bar::{StatusBar, StatusBarProps};

/// Create a status bar builder. Its regions are empty until set.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::widgets::pieces::status_bar::status_bar;
///
/// let bar = status_bar().left("Ready").right("Ln 4").build();
/// ```
pub fn status_bar() -> StatusBarBuilder {
    StatusBarBuilder {
        props: StatusBarProps::default(),
    }
}

/// Builder for a status bar.
#[derive(Clone, Debug, Default)]
pub struct StatusBarBuilder {
    props: StatusBarProps,
}

impl StatusBarBuilder {
    /// Set the text at the left edge.
    pub fn left(mut self, text: impl Into<String>) -> Self {
        self.props.left = text.into();
        self
    }

    /// Set the text in the middle.
    pub fn center(mut self, text: impl Into<String>) -> Self {
        self.props.center = text.into();
        self
    }

    /// Set the text at the right edge.
    pub fn right(mut self, text: impl Into<String>) -> Self {
        self.props.right = text.into();
        self
    }

    /// Add classes to the bar. A `w-`, `h-` class sets its size.
    pub fn class(mut self, class: &str) -> Self {
        self.props.classes.push(class.to_string());
        self
    }

    /// Build the bar's element.
    pub fn build(self) -> Element {
        Element::typed::<StatusBar>(self.props)
    }
}
