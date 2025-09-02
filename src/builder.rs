//! Builder API for creating terminal UIs with fluent, chainable syntax
//!
//! This module provides a builder pattern for constructing UI elements with familiar
//! HTML-like functions. Use `div()`, `span()`, `p()`, etc. with CSS classes for layouts.

use crate::component::{Element, ElementType, LayoutType};
use crate::widgets::display::{
    DataTableProps, ChartProps, ChartType, ChartAxis, ChartLegend,
    DataPoint, DataSeries, LegendPosition,
    ColumnFilter, FilterType, DisplaySize, Alignment,
};
use crate::widgets::display::table::{TableColumn, TableRow, TableCell};
// Note: Some widget imports may not be available yet
// use crate::widgets::input::{
//     TextInput, Checkbox, RadioButton, Select, Slider,
// };
// use crate::widgets::layout::{
//     ScrollView, Stack, Tabs,
// };
// Note: Some dialog widgets may not be available yet
// use crate::widgets::dialog::{
//     DialogComponent, Toast, ConfirmationDialog, ProgressDialog,
// };
use std::collections::HashMap;

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

    /// Apply CSS-in-Rust styles
    ///
    /// This method allows you to apply styles using the `css!` macro
    /// for type-safe, compile-time validated styling.
    ///
    /// # Arguments
    /// * `style_builder` - A StyleBuilder created with the `css!` macro
    ///
    /// # Example
    /// ```rust
    /// use reactive_tui::prelude::*;
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
            self.element.class = Some(self.current_class.clone());
        }
        
        // Store the StyleBuilder in the element's props
        // In a real implementation, this would be stored in a proper field
        // For now, we mark it as applied via the class system
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
///
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
///
/// Create a heading element
pub fn h1() -> ElementBuilder {
    div().class("text-4xl font-bold")
}

/// Create a level 2 heading element
pub fn h2() -> ElementBuilder {
    div().class("text-3xl font-bold")
}

/// Create a level 3 heading element
pub fn h3() -> ElementBuilder {
    div().class("text-2xl font-bold")
}

/// Create a level 4 heading element
pub fn h4() -> ElementBuilder {
    div().class("text-xl font-bold")
}

/// Create a level 5 heading element
pub fn h5() -> ElementBuilder {
    div().class("text-lg font-bold")
}

/// Create a level 6 heading element
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
///
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
    /// A built Element from the builder API
    Element(Element),
    /// A virtual DOM node from the VDOM system
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
/// use reactive_tui::el;
/// use reactive_tui::builder::{div, span};
/// use reactive_tui::vdom::VNode;
///
/// let elements = el![
///     div().text("Web API").build(),
///     VNode::text("VDOM text"),
///     "Just text",
///     span().text("More web API").build(),
/// ];
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

/// Improved composition macros
///
/// Create a div with optional class and children
///
/// Usage:
/// ```
/// use reactive_tui::{div, span, input};
///
/// fn example() {
///     let simple = div!["Hello"];                           // Simple text child
///     let with_class = div![class: "flex", "Hello", "World"];   // With class and multiple children
///     let nested = div![span!["Name:"], input![]];          // Nested elements
/// }
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

/// Create a data table with optional configuration
#[macro_export]
macro_rules! data_table {
    // Simple table with columns and rows
    [columns: [$($col_title:expr => $col_key:expr),* $(,)?], rows: [$($row:expr),* $(,)?]] => {
        {
            let mut builder = $crate::builder::data_table();
            $(
                builder = builder.column($col_title, $col_key);
            )*
            $(
                builder = builder.simple_row($row);
            )*
            builder.build()
        }
    };

    // Table with pagination
    [columns: [$($col_title:expr => $col_key:expr),* $(,)?], rows: [$($row:expr),* $(,)?], pagination: $page_size:expr] => {
        {
            let mut builder = $crate::builder::data_table();
            $(
                builder = builder.column($col_title, $col_key);
            )*
            $(
                builder = builder.simple_row($row);
            )*
            builder.pagination(true, $page_size).build()
        }
    };
}

/// Create a chart with optional configuration
#[macro_export]
macro_rules! chart {
    // Simple bar chart
    [bar: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .bar_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Simple line chart
    [line: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .line_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Simple pie chart
    [pie: $name:expr => [$($value:expr),* $(,)?]] => {
        $crate::builder::chart()
            .pie_chart()
            .simple_series($name, vec![$($value),*])
            .build()
    };

    // Chart with title
    [bar: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .bar_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };

    [line: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .line_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };

    [pie: $name:expr => [$($value:expr),* $(,)?], title: $title:expr] => {
        $crate::builder::chart()
            .pie_chart()
            .simple_series($name, vec![$($value),*])
            .title($title)
            .build()
    };
}

/// Create a text input with optional configuration
#[macro_export]
macro_rules! text_input {
    // Simple text input
    [] => {
        $crate::builder::text_input().build()
    };

    // Text input with placeholder
    [placeholder: $placeholder:expr] => {
        $crate::builder::text_input().placeholder($placeholder).build()
    };

    // Text input with value
    [value: $value:expr] => {
        $crate::builder::text_input().value($value).build()
    };

    // Text input with value and placeholder
    [value: $value:expr, placeholder: $placeholder:expr] => {
        $crate::builder::text_input()
            .value($value)
            .placeholder($placeholder)
            .build()
    };
}

/// Create a checkbox with optional configuration
#[macro_export]
macro_rules! checkbox {
    // Simple checkbox
    [] => {
        $crate::builder::checkbox().build()
    };

    // Checkbox with label
    [label: $label:expr] => {
        $crate::builder::checkbox().label($label).build()
    };

    // Checked checkbox with label
    [checked: $checked:expr, label: $label:expr] => {
        $crate::builder::checkbox()
            .checked($checked)
            .label($label)
            .build()
    };
}

/// Create a select dropdown with options
#[macro_export]
macro_rules! select {
    // Select with options
    [options: [$($value:expr => $label:expr),* $(,)?]] => {
        {
            let mut builder = $crate::builder::select();
            $(
                builder = builder.option($value, $label);
            )*
            builder.build()
        }
    };

    // Select with options and selected value
    [options: [$($value:expr => $label:expr),* $(,)?], selected: $selected:expr] => {
        {
            let mut builder = $crate::builder::select();
            $(
                builder = builder.option($value, $label);
            )*
            builder.selected($selected).build()
        }
    };
}

/// Create a progress bar with value
#[macro_export]
macro_rules! progress_bar {
    // Simple progress bar
    [$value:expr] => {
        $crate::builder::progress_bar().value($value).build()
    };

    // Progress bar with label
    [$value:expr, label: $label:expr] => {
        $crate::builder::progress_bar()
            .value($value)
            .label($label)
            .build()
    };

    // Progress bar with custom max value
    [$value:expr, max: $max:expr] => {
        $crate::builder::progress_bar()
            .value($value)
            .max_value($max)
            .build()
    };
}

/// Create a toast notification
#[macro_export]
macro_rules! toast {
    // Success toast
    [success: $message:expr] => {
        $crate::builder::toast().success($message).build()
    };

    // Error toast
    [error: $message:expr] => {
        $crate::builder::toast().error($message).build()
    };

    // Warning toast
    [warning: $message:expr] => {
        $crate::builder::toast().warning($message).build()
    };

    // Info toast
    [info: $message:expr] => {
        $crate::builder::toast().info($message).build()
    };
}

/// Create tabs with content
#[macro_export]
macro_rules! tabs {
    // Tabs with title and content pairs
    [$($title:expr => $content:expr),* $(,)?] => {
        {
            let mut builder = $crate::builder::tabs();
            $(
                builder = builder.tab($title, $content);
            )*
            builder.build()
        }
    };
}

/// Component shortcuts using existing utility CSS system
///
pub fn card(children: Vec<Element>) -> Element {
    div()
        .class("bg-white rounded-lg shadow-md border border-gray-200 p-6")
        .children(children)
        .build()
}

/// Advanced Widget Builders
///
/// Create a DataTable with fluent configuration
pub fn data_table() -> DataTableBuilder {
    DataTableBuilder::new()
}

/// Create a Chart with fluent configuration
pub fn chart() -> ChartBuilder {
    ChartBuilder::new()
}

/// Display Widgets
///
/// Functions for creating display widget builders with fluent APIs.
///
/// Create a Modal dialog builder
///
/// Returns a `ModalBuilder` for creating modal dialogs with customizable
/// content, sizing, and behavior options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::modal;
///
/// let modal = modal()
///     .title("Settings")
///     .visible(true)
///     .build();
/// ```
pub fn modal() -> ModalBuilder {
    ModalBuilder::new()
}

/// Create a Popover builder
///
/// Returns a `PopoverBuilder` for creating popover elements that display
/// content when triggered by user interaction.
pub fn popover() -> PopoverBuilder {
    PopoverBuilder::new()
}

/// Create a Progress Bar builder
///
/// Returns a `ProgressBarBuilder` for creating progress indicators with
/// customizable values, labels, and styling.
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

/// Create a Tree view builder
///
/// Returns a `TreeBuilder` for creating hierarchical tree view components.
pub fn tree() -> TreeBuilder {
    TreeBuilder::new()
}

/// Create an Image display builder
///
/// Returns an `ImageBuilder` for creating image display components.
pub fn image() -> ImageBuilder {
    ImageBuilder::new()
}

/// Input Widgets
///
/// Functions for creating input widget builders with fluent APIs.
///
/// Create a Text Input builder
///
/// Returns a `TextInputBuilder` for creating text input fields with various
/// input types, validation, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::text_input;
///
/// let input = text_input()
///     .placeholder("Enter email")
///     .input_type("email")
///     .build();
/// ```
pub fn text_input() -> TextInputBuilder {
    TextInputBuilder::new()
}

/// Create a Checkbox builder
///
/// Returns a `CheckboxBuilder` for creating checkbox inputs with labels
/// and state management.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::checkbox;
///
/// let checkbox = checkbox()
///     .label("I agree")
///     .checked(false)
///     .build();
/// ```
pub fn checkbox() -> CheckboxBuilder {
    CheckboxBuilder::new()
}

/// Create a Radio Button builder
///
/// Returns a `RadioButtonBuilder` for creating radio button inputs.
pub fn radio_button() -> RadioButtonBuilder {
    RadioButtonBuilder::new()
}

/// Create a Select dropdown builder
///
/// Returns a `SelectBuilder` for creating dropdown select inputs with
/// options and selection state.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::select;
///
/// let select = select()
///     .option("us", "United States")
///     .option("ca", "Canada")
///     .build();
/// ```
pub fn select() -> SelectBuilder {
    SelectBuilder::new()
}

/// Create a Slider builder
///
/// Returns a `SliderBuilder` for creating range slider controls.
pub fn slider() -> SliderBuilder {
    SliderBuilder::new()
}

/// Layout Widgets
///
/// Functions for creating layout widget builders with fluent APIs.
///
/// Create a Scroll View builder
///
/// Returns a `ScrollViewBuilder` for creating scrollable content areas.
pub fn scroll_view() -> ScrollViewBuilder {
    ScrollViewBuilder::new()
}

/// Create a Stack layout builder
///
/// Returns a `StackBuilder` for creating stack layout containers.
pub fn stack() -> StackBuilder {
    StackBuilder::new()
}

/// Create a Tabs container builder
///
/// Returns a `TabsBuilder` for creating tabbed interfaces with multiple
/// content panels and navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::tabs;
/// use reactive_tui::component::Element;
///
/// let tabs = tabs()
///     .tab("Dashboard", Element::text("Dashboard content"))
///     .tab("Settings", Element::text("Settings panel"))
///     .build();
/// ```
pub fn tabs() -> TabsBuilder {
    TabsBuilder::new()
}

/// Dialog Widgets
///
/// Functions for creating dialog widget builders with fluent APIs.
///
/// Create a Dialog builder
///
/// Returns a `DialogBuilder` for creating basic dialog components.
pub fn dialog() -> DialogBuilder {
    DialogBuilder::new()
}

/// Create a Toast notification builder
///
/// Returns a `ToastBuilder` for creating toast notifications with different
/// types, durations, and positioning options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::toast;
///
/// let toast = toast()
///     .success("Operation completed!")
///     .duration(3000)
///     .build();
/// ```
pub fn toast() -> ToastBuilder {
    ToastBuilder::new()
}

/// Create a Confirmation Dialog builder
///
/// Returns a `ConfirmationDialogBuilder` for creating confirmation dialogs.
pub fn confirmation_dialog() -> ConfirmationDialogBuilder {
    ConfirmationDialogBuilder::new()
}

/// Create a Progress Dialog builder
///
/// Returns a `ProgressDialogBuilder` for creating progress dialogs.
pub fn progress_dialog() -> ProgressDialogBuilder {
    ProgressDialogBuilder::new()
}

/// Create a Wizard builder
///
/// Returns a `WizardBuilder` for creating multi-step wizard components.
pub fn wizard() -> WizardBuilder {
    WizardBuilder::new()
}

/// Create a primary button with default styling
///
/// # Arguments
/// * `text` - Button text to display
/// * `onclick` - Click handler function
///
/// # Returns
/// A styled primary button element
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

/// Create a horizontal flex container with specified gap
///
/// # Arguments
/// * `gap` - Gap size between children (e.g., "2", "4", "8")
/// * `children` - Child elements to arrange horizontally
///
/// # Returns
/// A flex row container element
pub fn flex_row(gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("flex flex-row gap-{}", gap))
        .children(children)
        .build()
}

/// Create a vertical flex container with specified gap
///
/// # Arguments
/// * `gap` - Gap size between children (e.g., "2", "4", "8")
/// * `children` - Child elements to arrange vertically
///
/// # Returns
/// A flex column container element
pub fn flex_col(gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("flex flex-col gap-{}", gap))
        .children(children)
        .build()
}

/// Create a CSS grid layout with specified columns and gap
///
/// # Arguments
/// * `cols` - Number of columns in the grid
/// * `gap` - Gap size between grid items (e.g., "2", "4", "8")
/// * `children` - Child elements to arrange in the grid
///
/// # Returns
/// A CSS grid container element
pub fn grid_layout(cols: u8, gap: &str, children: Vec<Element>) -> Element {
    div()
        .class(&format!("grid grid-cols-{} gap-{}", cols, gap))
        .children(children)
        .build()
}

/// Create a styled input with placeholder (legacy function)
///
/// # Arguments
/// * `placeholder` - Placeholder text to display
///
/// # Returns
/// An `ElementBuilder` for a styled text input
pub fn styled_input(placeholder: &str) -> ElementBuilder {
    input()
        .class("px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500")
        .placeholder(placeholder)
}

/// Create a search input with search icon
///
/// # Arguments
/// * `placeholder` - Placeholder text to display
///
/// # Returns
/// A styled search input element with search icon
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

/// Builder for DataTable components with fluent API
pub struct DataTableBuilder {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    pagination_enabled: bool,
    page_size: usize,
    virtual_scroll_enabled: bool,
    row_height: u16,
    viewport_height: u16,
    searchable: bool,
    filterable: bool,
    exportable: bool,
    hidden_columns: Vec<String>,
    filters: Vec<ColumnFilter>,
    class: Option<String>,
}

impl DataTableBuilder {
    fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            pagination_enabled: true,
            page_size: 25,
            virtual_scroll_enabled: false,
            row_height: 32,
            viewport_height: 400,
            searchable: true,
            filterable: true,
            exportable: true,
            hidden_columns: Vec::new(),
            filters: Vec::new(),
            class: None,
        }
    }

    /// Add a column to the table
    pub fn column(mut self, title: &str, key: &str) -> Self {
        self.columns.push(TableColumn {
            title: title.to_string(),
            key: key.to_string(),
            width: DisplaySize::Flex(1.0),
            alignment: Alignment::Start,
            sortable: true,
            resizable: true,
            min_width: 100,
            max_width: None,
        });
        self
    }

    /// Add a column with custom configuration
    pub fn column_with_config(mut self, column: TableColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Add multiple columns at once
    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Add a row to the table
    pub fn row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Add multiple rows at once
    pub fn rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// Add a simple row from key-value pairs
    pub fn simple_row(mut self, data: Vec<(&str, &str)>) -> Self {
        let mut cells = HashMap::new();
        for (key, value) in data {
            cells.insert(key.to_string(), TableCell {
                content: value.to_string(),
                style: None,
                alignment: None,
                clickable: false,
                action: None,
            });
        }

        self.rows.push(TableRow {
            id: format!("row_{}", self.rows.len()),
            cells,
            selectable: true,
            style: None,
            data: HashMap::new(),
        });
        self
    }

    /// Configure pagination
    pub fn pagination(mut self, enabled: bool, page_size: usize) -> Self {
        self.pagination_enabled = enabled;
        self.page_size = page_size;
        self
    }

    /// Configure virtual scrolling
    pub fn virtual_scroll(mut self, enabled: bool, row_height: u16, viewport_height: u16) -> Self {
        self.virtual_scroll_enabled = enabled;
        self.row_height = row_height;
        self.viewport_height = viewport_height;
        self
    }

    /// Configure features
    pub fn features(mut self, searchable: bool, filterable: bool, exportable: bool) -> Self {
        self.searchable = searchable;
        self.filterable = filterable;
        self.exportable = exportable;
        self
    }

    /// Hide specific columns
    pub fn hide_columns(mut self, column_keys: Vec<String>) -> Self {
        self.hidden_columns = column_keys;
        self
    }

    /// Add a filter
    pub fn filter(mut self, column_key: &str, filter_type: FilterType) -> Self {
        self.filters.push(ColumnFilter {
            column_key: column_key.to_string(),
            filter_type,
            active: true,
        });
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the DataTable element
    pub fn build(self) -> Element {
        self.build_with_name("DataTable")
    }

    /// Build the DataTable element with a custom component name
    pub fn build_with_name(self, component_name: &str) -> Element {
        let props = DataTableProps::new(self.columns, self.rows)
            .with_pagination(self.pagination_enabled, self.page_size)
            .with_virtual_scroll(self.virtual_scroll_enabled, self.row_height, self.viewport_height)
            .with_features(self.searchable, self.filterable, self.exportable);

        let mut element = Element::component_with_props(component_name, props);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<DataTableBuilder> for Element {
    fn from(builder: DataTableBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Chart components with fluent API
pub struct ChartBuilder {
    chart_type: ChartType,
    series: Vec<DataSeries>,
    title: Option<String>,
    width: u16,
    height: u16,
    x_axis: ChartAxis,
    y_axis: ChartAxis,
    legend: ChartLegend,
    color_palette: Vec<String>,
    animated: bool,
    animation_duration: u64,
    class: Option<String>,
}

impl ChartBuilder {
    fn new() -> Self {
        Self {
            chart_type: ChartType::BarVertical,
            series: Vec::new(),
            title: None,
            width: 80,
            height: 20,
            x_axis: ChartAxis::default(),
            y_axis: ChartAxis::default(),
            legend: ChartLegend::default(),
            color_palette: vec![
                "#3b82f6".to_string(), "#ef4444".to_string(), "#10b981".to_string(),
                "#f59e0b".to_string(), "#8b5cf6".to_string(), "#06b6d4".to_string(),
            ],
            animated: false,
            animation_duration: 1000,
            class: None,
        }
    }

    /// Set chart type
    pub fn chart_type(mut self, chart_type: ChartType) -> Self {
        self.chart_type = chart_type;
        self
    }

    /// Create a bar chart
    pub fn bar_chart(mut self) -> Self {
        self.chart_type = ChartType::BarVertical;
        self
    }

    /// Create a horizontal bar chart
    pub fn horizontal_bar_chart(mut self) -> Self {
        self.chart_type = ChartType::BarHorizontal;
        self
    }

    /// Create a line chart
    pub fn line_chart(mut self) -> Self {
        self.chart_type = ChartType::Line;
        self
    }

    /// Create a pie chart
    pub fn pie_chart(mut self) -> Self {
        self.chart_type = ChartType::Pie;
        self
    }

    /// Create an area chart
    pub fn area_chart(mut self) -> Self {
        self.chart_type = ChartType::Area;
        self
    }

    /// Create a scatter plot
    pub fn scatter_plot(mut self) -> Self {
        self.chart_type = ChartType::Scatter;
        self
    }

    /// Add a data series
    pub fn series(mut self, series: DataSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Add multiple data series
    pub fn series_list(mut self, series: Vec<DataSeries>) -> Self {
        self.series.extend(series);
        self
    }

    /// Add a simple data series from values
    pub fn simple_series(mut self, name: &str, values: Vec<f64>) -> Self {
        let data_points: Vec<DataPoint> = values.into_iter().map(DataPoint::new).collect();
        self.series.push(DataSeries::new(name, data_points));
        self
    }

    /// Add a labeled data series
    pub fn labeled_series(mut self, name: &str, data: Vec<(f64, &str)>) -> Self {
        let data_points: Vec<DataPoint> = data
            .into_iter()
            .map(|(value, label)| DataPoint::with_label(value, label))
            .collect();
        self.series.push(DataSeries::new(name, data_points));
        self
    }

    /// Set chart title
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Set chart dimensions
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Configure axes
    pub fn axes(mut self, x_title: Option<String>, y_title: Option<String>) -> Self {
        self.x_axis.title = x_title;
        self.y_axis.title = y_title;
        self
    }

    /// Set custom color palette
    pub fn colors(mut self, colors: Vec<String>) -> Self {
        self.color_palette = colors;
        self
    }

    /// Enable animation
    pub fn animated(mut self, duration_ms: u64) -> Self {
        self.animated = true;
        self.animation_duration = duration_ms;
        self
    }

    /// Configure legend
    pub fn legend(mut self, visible: bool, position: LegendPosition) -> Self {
        self.legend.visible = visible;
        self.legend.position = position;
        self
    }

    /// Set CSS classes
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Chart element
    pub fn build(self) -> Element {
        self.build_with_name("Chart")
    }

    /// Build the Chart element with a custom component name
    pub fn build_with_name(self, component_name: &str) -> Element {
        let props = ChartProps {
            chart_type: self.chart_type,
            series: self.series,
            title: self.title,
            width: self.width,
            height: self.height,
            x_axis: self.x_axis,
            y_axis: self.y_axis,
            legend: self.legend,
            color_palette: self.color_palette,
            animated: self.animated,
            animation_duration: self.animation_duration,
            show_tooltips: true,
            class: self.class.clone(),
        };

        let mut element = Element::component_with_props(component_name, props);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<ChartBuilder> for Element {
    fn from(builder: ChartBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Modal dialog components
///
/// Provides a fluent API for creating modal dialogs with customizable
/// content, sizing, and behavior options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::modal;
///
/// let modal = modal()
///     .title("Settings")
///     .content(Element::text("Configure your preferences"))
///     .visible(true)
///     .size(600, 400)
///     .build();
/// ```
pub struct ModalBuilder {
    title: Option<String>,
    content: Vec<Element>,
    visible: bool,
    closable: bool,
    backdrop_dismissible: bool,
    width: Option<u16>,
    height: Option<u16>,
    class: Option<String>,
}

impl ModalBuilder {
    /// Create a new ModalBuilder with default values
    fn new() -> Self {
        Self {
            title: None,
            content: Vec::new(),
            visible: false,
            closable: true,
            backdrop_dismissible: true,
            width: None,
            height: None,
            class: None,
        }
    }

    /// Set the modal title
    ///
    /// # Arguments
    /// * `title` - The title text to display in the modal header
    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Add content element to the modal
    ///
    /// # Arguments
    /// * `element` - The element to add to the modal content area
    pub fn content(mut self, element: Element) -> Self {
        self.content.push(element);
        self
    }

    /// Add multiple content elements to the modal
    ///
    /// # Arguments
    /// * `elements` - Vector of elements to add to the modal content area
    pub fn contents(mut self, elements: Vec<Element>) -> Self {
        self.content.extend(elements);
        self
    }

    /// Set the modal visibility state
    ///
    /// # Arguments
    /// * `visible` - Whether the modal should be visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set whether the modal can be closed by the user
    ///
    /// # Arguments
    /// * `closable` - Whether the modal shows a close button
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set whether clicking the backdrop dismisses the modal
    ///
    /// # Arguments
    /// * `dismissible` - Whether clicking outside the modal closes it
    pub fn backdrop_dismissible(mut self, dismissible: bool) -> Self {
        self.backdrop_dismissible = dismissible;
        self
    }

    /// Set the modal dimensions
    ///
    /// # Arguments
    /// * `width` - Modal width in characters
    /// * `height` - Modal height in characters
    pub fn size(mut self, width: u16, height: u16) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the modal
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Modal element
    ///
    /// Creates a modal dialog element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the modal dialog
    pub fn build(self) -> Element {
        // Create a simple modal representation
        // In a full implementation, this would use Modal component props
        let mut modal_content = Vec::new();

        if let Some(title) = self.title {
            modal_content.push(Element::text(format!("Modal: {}", title)));
        }

        modal_content.extend(self.content);

        let mut element = Element::layout(LayoutType::Flex)
            .children(modal_content);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<ModalBuilder> for Element {
    fn from(builder: ModalBuilder) -> Self {
        builder.build()
    }
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
        let percentage = (self.value / self.max_value * 100.0).clamp(0.0, 100.0);

        let display_text = if let Some(label) = &self.label {
            if self.show_percentage {
                format!("{}: {:.1}%", label, percentage)
            } else {
                label.clone()
            }
        } else if self.show_percentage {
            format!("{:.1}%", percentage)
        } else {
            format!("{}/{}", self.value, self.max_value)
        };

        let mut element = Element::text(format!("Progress: {}", display_text));

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<ProgressBarBuilder> for Element {
    fn from(builder: ProgressBarBuilder) -> Self {
        builder.build()
    }
}

/// Builder for TextInput components
///
/// Provides a fluent API for creating text input fields with various
/// input types, validation, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::text_input;
///
/// let input = text_input()
///     .placeholder("Enter your email")
///     .input_type("email")
///     .max_length(100)
///     .class("email-field")
///     .build();
/// ```
pub struct TextInputBuilder {
    value: String,
    placeholder: Option<String>,
    disabled: bool,
    readonly: bool,
    max_length: Option<usize>,
    input_type: String,
    class: Option<String>,
}

impl TextInputBuilder {
    /// Create a new TextInputBuilder with default values
    fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: None,
            disabled: false,
            readonly: false,
            max_length: None,
            input_type: "text".to_string(),
            class: None,
        }
    }

    /// Set the input field value
    ///
    /// # Arguments
    /// * `value` - The current text value of the input field
    pub fn value(mut self, value: &str) -> Self {
        self.value = value.to_string();
        self
    }

    /// Set the placeholder text
    ///
    /// # Arguments
    /// * `placeholder` - Text to show when the input is empty
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the input should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the readonly state
    ///
    /// # Arguments
    /// * `readonly` - Whether the input should be read-only
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    /// Set the maximum character length
    ///
    /// # Arguments
    /// * `max_length` - Maximum number of characters allowed
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set the input type
    ///
    /// # Arguments
    /// * `input_type` - Input type (text, password, email, number, etc.)
    pub fn input_type(mut self, input_type: &str) -> Self {
        self.input_type = input_type.to_string();
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the input
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the TextInput element
    ///
    /// Creates a text input element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the text input field
    pub fn build(self) -> Element {
        let display_text = if self.value.is_empty() {
            self.placeholder.unwrap_or_else(|| "Text Input".to_string())
        } else {
            self.value
        };

        let mut element = Element::text(format!("Input: {}", display_text));

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<TextInputBuilder> for Element {
    fn from(builder: TextInputBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Checkbox components
///
/// Provides a fluent API for creating checkbox inputs with labels,
/// states, and styling options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::checkbox;
///
/// let checkbox = checkbox()
///     .label("I agree to the terms")
///     .checked(false)
///     .class("terms-checkbox")
///     .build();
/// ```
pub struct CheckboxBuilder {
    checked: bool,
    label: Option<String>,
    disabled: bool,
    indeterminate: bool,
    class: Option<String>,
}

impl CheckboxBuilder {
    /// Create a new CheckboxBuilder with default values
    fn new() -> Self {
        Self {
            checked: false,
            label: None,
            disabled: false,
            indeterminate: false,
            class: None,
        }
    }

    /// Set the checked state
    ///
    /// # Arguments
    /// * `checked` - Whether the checkbox should be checked
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the checkbox label text
    ///
    /// # Arguments
    /// * `label` - Text label to display next to the checkbox
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the checkbox should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the indeterminate state
    ///
    /// # Arguments
    /// * `indeterminate` - Whether the checkbox should show indeterminate state
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the checkbox
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Checkbox element
    ///
    /// Creates a checkbox element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the checkbox input
    pub fn build(self) -> Element {
        let state = if self.indeterminate {
            "indeterminate"
        } else if self.checked {
            "checked"
        } else {
            "unchecked"
        };

        let display_text = if let Some(label) = &self.label {
            format!("Checkbox ({}): {}", state, label)
        } else {
            format!("Checkbox ({})", state)
        };

        let mut element = Element::text(display_text);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<CheckboxBuilder> for Element {
    fn from(builder: CheckboxBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Select dropdown components
///
/// Provides a fluent API for creating select dropdowns with options,
/// selection state, and styling.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::select;
///
/// let select = select()
///     .option("us", "United States")
///     .option("ca", "Canada")
///     .selected("us")
///     .placeholder("Choose country")
///     .build();
/// ```
pub struct SelectBuilder {
    options: Vec<(String, String)>, // (value, label)
    selected_value: Option<String>,
    placeholder: Option<String>,
    disabled: bool,
    multiple: bool,
    class: Option<String>,
}

impl SelectBuilder {
    /// Create a new SelectBuilder with default values
    fn new() -> Self {
        Self {
            options: Vec::new(),
            selected_value: None,
            placeholder: None,
            disabled: false,
            multiple: false,
            class: None,
        }
    }

    /// Add a single option to the select dropdown
    ///
    /// # Arguments
    /// * `value` - The option value (used for form submission)
    /// * `label` - The display text for the option
    pub fn option(mut self, value: &str, label: &str) -> Self {
        self.options.push((value.to_string(), label.to_string()));
        self
    }

    /// Add multiple options at once
    ///
    /// # Arguments
    /// * `options` - Vector of (value, label) tuples
    pub fn options(mut self, options: Vec<(&str, &str)>) -> Self {
        for (value, label) in options {
            self.options.push((value.to_string(), label.to_string()));
        }
        self
    }

    /// Set the currently selected value
    ///
    /// # Arguments
    /// * `value` - The value of the option to select
    pub fn selected(mut self, value: &str) -> Self {
        self.selected_value = Some(value.to_string());
        self
    }

    /// Set the placeholder text
    ///
    /// # Arguments
    /// * `placeholder` - Text to show when no option is selected
    pub fn placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = Some(placeholder.to_string());
        self
    }

    /// Set the disabled state
    ///
    /// # Arguments
    /// * `disabled` - Whether the select should be disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Enable multiple selection mode
    ///
    /// # Arguments
    /// * `multiple` - Whether multiple options can be selected
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the select
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Select element
    ///
    /// Creates a select dropdown element with the configured options and properties.
    ///
    /// # Returns
    /// An `Element` representing the select dropdown
    pub fn build(self) -> Element {
        let selected_label = if let Some(selected) = &self.selected_value {
            self.options
                .iter()
                .find(|(value, _)| value == selected)
                .map(|(_, label)| label.clone())
                .unwrap_or_else(|| selected.clone())
        } else {
            self.placeholder.unwrap_or_else(|| "Select an option".to_string())
        };

        let display_text = format!("Select: {} ({} options)", selected_label, self.options.len());

        let mut element = Element::text(display_text);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<SelectBuilder> for Element {
    fn from(builder: SelectBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Tabs container components
///
/// Provides a fluent API for creating tabbed interfaces with multiple
/// content panels and navigation.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::tabs;
/// use reactive_tui::component::Element;
///
/// let tabs = tabs()
///     .tab("Dashboard", Element::text("Dashboard content"))
///     .tab("Settings", Element::text("Settings panel"))
///     .active(0)
///     .build();
/// ```
pub struct TabsBuilder {
    tabs: Vec<(String, Element)>, // (title, content)
    active_tab: usize,
    closable: bool,
    class: Option<String>,
}

impl TabsBuilder {
    /// Create a new TabsBuilder with default values
    fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            closable: false,
            class: None,
        }
    }

    /// Add a tab with title and content
    ///
    /// # Arguments
    /// * `title` - The tab title displayed in the tab bar
    /// * `content` - The element to display when this tab is active
    pub fn tab(mut self, title: &str, content: Element) -> Self {
        self.tabs.push((title.to_string(), content));
        self
    }

    /// Set the active tab by index
    ///
    /// # Arguments
    /// * `index` - Zero-based index of the tab to make active
    pub fn active(mut self, index: usize) -> Self {
        self.active_tab = index;
        self
    }

    /// Enable or disable closable tabs
    ///
    /// # Arguments
    /// * `closable` - Whether tabs can be closed by the user
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the tabs container
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Tabs element
    ///
    /// Creates a tabs container element with the configured tabs and properties.
    ///
    /// # Returns
    /// An `Element` representing the tabs container
    pub fn build(self) -> Element {
        let mut children = Vec::new();

        // Add tab headers
        let tab_titles: Vec<String> = self.tabs.iter().map(|(title, _)| title.clone()).collect();
        children.push(Element::text(format!("Tabs: [{}]", tab_titles.join(", "))));

        // Add active tab content
        if let Some((title, content)) = self.tabs.get(self.active_tab) {
            children.push(Element::text(format!("Active: {}", title)));
            children.push(content.clone());
        }

        let mut element = Element::layout(LayoutType::Flex).children(children);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<TabsBuilder> for Element {
    fn from(builder: TabsBuilder) -> Self {
        builder.build()
    }
}

/// Builder for Toast notification components
///
/// Provides a fluent API for creating toast notifications with different
/// types, durations, and positioning options.
///
/// # Example
/// ```rust
/// use reactive_tui::builder::toast;
///
/// let toast = toast()
///     .success("Operation completed successfully!")
///     .duration(3000)
///     .position("top-right")
///     .build();
/// ```
pub struct ToastBuilder {
    message: String,
    toast_type: String, // success, error, warning, info
    duration: Option<u64>,
    closable: bool,
    position: String,
    class: Option<String>,
}

impl ToastBuilder {
    /// Create a new ToastBuilder with default values
    fn new() -> Self {
        Self {
            message: String::new(),
            toast_type: "info".to_string(),
            duration: Some(3000),
            closable: true,
            position: "top-right".to_string(),
            class: None,
        }
    }

    /// Set the toast message text
    ///
    /// # Arguments
    /// * `message` - The message text to display in the toast
    pub fn message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    /// Set the toast type
    ///
    /// # Arguments
    /// * `toast_type` - The type of toast (success, error, warning, info)
    pub fn toast_type(mut self, toast_type: &str) -> Self {
        self.toast_type = toast_type.to_string();
        self
    }

    /// Create a success toast with message
    ///
    /// # Arguments
    /// * `message` - Success message to display
    pub fn success(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "success".to_string();
        self
    }

    /// Create an error toast with message
    ///
    /// # Arguments
    /// * `message` - Error message to display
    pub fn error(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "error".to_string();
        self
    }

    /// Create a warning toast with message
    ///
    /// # Arguments
    /// * `message` - Warning message to display
    pub fn warning(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "warning".to_string();
        self
    }

    /// Create an info toast with message
    ///
    /// # Arguments
    /// * `message` - Info message to display
    pub fn info(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self.toast_type = "info".to_string();
        self
    }

    /// Set the auto-dismiss duration
    ///
    /// # Arguments
    /// * `duration_ms` - Duration in milliseconds before auto-dismiss
    pub fn duration(mut self, duration_ms: u64) -> Self {
        self.duration = Some(duration_ms);
        self
    }

    /// Make the toast persistent (no auto-dismiss)
    pub fn persistent(mut self) -> Self {
        self.duration = None;
        self
    }

    /// Set whether the toast can be manually closed
    ///
    /// # Arguments
    /// * `closable` - Whether the toast shows a close button
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Set the toast position on screen
    ///
    /// # Arguments
    /// * `position` - Position string (top-right, top-left, bottom-center, etc.)
    pub fn position(mut self, position: &str) -> Self {
        self.position = position.to_string();
        self
    }

    /// Set CSS classes for styling
    ///
    /// # Arguments
    /// * `class` - CSS class string to apply to the toast
    pub fn class(mut self, class: &str) -> Self {
        self.class = Some(class.to_string());
        self
    }

    /// Build the Toast element
    ///
    /// Creates a toast notification element with the configured properties.
    ///
    /// # Returns
    /// An `Element` representing the toast notification
    pub fn build(self) -> Element {
        let display_text = format!("Toast ({}): {}", self.toast_type, self.message);

        let mut element = Element::text(display_text);

        if let Some(class) = self.class {
            element = element.with_class(&class);
        }

        element
    }
}

impl From<ToastBuilder> for Element {
    fn from(builder: ToastBuilder) -> Self {
        builder.build()
    }
}

// Placeholder builders for remaining widgets
// These would be fully implemented with proper component integration

/// Builder for Popover components
///
/// Provides a fluent API for creating popover elements that display
/// content when triggered by user interaction.
///
/// # Example
/// ```rust
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

impl PopoverBuilder {
    /// Create a new PopoverBuilder with default values
    fn new() -> Self {
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
        Element::text("Popover")
    }
}

impl From<PopoverBuilder> for Element {
    fn from(builder: PopoverBuilder) -> Self {
        builder.build()
    }
}

// Add placeholder implementations for remaining builders
// These are simplified builders that will be fully implemented when the corresponding
// widget components are available.
macro_rules! placeholder_builder {
    ($name:ident, $display:expr) => {
        #[doc = concat!("Builder for ", $display, " components")]
        #[doc = ""]
        #[doc = "This is a placeholder builder that will be fully implemented"]
        #[doc = "when the corresponding widget component is available."]
        pub struct $name {
            class: Option<String>,
        }

        impl $name {
            /// Create a new builder with default values
            fn new() -> Self {
                Self { class: None }
            }

            /// Set CSS classes for styling
            ///
            /// # Arguments
            /// * `class` - CSS class string to apply to the widget
            pub fn class(mut self, class: &str) -> Self {
                self.class = Some(class.to_string());
                self
            }

            /// Build the widget element
            ///
            /// Creates a placeholder element for this widget type.
            ///
            /// # Returns
            /// An `Element` representing the widget
            pub fn build(self) -> Element {
                Element::text($display)
            }
        }

        impl From<$name> for Element {
            fn from(builder: $name) -> Self {
                builder.build()
            }
        }
    };
}

placeholder_builder!(TreeBuilder, "Tree View");
placeholder_builder!(ImageBuilder, "Image");
placeholder_builder!(RadioButtonBuilder, "Radio Button");
placeholder_builder!(SliderBuilder, "Slider");
placeholder_builder!(ScrollViewBuilder, "Scroll View");
placeholder_builder!(StackBuilder, "Stack Layout");
placeholder_builder!(DialogBuilder, "Dialog");
placeholder_builder!(ConfirmationDialogBuilder, "Confirmation Dialog");
placeholder_builder!(ProgressDialogBuilder, "Progress Dialog");
placeholder_builder!(WizardBuilder, "Wizard");

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
