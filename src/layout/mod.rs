//! Layout engine for terminal UI components
//!
//! This module provides a CSS-like layout system built on the Taffy layout engine,
//! enabling flexbox and grid layouts with terminal-specific optimizations.

use crate::error::{ReactiveError, Result};
use taffy::{geometry::Size, prelude::NodeId, style::Style, AvailableSpace, TaffyTree};

/// Color parsing and management for terminal UI
pub mod colors;
/// CSS-like utility class parsing and application
pub mod css;
/// Grid layout utilities and helpers
pub mod grid;
/// Layout manager for persistent Taffy tree and incremental updates
pub mod manager;
pub(crate) mod motion;
/// Paint tree for rendering layout results to terminal
pub mod paint_tree;
pub use paint_tree::cells::CellGrid;
/// Layout renderer for painting to surfaces
pub mod renderer;
/// Style builder and management utilities
pub mod style;
pub(crate) mod text;

/// Main layout engine that manages the layout tree
///
/// The `LayoutEngine` wraps the Taffy layout tree and provides
/// a simplified interface for terminal UI layout computation.
pub struct LayoutEngine {
    /// The underlying Taffy layout tree
    tree: TaffyTree<()>,
    /// Root node of the layout tree
    root: NodeId,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    /// Create a new layout engine with a default root node
    ///
    /// A fresh tree always has room for its first node, so this cannot
    /// fail in practice; use `try_new` when even that needs proving.
    ///
    /// # Returns
    /// A new `LayoutEngine` with an empty layout tree
    pub fn new() -> Self {
        Self::try_new().expect("fresh layout tree accepts its root node")
    }

    /// Fallible construction, mapping allocator failures into
    /// `ReactiveError` like every other tree operation here.
    ///
    /// # Returns
    /// A new `LayoutEngine`, or an error if the root node cannot be created
    pub fn try_new() -> Result<Self> {
        let mut tree = TaffyTree::new();
        let root = tree
            .new_leaf_with_context(Style::default(), ())
            .map_err(|e| ReactiveError::layout(format!("Failed to create root: {e}")))?;
        Ok(Self { tree, root })
    }

    /// Get the root node ID of the layout tree
    ///
    /// # Returns
    /// The `NodeId` of the root node
    pub fn root(&self) -> NodeId {
        self.root
    }
    /// Set the style for a specific node
    ///
    /// # Arguments
    /// * `node` - The node to update
    /// * `style` - The new style to apply
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the node doesn't exist
    pub fn set_style(&mut self, node: NodeId, style: Style) -> Result<()> {
        self.tree
            .set_style(node, style)
            .map_err(|e| ReactiveError::layout(format!("Failed to set style: {}", e)))
    }
    /// Set the children for a parent node
    ///
    /// # Arguments
    /// * `parent` - The parent node
    /// * `children` - Array of child node IDs
    ///
    /// # Returns
    /// `Ok(())` on success, or an error if the operation fails
    pub fn set_children(&mut self, parent: NodeId, children: &[NodeId]) -> Result<()> {
        self.tree
            .set_children(parent, children)
            .map_err(|e| ReactiveError::layout(format!("Failed to set children: {}", e)))
    }
    /// Create a new leaf node with the given style
    ///
    /// # Arguments
    /// * `style` - The style for the new node
    ///
    /// # Returns
    /// The `NodeId` of the newly created node, or an error
    pub fn new_leaf(&mut self, style: Style) -> Result<NodeId> {
        self.tree
            .new_leaf(style)
            .map_err(|e| ReactiveError::layout(format!("Failed to create leaf: {}", e)))
    }
    /// Compute the layout for the entire tree
    ///
    /// # Arguments
    /// * `width` - Optional container width constraint in terminal cells
    ///
    /// # Returns
    /// `Ok(())` on successful layout computation, or an error
    pub fn compute(&mut self, width: Option<f32>) -> Result<()> {
        let size = Size {
            width: width
                .map(AvailableSpace::Definite)
                .unwrap_or(AvailableSpace::MaxContent),
            height: AvailableSpace::MaxContent,
        };
        self.tree
            .compute_layout(self.root, size)
            .map_err(|e| ReactiveError::layout(format!("Failed to compute layout: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallible_construction_yields_a_usable_root() {
        let engine = LayoutEngine::try_new().expect("fresh tree");
        // The root accepts style updates, proving it is a live node.
        let mut engine = engine;
        let root = engine.root();
        let updated = engine.set_style(root, Style::default());
        assert!(updated.is_ok(), "root style: {updated:?}");
    }
}
