//! The log view builder (NAV-010): `builder::log_view()`. Its element is a
//! `LogView` that shows the rows of the handle the builder was given.

use crate::component::Element;
use crate::widgets::layout::{LogHandle, LogView, LogViewProps};

/// Builder for log views.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::log_view;
/// use reactive_tui::widgets::layout::{LogHandle, LogRow};
///
/// let handle = LogHandle::new();
/// handle.append(LogRow::new("Build started"));
/// let view = log_view().handle(handle).aria_label("Build log").build();
/// ```
#[derive(Clone)]
pub struct LogViewBuilder {
    props: LogViewProps,
    class: Option<String>,
}

/// Create a log view builder with a new, empty handle.
pub fn log_view() -> LogViewBuilder {
    LogViewBuilder::new()
}

impl Default for LogViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogViewBuilder {
    /// Create a builder whose view shows a new, empty handle.
    pub fn new() -> Self {
        Self {
            props: LogViewProps {
                handle: LogHandle::new(),
                new_word: None,
                aria_label: None,
                width: None,
                height: None,
            },
            class: None,
        }
    }

    /// Set the rows the view shows.
    pub fn handle(mut self, handle: LogHandle) -> Self {
        self.props.handle = handle;
        self
    }

    /// Set the word in the `↓ N new` bar.
    pub fn new_word(mut self, word: &str) -> Self {
        self.props.new_word = Some(word.to_string());
        self
    }

    /// Set the name the screen reader hears for the view.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set the CSS-like class names of the element.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the view's element.
    pub fn build(self) -> Element {
        let mut element = Element::typed::<LogView>(self.props);
        if let Some(class) = self.class {
            element = element.with_class(&class);
        }
        element
    }
}
