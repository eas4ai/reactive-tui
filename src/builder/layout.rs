//! Layout helpers and convenience functions for common UI patterns
//!
//! This module provides pre-styled components and layout utilities that make it easy
//! to create common UI patterns with consistent styling.

use super::core::{aside, button, div, grid_builder, input, main, span, ElementBuilder};
use crate::component::Element;

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

/// Create a sidebar layout
pub fn sidebar() -> ElementBuilder {
    aside().class("w-64 bg-gray-800 text-white")
}

/// Create a content area
pub fn content() -> ElementBuilder {
    main().class("flex-1 p-0.5")
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

/// Component shortcuts using existing utility CSS system
///
pub fn card(children: Vec<Element>) -> Element {
    div()
        .class("bg-white rounded-lg shadow-md border border-gray-200 p-0.5")
        .children(children)
        .build()
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
    F: Fn() + Send + Sync + 'static,
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
                .class("pl-1 pr-0.5 py-0.25 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500")
                .placeholder(placeholder)
                .build(),
            div()
                .class("absolute left-1 top-1 text-gray-400")
                .text("🔍")
                .build(),
        ])
        .build()
}
