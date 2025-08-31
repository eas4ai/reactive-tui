//! Web-like API for creating terminal UIs with React + Tailwind patterns
//!
//! This module provides a familiar web development experience for terminal applications.
//! Use `div()`, `span()`, `p()`, etc. with Tailwind-like CSS classes for responsive layouts.

use crate::component::{Element, ElementType, LayoutType};

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
pub fn grid() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Grid)).class("grid")
}

/// Create a flex container
pub fn flex() -> ElementBuilder {
    ElementBuilder::new(ElementType::Layout(LayoutType::Flex)).class("flex")
}

/// Builder for creating elements with a fluent API
pub struct ElementBuilder {
    element: Element,
    current_class: String,
}

impl ElementBuilder {
    fn new(element_type: ElementType) -> Self {
        Self {
            element: Element {
                element_type,
                props: std::sync::Arc::new(()),
                children: Vec::new(),
                key: None,
                class: None,
            },
            current_class: String::new(),
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

    /// Set an ID (maps to key internally)
    pub fn id(self, id: &str) -> Self {
        self.key(id)
    }

    /// Build the final Element
    pub fn build(self) -> Element {
        self.element
    }
}

// Implement Into<Element> for convenience
impl Into<Element> for ElementBuilder {
    fn into(self) -> Element {
        self.build()
    }
}

/// Convenience functions for common layout patterns

/// Create a full-screen container that fills the terminal
pub fn screen() -> ElementBuilder {
    div().class("h-screen w-screen flex flex-col")
}

/// Create a centered container
pub fn container() -> ElementBuilder {
    div().class("container mx-auto")
}

/// Create a card-like container
pub fn card() -> ElementBuilder {
    div().class("bg-white border border-gray-200 rounded-lg shadow-sm")
}

/// Create a button-like element
pub fn button() -> ElementBuilder {
    div().class("px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 cursor-pointer")
}

/// Create an input-like element
pub fn input() -> ElementBuilder {
    div().class("px-3 py-2 border border-gray-300 rounded focus:outline-none focus:border-blue-500")
}

/// Create a sidebar layout
pub fn sidebar() -> ElementBuilder {
    aside().class("w-64 bg-gray-800 text-white")
}

/// Create a content area
pub fn content() -> ElementBuilder {
    main().class("flex-1 p-6")
}

/// Create a responsive grid
pub fn responsive_grid(cols: u8) -> ElementBuilder {
    grid().class(&format!("grid-cols-1 md:grid-cols-{} gap-4", cols))
}

/// Text utility functions

/// Create a heading element
pub fn h1() -> ElementBuilder {
    div().class("text-4xl font-bold")
}

pub fn h2() -> ElementBuilder {
    div().class("text-3xl font-bold")
}

pub fn h3() -> ElementBuilder {
    div().class("text-2xl font-bold")
}

pub fn h4() -> ElementBuilder {
    div().class("text-xl font-bold")
}

pub fn h5() -> ElementBuilder {
    div().class("text-lg font-bold")
}

pub fn h6() -> ElementBuilder {
    div().class("text-base font-bold")
}

/// Create a text element with specific styling
pub fn text(content: &str) -> Element {
    span().text(content).build()
}

/// Create a label element
pub fn label(content: &str) -> Element {
    span().class("text-sm font-medium").text(content).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_div_creation() {
        let element = div()
            .class("flex items-center justify-center")
            .child(text("Hello World"))
            .build();

        assert!(matches!(
            element.element_type,
            ElementType::Layout(LayoutType::Flex)
        ));
        assert_eq!(
            element.class,
            Some("flex items-center justify-center".to_string())
        );
        assert_eq!(element.children.len(), 1);
    }

    #[test]
    fn test_responsive_layout() {
        let layout = screen()
            .children(vec![
                header()
                    .class("p-4 bg-blue-600")
                    .child(text("Header"))
                    .build(),
                div()
                    .class("flex flex-1")
                    .children(vec![
                        sidebar().child(text("Sidebar")).build(),
                        content().child(text("Main Content")).build(),
                    ])
                    .build(),
                footer()
                    .class("p-2 bg-gray-800")
                    .child(text("Footer"))
                    .build(),
            ])
            .build();

        assert_eq!(layout.children.len(), 3);
    }
}
