//! Core builder functionality for creating terminal UI elements
//!
//! This module provides the fundamental ElementBuilder struct and basic HTML-like
//! element creation functions that form the foundation of the builder API.

use crate::component::{Element, ElementType, LayoutType};
use crate::layout::css::gradients::{Gradient, GradientBorder, GradientDirection};

/// Create a div element (flex container by default)
pub fn div() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex))
}

/// Create a span element (inline text container)
pub fn span() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("inline")
}

/// Create a paragraph element
pub fn p() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a section element
pub fn section() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create an article element
pub fn article() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a header element
pub fn header() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a footer element
pub fn footer() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a nav element
pub fn nav() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a main element
pub fn main() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block flex-1")
}

/// Create an aside element (sidebar)
pub fn aside() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("block")
}

/// Create a grid container
pub fn grid_builder() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Grid)).class("grid")
}

/// Create a flex container
pub fn flex() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("flex")
}

/// Create a button-like element
pub fn button() -> ElementBuilder {
    div().class("px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer")
}

/// Create an editable single-line input. `text` supplies its initial value.
pub fn input() -> ElementBuilder {
    let mut builder = div().class(
        "px-0.5 py-0.25 border border-gray-300 rounded focus:outline-none focus:border-blue-500",
    );
    builder.input = Some(crate::widgets::TextInputProps::default());
    builder
}

/// Builder for creating elements with a fluent API
pub struct ElementBuilder {
    element: Element,
    current_class: String,
    gradient: Option<Gradient>,
    gradient_border: Option<GradientBorder>,
    input: Option<crate::widgets::TextInputProps>,
}

impl ElementBuilder {
    /// Create a new ElementBuilder with the specified element type
    pub fn new(element_type: ElementType) -> Self {
        Self {
            element: Element {
                element_type,
                props: std::sync::Arc::new(()),
                children: Vec::new(),
                key: None,
                class: None,
                focus: None,
                metadata: Default::default(),
            },
            current_class: String::new(),
            gradient: None,
            gradient_border: None,
            input: None,
        }
    }

    /// Set CSS classes (Tailwind-like utilities)
    pub fn class(mut self, classes: &str) -> Self {
        if !self.current_class.is_empty() {
            self.current_class.push(' ');
        }
        self.current_class.push_str(classes);
        self.element.class = Some(self.current_class.clone());
        self
    }

    /// Apply CSS-in-Rust styles
    ///
    /// This method allows you to apply styles using the `css!` macro
    /// for type-safe, compile-time validated styling.
    ///
    /// # Arguments
    /// * `style_builder` - A StyleBuilder created with the `css!` macro
    ///
    /// # Example
    /// ```rust,ignore
    /// use reactive_tui::prelude::*;
    /// use reactive_tui::css;
    ///
    /// let element = div()
    ///     .styles(css! {
    ///         display: Display::Flex,
    ///         background_color: (0.0, 0.0, 1.0, 1.0),
    ///         padding: 16.0,
    ///     })
    ///     .build();
    /// ```
    pub fn styles(mut self, style_builder: crate::layout::style::StyleBuilder) -> Self {
        self.element.metadata.styles = Some(std::sync::Arc::new(style_builder.snapshot()));
        // Retain the existing public marker as well as the actual style data.
        self.class("css-in-rust-applied")
    }

    /// Add additional CSS classes
    pub fn add_class(self, classes: &str) -> Self {
        self.class(classes)
    }

    /// Set text content (for text nodes)
    pub fn text(mut self, content: &str) -> Self {
        if let Some(input) = &mut self.input {
            input.value = content.to_owned();
            return self;
        }
        // Convert to text element if it's not already
        self.element.element_type = ElementType::Text(content.to_string());
        self.element.props = std::sync::Arc::new(content.to_string());
        self
    }

    /// Add a single child element
    pub fn child(mut self, child: Element) -> Self {
        self.element.children.push(child);
        self
    }

    /// Add multiple child elements
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.element.children.extend(children);
        self
    }

    /// Add children from an iterator
    pub fn children_iter<I>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = Element>,
    {
        self.element.children.extend(children);
        self
    }

    /// Set a key for React-like reconciliation
    pub fn key(mut self, key: &str) -> Self {
        self.element.key = Some(key.to_string());
        self
    }

    /// Add click handler - integrates with reactive-tui's event system
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.element
            .metadata
            .on_click
            .push(std::sync::Arc::new(handler));
        self.class("interactive")
    }

    /// Set placeholder for input elements
    /// Inputs render this text while empty; the styling marker is also retained.
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        if let Some(input) = &mut self.input {
            input.placeholder = Some(placeholder.to_owned());
        }
        self.class(&format!(
            "data-placeholder-{}",
            placeholder
                .replace(" ", "-")
                .replace("'", "")
                .replace("\"", "")
        ))
    }

    /// Set an ID (maps to key internally)
    pub fn id(self, id: &str) -> Self {
        self.key(id)
    }

    /// Set a gradient background
    pub fn gradient(mut self, direction: GradientDirection) -> Self {
        let gradient = Gradient::new(direction);
        self.gradient = Some(gradient);
        self
    }

    /// Set gradient from color
    pub fn from_color(mut self, r: u8, g: u8, b: u8) -> Self {
        if let Some(ref mut gradient) = self.gradient {
            gradient.stops.from = Some((r, g, b, 1.0));
        } else {
            let mut gradient = Gradient::new(GradientDirection::ToRight);
            gradient.stops.from = Some((r, g, b, 1.0));
            self.gradient = Some(gradient);
        }
        self
    }

    /// Set gradient via (middle) color
    pub fn via_color(mut self, r: u8, g: u8, b: u8) -> Self {
        if let Some(ref mut gradient) = self.gradient {
            gradient.stops.via = Some((r, g, b, 1.0));
        } else {
            let mut gradient = Gradient::new(GradientDirection::ToRight);
            gradient.stops.via = Some((r, g, b, 1.0));
            self.gradient = Some(gradient);
        }
        self
    }

    /// Set gradient to color
    pub fn to_color(mut self, r: u8, g: u8, b: u8) -> Self {
        if let Some(ref mut gradient) = self.gradient {
            gradient.stops.to = Some((r, g, b, 1.0));
        } else {
            let mut gradient = Gradient::new(GradientDirection::ToRight);
            gradient.stops.to = Some((r, g, b, 1.0));
            self.gradient = Some(gradient);
        }
        self
    }

    /// Add gradient border with specified width
    pub fn gradient_border(mut self, width: usize) -> Self {
        if let Some(ref gradient) = self.gradient {
            self.gradient_border = Some(GradientBorder::new(gradient.clone(), width));
        } else {
            // Create default rainbow gradient for border if no gradient specified
            let gradient_border = GradientBorder::rainbow_border(width);
            self.gradient_border = Some(gradient_border);
        }
        self
    }

    /// Set whether the resulting element accepts focus and activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.element.metadata.disabled = disabled;
        self
    }

    /// Build the final Element
    pub fn build(mut self) -> Element {
        if let Some(props) = self.input {
            let input = Element::typed::<crate::widgets::TextInput>(props);
            self.element.element_type = input.element_type;
            self.element.props = input.props;
            self.element.metadata.factory = input.metadata.factory;
            // Editing consumes pointer input during bubbling. Observe the click
            // on the same measured target before the editor handles it.
            let callbacks = std::mem::take(&mut self.element.metadata.on_click);
            if !callbacks.is_empty() {
                self.element
                    .metadata
                    .capture_events
                    .push(std::sync::Arc::new(move |event| {
                        use crate::event::{
                            router::EventResult,
                            types::{Event, MouseButton, MouseEventKind},
                        };
                        if matches!(event, Event::Mouse(mouse) if mouse.button == MouseButton::Left
                        && matches!(mouse.kind, MouseEventKind::Down | MouseEventKind::Click))
                        {
                            for callback in &callbacks {
                                callback();
                            }
                        }
                        EventResult::Ignored
                    }));
            }
        }
        self.element.metadata.gradient = self.gradient;
        self.element.metadata.gradient_border = self.gradient_border;

        self.element
    }
}

// Implement Into<Element> for convenience
impl From<ElementBuilder> for Element {
    fn from(val: ElementBuilder) -> Self {
        val.build()
    }
}

/// Universal conversion to Element
pub fn to_element<T: IntoElement>(item: T) -> Element {
    item.into_element()
}

/// Trait for anything that can become an Element
pub trait IntoElement {
    /// Convert this item into an Element
    ///
    /// # Returns
    /// An `Element` representation of this item
    fn into_element(self) -> Element;
}

impl IntoElement for Element {
    fn into_element(self) -> Element {
        self
    }
}

impl IntoElement for crate::vdom::VNode {
    fn into_element(self) -> Element {
        crate::vdom::bridge::vdom_to_element(self)
    }
}

impl IntoElement for &str {
    fn into_element(self) -> Element {
        Element::text(self)
    }
}

impl IntoElement for String {
    fn into_element(self) -> Element {
        Element::text(self)
    }
}

impl IntoElement for ElementBuilder {
    fn into_element(self) -> Element {
        self.build()
    }
}
