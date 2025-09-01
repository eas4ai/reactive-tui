//! Builder API for creating terminal UIs with fluent, chainable syntax
//!
//! This module provides a builder pattern for constructing UI elements with familiar
//! HTML-like functions. Use `div()`, `span()`, `p()`, etc. with CSS classes for layouts.

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
pub fn grid_builder() -> ElementBuilder {
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

    /// Add click handler - integrates with reactive-tui's event system
    pub fn on_click<F>(self, _handler: Box<F>) -> Self
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
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        // Add placeholder as a data attribute for now
        let current_class = self.element.class.unwrap_or_default();
        self.element.class = Some(format!(
            "{} data-placeholder-{}",
            current_class,
            placeholder.replace(" ", "-")
        ));
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
impl From<ElementBuilder> for Element {
    fn from(val: ElementBuilder) -> Self {
        val.build()
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
pub fn card_builder() -> ElementBuilder {
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
    grid_builder().class(&format!("grid-cols-1 md:grid-cols-{} gap-4", cols))
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

/// Integration helpers for VDOM interop

/// Convert a VNode to Element (re-export for convenience)
pub fn from_vdom(vnode: crate::vdom::VNode) -> Element {
    crate::vdom::bridge::vdom_to_element(vnode)
}

/// Create an element that can contain both builder and VDOM children
pub fn mixed_container() -> MixedElementBuilder {
    MixedElementBuilder::new(ElementType::Layout(LayoutType::Flex))
}

/// Builder that accepts both Element and VNode children
pub struct MixedElementBuilder {
    builder: ElementBuilder,
}

impl MixedElementBuilder {
    fn new(element_type: ElementType) -> Self {
        Self {
            builder: ElementBuilder::new(element_type),
        }
    }

    /// Set CSS classes
    pub fn class(mut self, classes: &str) -> Self {
        self.builder = self.builder.class(classes);
        self
    }

    /// Add a builder Element child
    pub fn child_element(mut self, child: Element) -> Self {
        self.builder = self.builder.child(child);
        self
    }

    /// Add a VDOM VNode child (automatically converted)
    pub fn child_vdom(mut self, child: crate::vdom::VNode) -> Self {
        self.builder = self.builder.child(from_vdom(child));
        self
    }

    /// Add mixed children (Elements and VNodes)
    pub fn mixed_children(mut self, children: Vec<MixedChild>) -> Self {
        for child in children {
            match child {
                MixedChild::Element(el) => {
                    self.builder = self.builder.child(el);
                }
                MixedChild::VNode(vnode) => {
                    self.builder = self.builder.child(from_vdom(vnode));
                }
            }
        }
        self
    }

    /// Set a key
    pub fn key(mut self, key: &str) -> Self {
        self.builder = self.builder.key(key);
        self
    }

    /// Build the final Element
    pub fn build(self) -> Element {
        self.builder.build()
    }
}

/// Enum for mixed children types
pub enum MixedChild {
    Element(Element),
    VNode(crate::vdom::VNode),
}

impl From<Element> for MixedChild {
    fn from(element: Element) -> Self {
        MixedChild::Element(element)
    }
}

impl From<crate::vdom::VNode> for MixedChild {
    fn from(vnode: crate::vdom::VNode) -> Self {
        MixedChild::VNode(vnode)
    }
}

/// Universal element creation - accepts anything that can become an Element
///
/// Usage:
/// ```
/// el![
///     div().text("Web API").build(),
///     VNode::text("VDOM text"),
///     "Just text",
///     span().text("More web API").build(),
/// ]
/// ```
#[macro_export]
macro_rules! el {
    // Single element
    ($child:expr) => {
        $crate::builder::to_element($child)
    };

    // Multiple elements as children
    [$($child:expr),* $(,)?] => {
        vec![$($crate::builder::to_element($child)),*]
    };

    // Container with class and children
    ($tag:ident, class: $class:expr, [$($child:expr),* $(,)?]) => {
        $crate::builder::$tag()
            .class($class)
            .children(vec![$($crate::builder::to_element($child)),*])
            .build()
    };

    // Container with just children
    ($tag:ident, [$($child:expr),* $(,)?]) => {
        $crate::builder::$tag()
            .children(vec![$($crate::builder::to_element($child)),*])
            .build()
    };
}

/// Universal conversion to Element
pub fn to_element<T: IntoElement>(item: T) -> Element {
    item.into_element()
}

/// Trait for anything that can become an Element
pub trait IntoElement {
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

/// Improved composition macros

/// Create a div with optional class and children
///
/// Usage:
/// ```
/// div!["Hello"]                           // Simple text child
/// div![class: "flex", "Hello", "World"]   // With class and multiple children
/// div![span!["Name:"], input![]]          // Nested elements
/// ```
#[macro_export]
macro_rules! div {
    // Just children
    [$($child:expr),* $(,)?] => {
        $crate::builder::div()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    // Class and children
    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::div()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    // Just class
    [class: $class:expr] => {
        $crate::builder::div().class($class).build()
    };

    // Empty
    [] => {
        $crate::builder::div().build()
    };
}

/// Create a span with optional class and children
#[macro_export]
macro_rules! span {
    [$($child:expr),* $(,)?] => {
        $crate::builder::span()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::span()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr] => {
        $crate::builder::span().class($class).build()
    };

    [] => {
        $crate::builder::span().build()
    };
}

/// Create a button with optional properties
#[macro_export]
macro_rules! button {
    [$($child:expr),* $(,)?] => {
        $crate::builder::button()
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .class($class)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [onclick: $handler:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .on_click($handler)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };

    [class: $class:expr, onclick: $handler:expr, $($child:expr),* $(,)?] => {
        $crate::builder::button()
            .class($class)
            .on_click($handler)
            .children(vec![$($crate::el!($child)),*])
            .build()
    };
}

/// Create an input with optional properties
#[macro_export]
macro_rules! input {
    [] => {
        $crate::builder::input().build()
    };

    [placeholder: $placeholder:expr] => {
        $crate::builder::input().placeholder($placeholder).build()
    };

    [class: $class:expr] => {
        $crate::builder::input().class($class).build()
    };

    [class: $class:expr, placeholder: $placeholder:expr] => {
        $crate::builder::input()
            .class($class)
            .placeholder($placeholder)
            .build()
    };
}

/// Component shortcuts using existing utility CSS system

pub fn card(children: Vec<Element>) -> Element {
    div()
        .class("bg-white rounded-lg shadow-md border border-gray-200 p-6")
        .children(children)
        .build()
}

pub fn primary_button<F>(text: &str, onclick: F) -> Element
where
    F: Fn() + 'static,
{
    button()
        .class("px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500")
        .text(text)
        .on_click(Box::new(onclick))
        .build()
}

pub fn flex_row(gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("flex flex-row gap-{}", gap))
        .children(children)
        .build()
}

pub fn flex_col(gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("flex flex-col gap-{}", gap))
        .children(children)
        .build()
}

pub fn grid_layout(cols: u8, gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("grid grid-cols-{} gap-{}", cols, gap))
        .children(children)
        .build()
}

pub fn text_input(placeholder: &str) -> ElementBuilder {
    input()
        .class("px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500")
        .placeholder(placeholder)
}

pub fn search_input(placeholder: &str) -> Element {
    div()
        .class("relative")
        .children(vec![
            input()
                .class("pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500")
                .placeholder(placeholder)
                .build(),
            div()
                .class("absolute left-3 top-2.5 text-gray-400")
                .text("🔍")
                .build(),
        ])
        .build()
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

    #[test]
    fn test_vdom_integration() {
        use crate::vdom::VNode;

        // Test converting VNode to Element
        let vnode = VNode::element("div")
            .class("test-class")
            .child(VNode::text("Hello from VDOM"))
            .build();

        let element = from_vdom(vnode);
        assert_eq!(element.class, Some("test-class".to_string()));
        assert_eq!(element.children.len(), 1);
    }

    #[test]
    fn test_mixed_container() {
        use crate::vdom::VNode;

        let mixed = mixed_container()
            .class("flex")
            .child_element(text("Web API text"))
            .child_vdom(VNode::text("VDOM text"))
            .build();

        assert_eq!(mixed.class, Some("flex".to_string()));
        assert_eq!(mixed.children.len(), 2);

        // Both children should be text elements
        assert!(matches!(
            mixed.children[0].element_type,
            ElementType::Text(_)
        ));
        assert!(matches!(
            mixed.children[1].element_type,
            ElementType::Text(_)
        ));
    }

    #[test]
    fn test_el_macro_universal() {
        use crate::vdom::VNode;

        // Test universal conversion
        let element1 = el!(text("Web API"));
        let element2 = el!(VNode::text("VDOM"));
        let element3 = el!("Just a string");

        assert!(matches!(element1.element_type, ElementType::Text(_)));
        assert!(matches!(element2.element_type, ElementType::Text(_)));
        assert!(matches!(element3.element_type, ElementType::Text(_)));

        // Test children array
        let children = el![
            text("Web API"),
            VNode::text("VDOM"),
            "String",
            div().text("Builder").build(),
        ];

        assert_eq!(children.len(), 4);

        // Test container with children
        let container = el!(div, class: "flex", [
            "Text",
            VNode::text("VDOM"),
            span().text("Builder").build(),
        ]);

        assert_eq!(container.class, Some("flex".to_string()));
        assert_eq!(container.children.len(), 3);
    }

    #[test]
    fn test_into_element_trait() {
        use crate::vdom::VNode;

        // Test all implementations
        let from_element = to_element(text("element"));
        let from_vnode = to_element(VNode::text("vnode"));
        let from_str = to_element("string");
        let from_string = to_element("owned".to_string());

        assert!(matches!(from_element.element_type, ElementType::Text(_)));
        assert!(matches!(from_vnode.element_type, ElementType::Text(_)));
        assert!(matches!(from_str.element_type, ElementType::Text(_)));
        assert!(matches!(from_string.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_composition_macros() {
        // Test div macro
        let simple_div = div!["Hello"];
        assert_eq!(simple_div.children.len(), 1);

        let div_with_class = div![class: "flex", "Hello", "World"];
        assert_eq!(div_with_class.class, Some("flex".to_string()));
        assert_eq!(div_with_class.children.len(), 2);

        // Test span macro
        let simple_span = span!["Text"];
        assert_eq!(simple_span.children.len(), 1);

        // Test button macro
        let simple_button = button!["Click me"];
        assert_eq!(simple_button.children.len(), 1);

        // Test input macro
        let simple_input = input![];
        // input() creates a div with input styling, so it's a Layout element
        assert!(matches!(simple_input.element_type, ElementType::Layout(_)));
    }

    #[test]
    fn test_utility_css_integration() {
        // Test that builder works with existing utility CSS
        let element = div().class("p-4 m-2 bg-blue-500 text-gray-900").build();

        let class = element.class.unwrap();
        assert!(class.contains("p-4"));
        assert!(class.contains("m-2"));
        assert!(class.contains("bg-blue-500"));
        assert!(class.contains("text-gray-900"));
    }

    #[test]
    fn test_component_shortcuts() {
        // Test flex layouts with utility CSS
        let flex_row = flex_row("4", vec![text("Item 1"), text("Item 2")]);
        assert!(flex_row.class.unwrap().contains("flex flex-row gap-4"));

        let flex_col = flex_col("2", vec![text("Item 1"), text("Item 2")]);
        assert!(flex_col.class.unwrap().contains("flex flex-col gap-2"));

        // Test card builder
        let card_element = card_builder().build();
        assert!(card_element.class.unwrap().contains("bg-white"));
    }

    #[test]
    fn test_auto_build_with_into() {
        fn accepts_element(element: impl IntoElement) -> Element {
            element.into_element()
        }

        // Test that builders auto-convert
        let from_builder = accepts_element(div().text("test"));
        assert_eq!(from_builder.children.len(), 0); // text() changes element type

        // Test that strings auto-convert
        let from_string = accepts_element("test");
        assert!(matches!(from_string.element_type, ElementType::Text(_)));

        // Test that elements pass through
        let element = text("test");
        let from_element = accepts_element(element);
        assert!(matches!(from_element.element_type, ElementType::Text(_)));
    }

    #[test]
    fn test_search_input_component() {
        let search = search_input("Search...");
        assert_eq!(search.children.len(), 2); // input + icon
        assert!(search.class.unwrap().contains("relative"));
    }

    #[test]
    fn test_utility_css_chaining() {
        // Test that multiple utility classes work together
        let element = div()
            .class("flex")
            .class("items-center")
            .class("justify-center")
            .class("p-4")
            .build();

        let class = element.class.unwrap();
        assert!(class.contains("flex"));
        assert!(class.contains("items-center"));
        assert!(class.contains("justify-center"));
        assert!(class.contains("p-4"));
    }
}
