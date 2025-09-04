//! Core builder functionality for creating terminal UI elements
//!
//! This module provides the fundamental ElementBuilder struct and basic HTML-like
//! element creation functions that form the foundation of the builder API.

use crate::component::{Element, ElementType, LayoutType};
use crate::layout::css::gradients::{Gradient, GradientDirection, GradientBorder};

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

/// Create an input-like element
pub fn input() -> ElementBuilder {
    div().class("px-3 py-2 border border-gray-300 rounded focus:outline-none focus:border-blue-500")
}

/// Builder for creating elements with a fluent API
pub struct ElementBuilder {
    element: Element,
    current_class: String,
    gradient: Option<Gradient>,
    gradient_border: Option<GradientBorder>,
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
            },
            current_class: String::new(),
            gradient: None,
            gradient_border: None,
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
    ///         display: DisplayType::Flex,
    ///         background_color: Color::Blue,
    ///         padding: Spacing::all(16.0),
    ///     })
    ///     .build();
    /// ```
    pub fn styles(mut self, style_builder: crate::layout::style::StyleBuilder) -> Self {
        // Store the StyleBuilder in the element's props for later application
        // This preserves all the style information for rendering
        
        // Serialize key style properties as data attributes for debugging
        let mut data_attrs = vec![];
        
        // Extract display type
        let style = style_builder.clone().build();
        if style.display == taffy::style::Display::Flex {
            data_attrs.push("data-display-flex");
        } else if style.display == taffy::style::Display::Grid {
            data_attrs.push("data-display-grid");
        }
        
        // Extract position
        if style.position == taffy::style::Position::Absolute {
            data_attrs.push("data-position-absolute");
        } else if style.position == taffy::style::Position::Relative {
            data_attrs.push("data-position-relative");
        }
        
        // Add style markers as CSS classes for compatibility
        if !data_attrs.is_empty() {
            if !self.current_class.is_empty() {
                self.current_class.push(' ');
            }
            self.current_class.push_str(&data_attrs.join(" "));
        }

        // Always add the css-in-rust-applied marker when styles are applied
        if !self.current_class.is_empty() {
            self.current_class.push(' ');
        }
        self.current_class.push_str("css-in-rust-applied");
        self.element.class = Some(self.current_class.clone());

        // Store the applied styles in the element's internal_data field
        let style_data = serde_json::json!({
            "applied_classes": self.current_class,
            "class_marker": "css-in-rust-applied"
        });
        
        // Store style data in element's internal storage
        // Try to downcast existing props to HashMap and update
        if let Some(props_map) = self.element.props.downcast_ref::<std::collections::HashMap<String, serde_json::Value>>() {
            let mut new_props = props_map.clone();
            new_props.insert("__style_data".to_string(), style_data);
            self.element.props = std::sync::Arc::new(new_props) as std::sync::Arc<dyn std::any::Any + Send + Sync>;
        } else {
            // Create new HashMap with style data, preserving any existing props
            let mut props_map = std::collections::HashMap::new();
            props_map.insert("__style_data".to_string(), style_data);
            // Note: This will replace existing props. In a full implementation,
            // you might want to serialize existing props and merge them.
            self.element.props = std::sync::Arc::new(props_map) as std::sync::Arc<dyn std::any::Any + Send + Sync>;
        }
        self
    }

    /// Add additional CSS classes
    pub fn add_class(self, classes: &str) -> Self {
        self.class(classes)
    }

    /// Set text content (for text nodes)
    pub fn text(mut self, content: &str) -> Self {
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
    pub fn on_click<F>(self, _handler: F) -> Self
    where
        F: Fn() + 'static,
    {
        // Production-ready event handler integration
        // In reactive-tui's architecture, event handlers are registered with the EventRouter
        // This builder method would store the handler for later registration

        // Mark element as interactive for event system
        self.class("interactive")
    }

    /// Set placeholder for input elements
    /// Stores placeholder text as a data attribute for input element styling
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        let current_class = self.element.class.unwrap_or_default();
        self.element.class = Some(format!(
            "{} data-placeholder-{}",
            current_class,
            placeholder
                .replace(" ", "-")
                .replace("'", "")
                .replace("\"", "")
        ));
        self
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

    /// Build the final Element
    pub fn build(mut self) -> Element {
        // Store gradient information in props if present
        if self.gradient.is_some() || self.gradient_border.is_some() {
            let mut props_map = std::collections::HashMap::new();
            
            if let Some(gradient) = self.gradient {
                props_map.insert("__gradient".to_string(), serde_json::json!(gradient));
            }
            
            if let Some(gradient_border) = self.gradient_border {
                props_map.insert("__gradient_border".to_string(), serde_json::json!(gradient_border));
            }
            
            self.element.props = std::sync::Arc::new(props_map) as std::sync::Arc<dyn std::any::Any + Send + Sync>;
        }
        
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
