//! Grid Renderer Integration
//!
//! Bridges the declarative grid system with the existing visual renderer

use crate::component::{Element, ElementType};
use crate::core::surface::Surface;
use crate::error::Result;
use crate::layout::grid::{DeclarativeGrid, GridArea, GridChild};
use crate::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
use std::borrow::Cow;

/// Extract text content from an Element and its children
fn extract_element_text(element: &Element) -> Option<String> {
    match &element.element_type {
        ElementType::Text(text) => Some(text.clone()),
        ElementType::Component(component_name) => {
            // For components, try to extract from children or use component name
            let child_text = extract_children_text(element);
            if child_text.is_some() {
                child_text
            } else {
                // Check if it's a known interactive component
                match component_name.as_str() {
                    "Input" | "TextInput" => {
                        // For input components, we can't easily access props due to type erasure
                        // In a full implementation, we'd need a trait for extracting display text
                        // For now, provide a reasonable default
                        Some("[Input]".to_string())
                    }
                    "Button" => Some("[Button]".to_string()),
                    _ => Some(format!("[{}]", component_name)),
                }
            }
        }
        ElementType::Layout(layout_type) => {
            // For layout containers, extract all child text
            let child_text = extract_children_text(element);
            if child_text.is_some() {
                child_text
            } else {
                Some(format!("[{:?}]", layout_type))
            }
        }
        ElementType::Fragment => {
            // For fragments, just extract child text
            extract_children_text(element)
        }
        ElementType::Empty => {
            // Empty elements have no text
            None
        }
    }
}

/// Extract text from all children of an element
fn extract_children_text(element: &Element) -> Option<String> {
    if element.children.is_empty() {
        return None;
    }

    let mut texts = Vec::new();
    for child in &element.children {
        if let Some(text) = extract_element_text(child) {
            texts.push(text);
        }
    }

    if texts.is_empty() {
        None
    } else {
        Some(texts.join(" "))
    }
}

/// Convert a DeclarativeGrid into a visual NodeSpec tree for rendering
pub fn grid_to_node_spec(grid: &DeclarativeGrid) -> NodeSpec<'static> {
    // Create the main grid container with proper sizing
    let mut grid_classes = vec!["grid".to_string()];

    // Add grid template classes
    grid_classes.push(format!("grid-cols-{}", grid.cols));
    grid_classes.push(format!("grid-rows-{}", grid.rows));

    if grid.gap > 0 {
        grid_classes.push(format!("gap-{}", grid.gap));
    } else {
        // Use individual gap settings if no general gap is set
        if grid.column_gap > 0 {
            grid_classes.push(format!("gap-x-{}", grid.column_gap));
        }
        if grid.row_gap > 0 {
            grid_classes.push(format!("gap-y-{}", grid.row_gap));
        }
    }

    // Make grid container responsive by default - fill entire terminal
    grid_classes.push("w-full".to_string()); // Fill available width (100vw equivalent)
    grid_classes.push("h-full".to_string()); // Fill available height (100vh equivalent)
    grid_classes.push("min-w-full".to_string()); // Ensure full width
    grid_classes.push("min-h-full".to_string()); // Ensure full height

    // Add custom CSS class if specified
    if let Some(ref css_class) = grid.css_class {
        grid_classes.push(css_class.clone());
    }

    let grid_class = grid_classes.join(" ");

    // Convert areas to child nodes, sorted by z-index
    let mut children = Vec::new();
    for area in grid.areas_by_z_index() {
        let child_node = area_to_node_spec(area);
        children.push(child_node);
    }

    // Also convert grid children if any
    for child in &grid.children {
        let child_node = grid_child_to_node_spec(child);
        children.push(child_node);
    }

    NodeSpec {
        class: Cow::Owned(grid_class),
        text: None,
        children,
    }
}

/// Convert a GridChild into a NodeSpec
fn grid_child_to_node_spec(child: &GridChild) -> NodeSpec<'static> {
    let mut classes = Vec::new();

    // Add grid positioning classes
    classes.push(format!("col-start-{}", child.column + 1));
    classes.push(format!("row-start-{}", child.row + 1));

    if child.column_span > 1 {
        classes.push(format!("col-span-{}", child.column_span));
    }
    if child.row_span > 1 {
        classes.push(format!("row-span-{}", child.row_span));
    }

    // Default styling for visual appearance
    classes.push("min-w-80".to_string());
    classes.push("min-h-40".to_string());
    classes.push("flex-1".to_string());
    classes.push("p-4".to_string());
    classes.push("flex".to_string());
    classes.push("items-center".to_string());
    classes.push("justify-center".to_string());
    classes.push("border".to_string());

    let class_string = classes.join(" ");

    // Extract text content from the element
    let text_content = extract_element_text(&child.element);

    NodeSpec {
        class: Cow::Owned(class_string),
        text: text_content.map(Cow::Owned),
        children: vec![],
    }
}

/// Convert a GridArea into a NodeSpec
fn area_to_node_spec(area: &GridArea) -> NodeSpec<'static> {
    let mut classes = Vec::new();

    // Add grid positioning classes
    classes.push(format!("col-start-{}", area.col + 1));
    classes.push(format!("row-start-{}", area.row + 1));

    if area.col_span > 1 {
        classes.push(format!("col-span-{}", area.col_span));
    }
    if area.row_span > 1 {
        classes.push(format!("row-span-{}", area.row_span));
    }

    // Add z-index if not default
    if area.z_index != 0 {
        classes.push(format!("z-{}", area.z_index));
    }

    // Add custom CSS classes
    if let Some(ref css_class) = area.css_class {
        classes.push(css_class.clone());
    }

    // Default styling for visual appearance - make cells responsive
    classes.push("min-w-80".to_string()); // Minimum width for visibility
    classes.push("min-h-40".to_string()); // Minimum height for visibility
    classes.push("flex-1".to_string()); // Grow to fill available space
    classes.push("p-4".to_string()); // Padding for better appearance
    classes.push("flex".to_string()); // Flexbox for centering
    classes.push("items-center".to_string()); // Center items vertically
    classes.push("justify-center".to_string()); // Center content horizontally
    classes.push("border".to_string()); // Add border for visual separation

    let class_string = classes.join(" ");

    NodeSpec {
        class: Cow::Owned(class_string),
        text: Some(Cow::Owned(area.name.clone())),
        children: vec![],
    }
}

/// Render a DeclarativeGrid to a Surface using the existing renderer
pub fn render_grid_to_surface(
    grid: &DeclarativeGrid,
    surface: &mut Surface,
    width: usize,
    options: Option<PaintOptions>,
) -> Result<()> {
    let node_spec = grid_to_node_spec(grid);
    let opts = options.unwrap_or_default();
    layout_and_paint_with(&node_spec, surface, width, &opts)
}

/// Render a DeclarativeGrid to a Surface with default options
pub fn render_grid(grid: &DeclarativeGrid, surface: &mut Surface, width: usize) -> Result<()> {
    // Convert grid to NodeSpec and render using the paint tree
    let node_spec = grid_to_node_spec(grid);
    let options = PaintOptions::default();
    layout_and_paint_with(&node_spec, surface, width, &options)
}

/// Create a visual demo of the grid system
pub fn create_demo_grid() -> DeclarativeGrid {
    crate::layout! {
        grid(cols: 3, rows: 2, gap: 1) {
            "First column" at (0, 0) class "bg-blue-100",
            "Two" at (0, 1) class "bg-green-100",
            "Three" at (0, 2) class "bg-red-100",
            "Four" at (1, 0) class "bg-yellow-100",
            "Five" at (1, 1) class "bg-purple-100",
            "Six" at (1, 2) class "bg-pink-100",
        }
    }
}

/// Create a layered demo with z-index
pub fn create_layered_demo() -> DeclarativeGrid {
    crate::layout! {
        grid(cols: 3, rows: 3, gap: 1) {
            "Background" at (0, 0) span (3, 3) class "bg-gray-200" z 0,
            "Card 1" at (0, 0) span (2, 2) class "bg-white shadow border-2" z 1,
            "Card 2" at (1, 1) span (2, 2) class "bg-blue-50 shadow border-2" z 2,
            "Tooltip" at (0, 2) class "bg-yellow-300 border-2 text-xs" z 10,
            "Modal" at (1, 1) class "bg-red-100 border-4 border-red-500" z 100,
        }
    }
}

/// Create a dashboard layout demo
pub fn create_dashboard_demo() -> DeclarativeGrid {
    crate::layout! {
        grid(cols: 4, rows: 4, gap: 1) {
            "Header" at (0, 0) span (1, 4) class "bg-blue-600 text-white" z 1,
            "Sidebar" at (1, 0) class "bg-gray-200" z 1,
            "Main Content" at (1, 1) span (2, 2) class "bg-white" z 1,
            "Notifications" at (1, 3) class "bg-yellow-100" z 1,
            "Footer" at (3, 0) span (1, 4) class "bg-gray-800 text-white" z 1,

            // Modal overlay
            "Modal Backdrop" at (0, 0) span (4, 4) class "bg-black bg-opacity-50" z 100,
            "Modal Dialog" at (1, 1) span (2, 2) class "bg-white rounded shadow-xl border-2" z 101,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::surface::Surface;

    #[test]
    fn test_grid_to_node_spec() {
        let grid = create_demo_grid();
        let node_spec = grid_to_node_spec(&grid);

        // Should have grid classes
        assert!(node_spec.class.contains("grid"));
        assert!(node_spec.class.contains("grid-cols-3"));
        assert!(node_spec.class.contains("grid-rows-2"));

        // Should have 6 children
        assert_eq!(node_spec.children.len(), 6);

        // First child should have proper positioning
        let first_child = &node_spec.children[0];
        assert!(first_child.class.contains("col-start-1"));
        assert!(first_child.class.contains("row-start-1"));
        assert_eq!(first_child.text.as_ref().unwrap(), "First column");
    }

    #[test]
    fn test_area_to_node_spec() {
        let area = crate::layout::grid::GridArea::new("Test")
            .at(1, 2)
            .span(2, 3)
            .class("custom-class")
            .z(10);

        let node_spec = area_to_node_spec(&area);
        let class_str = &node_spec.class;

        assert!(class_str.contains("col-start-3")); // col + 1
        assert!(class_str.contains("row-start-2")); // row + 1
        assert!(class_str.contains("col-span-3"));
        assert!(class_str.contains("row-span-2"));
        assert!(class_str.contains("z-10"));
        assert!(class_str.contains("custom-class"));
        assert!(class_str.contains("border"));
        assert_eq!(node_spec.text.as_ref().unwrap(), "Test");
    }

    #[test]
    fn test_render_grid() {
        let grid = create_demo_grid();
        let mut surface = Surface::new(80, 24);

        // Should not panic
        let result = render_grid(&grid, &mut surface, 80);
        assert!(result.is_ok());
    }

    #[test]
    fn test_layered_demo() {
        let grid = create_layered_demo();

        // Should have areas with different z-indices
        let areas = grid.areas_by_z_index();
        assert_eq!(areas[0].z_index, 0); // Background first
        assert_eq!(areas[areas.len() - 1].z_index, 100); // Modal last
    }
}
