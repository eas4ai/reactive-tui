//! The resizable split builder (NAV-007 and NAV-008): `builder::resizable()`.
//! Its element is a `Resizable` with the builder's callback handed to it.

use crate::component::Element;
use crate::widgets::layout::resizable::ResizeCallback;
use crate::widgets::layout::{Pane, Resizable, ResizableProps, SplitDirection};
use std::sync::Arc;

/// Builder for resizable splits.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::{div, resizable};
///
/// let split = resizable()
///     .vertical()
///     .aria_label("Panes")
///     .pane(div().build())
///     .pane(div().build())
///     .build();
/// ```
#[derive(Clone, Default)]
pub struct ResizableBuilder {
    props: ResizableProps,
    class: Option<String>,
    on_resize: Option<ResizeCallback>,
}

/// Create a resizable split builder.
pub fn resizable() -> ResizableBuilder {
    ResizableBuilder::new()
}

impl ResizableBuilder {
    /// Create a builder with the default settings: panes side by side.
    pub fn new() -> Self {
        Self::default()
    }

    /// Lay the panes side by side, with vertical dividers (the default).
    pub fn horizontal(mut self) -> Self {
        self.props.direction = SplitDirection::Row;
        self
    }

    /// Stack the panes, with horizontal dividers.
    pub fn vertical(mut self) -> Self {
        self.props.direction = SplitDirection::Column;
        self
    }

    /// Add a pane with an equal share.
    pub fn pane(self, content: Element) -> Self {
        self.pane_with(Pane::new(content))
    }

    /// Add a pane built with its own size and minimum.
    pub fn pane_with(mut self, pane: Pane) -> Self {
        self.props.panes.push(pane);
        self
    }

    /// Set the name the screen reader hears for the split.
    pub fn aria_label(mut self, label: &str) -> Self {
        self.props.aria_label = Some(label.to_string());
        self
    }

    /// Set whether the dividers ignore keys and drags.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.props.disabled = disabled;
        self
    }

    /// Set the CSS-like class names of the element.
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Set the callback called with every pane's size after each move.
    pub fn on_resize(mut self, f: impl Fn(Vec<u16>) + Send + Sync + 'static) -> Self {
        self.on_resize = Some(Arc::new(f));
        self
    }

    /// Build the split's element.
    pub fn build(self) -> Element {
        let on_resize = self.on_resize;
        let mut element = Element::typed_with::<Resizable>(self.props, move |props| {
            let mut split = Resizable::new(props);
            split.on_resize = on_resize.clone();
            split
        });
        if let Some(class) = self.class {
            element = element.with_class(&class);
        }
        element
    }
}
