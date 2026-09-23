//! Layout manager maintaining persistent Taffy tree for incremental updates

use crate::component::{Element, ElementType};
use crate::error::{ReactiveError, Result};
use std::collections::{HashMap, HashSet};
use taffy::{AvailableSpace, Display, FlexDirection, NodeId, Position, Size, Style, TaffyTree};

/// Key type for identifying nodes in the layout tree
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LayoutKey(String);

impl LayoutKey {
    /// Create a new layout key from a string
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl From<&str> for LayoutKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for LayoutKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Paint operation for rendering changes
#[derive(Debug, Clone)]
pub enum PaintOp {
    /// Clear a rectangular region
    Clear {
        /// Top-left x position in cells
        x: u16,
        /// Top-left y position in cells
        y: u16,
        /// Width in cells
        width: u16,
        /// Height in cells
        height: u16,
    },
    /// Draw text at position
    Text {
        /// Left position in cells
        x: u16,
        /// Top position in cells
        y: u16,
        /// Text content to render
        content: String,
        /// Styling to apply when rendering the text
        style: TextStyle,
    },
    /// Draw a box/border
    Box {
        /// Left position in cells
        x: u16,
        /// Top position in cells
        y: u16,
        /// Width in cells
        width: u16,
        /// Height in cells
        height: u16,
        /// Styling for the box/border
        style: BoxStyle,
    },
}

/// Text styling for paint operations
#[derive(Debug, Clone, Default)]
pub struct TextStyle {
    /// Optional foreground color as (r, g, b)
    pub fg: Option<(u8, u8, u8)>,
    /// Optional background color as (r, g, b)
    pub bg: Option<(u8, u8, u8)>,
    /// Render text in bold weight
    pub bold: bool,
    /// Render text in italic style
    pub italic: bool,
    /// Underline the text
    pub underline: bool,
}

/// Box styling for paint operations
#[derive(Debug, Clone, Default)]
pub struct BoxStyle {
    /// Optional border color as (r, g, b)
    pub border_color: Option<(u8, u8, u8)>,
    /// Optional background color as (r, g, b)
    pub bg: Option<(u8, u8, u8)>,
    /// Use a double-line border instead of a single-line border
    pub double_border: bool,
}

/// Node metadata for painting
struct NodeMeta {
    /// Source element this node represents
    element: Element,
    /// Last computed Taffy layout for this node
    layout: taffy::Layout,
    /// Whether this node/layout is out of date and needs recomputation
    dirty: bool,
}

/// Layout manager maintaining persistent Taffy tree
pub struct LayoutManager {
    /// The Taffy layout tree
    taffy: TaffyTree,
    /// Map from element keys to Taffy node IDs
    node_map: HashMap<LayoutKey, NodeId>,
    /// Reverse map from Taffy nodes to element keys
    reverse_map: HashMap<NodeId, LayoutKey>,
    /// Metadata for each node
    meta: HashMap<NodeId, NodeMeta>,
    /// Nodes that need layout recomputation
    dirty_nodes: HashSet<NodeId>,
    /// Root node ID if set
    root: Option<NodeId>,
    /// Available space for layout
    available_space: Size<AvailableSpace>,
    /// Paint operations generated from last layout
    paint_ops: Vec<PaintOp>,
}

impl LayoutManager {
    /// Create a new layout manager with given dimensions
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            taffy: TaffyTree::new(),
            node_map: HashMap::new(),
            reverse_map: HashMap::new(),
            meta: HashMap::new(),
            dirty_nodes: HashSet::new(),
            root: None,
            available_space: Size {
                width: AvailableSpace::Definite(width as f32),
                height: AvailableSpace::Definite(height as f32),
            },
            paint_ops: Vec::new(),
        }
    }

    /// Update available space for layout
    pub fn set_dimensions(&mut self, width: u16, height: u16) {
        self.available_space = Size {
            width: AvailableSpace::Definite(width as f32),
            height: AvailableSpace::Definite(height as f32),
        };
        // Mark root as dirty to trigger full relayout
        if let Some(root) = self.root {
            self.dirty_nodes.insert(root);
        }
    }

    /// Insert a new node into the layout tree
    pub fn insert_node(
        &mut self,
        parent: Option<LayoutKey>,
        index: usize,
        element: Element,
    ) -> Result<()> {
        let key = element.key.clone().ok_or_else(|| {
            ReactiveError::layout("Element must have a key for layout management")
        })?;
        let layout_key = LayoutKey::new(key.clone());

        // Convert element to Taffy style
        let style = self.element_to_style(&element);

        // Create new Taffy node
        let node_id = self
            .taffy
            .new_leaf(style)
            .map_err(|e| ReactiveError::layout(format!("Failed to create Taffy node: {}", e)))?;

        // Insert into parent if specified
        if let Some(parent_key) = parent {
            if let Some(&parent_id) = self.node_map.get(&parent_key) {
                self.taffy
                    .insert_child_at_index(parent_id, index, node_id)
                    .map_err(|e| ReactiveError::layout(format!("Failed to insert child: {}", e)))?;
                self.dirty_nodes.insert(parent_id);
            }
        } else {
            // This is the root node
            self.root = Some(node_id);
        }

        // Store mappings and metadata
        self.node_map.insert(layout_key.clone(), node_id);
        self.reverse_map.insert(node_id, layout_key);
        self.meta.insert(
            node_id,
            NodeMeta {
                element,
                layout: taffy::Layout::new(),
                dirty: true,
            },
        );
        self.dirty_nodes.insert(node_id);

        // Recursively insert children
        self.insert_children(node_id)?;

        Ok(())
    }

    /// Insert children of a node
    fn insert_children(&mut self, parent_id: NodeId) -> Result<()> {
        let parent_key = self.reverse_map.get(&parent_id).cloned();
        if let Some(key) = parent_key {
            if let Some(meta) = self.meta.get(&parent_id) {
                let children = meta.element.children.clone();
                for (index, child) in children.into_iter().enumerate() {
                    self.insert_node(Some(key.clone()), index, child)?;
                }
            }
        }
        Ok(())
    }

    /// Remove a node from the layout tree
    pub fn remove_node(&mut self, key: LayoutKey) -> Result<()> {
        if let Some(&node_id) = self.node_map.get(&key) {
            // Remove from Taffy tree
            self.taffy
                .remove(node_id)
                .map_err(|e| ReactiveError::layout(format!("Failed to remove node: {}", e)))?;

            // Clean up mappings
            self.node_map.remove(&key);
            self.reverse_map.remove(&node_id);
            self.meta.remove(&node_id);
            self.dirty_nodes.remove(&node_id);

            // If this was root, clear it
            if Some(node_id) == self.root {
                self.root = None;
            }
        }
        Ok(())
    }

    /// Update an existing node
    pub fn update_node(&mut self, key: LayoutKey, element: Element) -> Result<()> {
        if let Some(&node_id) = self.node_map.get(&key) {
            // Update style
            let style = self.element_to_style(&element);
            self.taffy
                .set_style(node_id, style)
                .map_err(|e| ReactiveError::layout(format!("Failed to update style: {}", e)))?;

            // Update metadata
            if let Some(meta) = self.meta.get_mut(&node_id) {
                meta.element = element;
                meta.dirty = true;
            }

            // Mark as dirty
            self.dirty_nodes.insert(node_id);

            // Mark parent as dirty too
            if let Some(parent) = self.taffy.parent(node_id) {
                self.dirty_nodes.insert(parent);
            }
        }
        Ok(())
    }

    /// Move a node to a new position
    pub fn move_node(
        &mut self,
        key: LayoutKey,
        new_parent: Option<LayoutKey>,
        index: usize,
    ) -> Result<()> {
        if let Some(&node_id) = self.node_map.get(&key) {
            // Remove from current parent
            if let Some(old_parent) = self.taffy.parent(node_id) {
                self.taffy.remove_child(old_parent, node_id).map_err(|e| {
                    ReactiveError::layout(format!("Failed to remove from parent: {}", e))
                })?;
                self.dirty_nodes.insert(old_parent);
            }

            // Add to new parent
            if let Some(parent_key) = new_parent {
                if let Some(&parent_id) = self.node_map.get(&parent_key) {
                    self.taffy
                        .insert_child_at_index(parent_id, index, node_id)
                        .map_err(|e| {
                            ReactiveError::layout(format!("Failed to move node: {}", e))
                        })?;
                    self.dirty_nodes.insert(parent_id);
                }
            }

            self.dirty_nodes.insert(node_id);
        }
        Ok(())
    }

    /// Compute layout for dirty nodes
    pub fn compute_dirty_layouts(&mut self) -> Result<()> {
        if self.dirty_nodes.is_empty() {
            return Ok(());
        }

        // If root is dirty, recompute entire tree
        if let Some(root) = self.root {
            if self.dirty_nodes.contains(&root) {
                self.taffy
                    .compute_layout(root, self.available_space)
                    .map_err(|e| {
                        ReactiveError::layout(format!("Failed to compute layout: {}", e))
                    })?;

                // Update all metadata with computed layouts
                self.update_layout_metadata(root)?;
                self.dirty_nodes.clear();
                return Ok(());
            }
        }

        // Otherwise compute layout for each dirty subtree
        let dirty_roots = self.find_dirty_roots();
        for node in dirty_roots {
            // Get the available space from parent's layout
            let available = if let Some(parent) = self.taffy.parent(node) {
                if let Some(parent_meta) = self.meta.get(&parent) {
                    Size {
                        width: AvailableSpace::Definite(parent_meta.layout.size.width),
                        height: AvailableSpace::Definite(parent_meta.layout.size.height),
                    }
                } else {
                    self.available_space
                }
            } else {
                self.available_space
            };

            self.taffy.compute_layout(node, available).map_err(|e| {
                ReactiveError::layout(format!("Failed to compute subtree layout: {}", e))
            })?;

            self.update_layout_metadata(node)?;
        }

        self.dirty_nodes.clear();
        Ok(())
    }

    /// Find root nodes of dirty subtrees
    fn find_dirty_roots(&self) -> Vec<NodeId> {
        let mut roots = Vec::new();
        for &node in &self.dirty_nodes {
            let mut is_root = true;
            let mut current = node;

            // Check if any ancestor is also dirty
            while let Some(parent) = self.taffy.parent(current) {
                if self.dirty_nodes.contains(&parent) {
                    is_root = false;
                    break;
                }
                current = parent;
            }

            if is_root {
                roots.push(node);
            }
        }
        roots
    }

    /// Update layout metadata after computation
    fn update_layout_metadata(&mut self, node: NodeId) -> Result<()> {
        let layout = *self
            .taffy
            .layout(node)
            .map_err(|e| ReactiveError::layout(format!("Failed to get layout: {}", e)))?;

        if let Some(meta) = self.meta.get_mut(&node) {
            meta.layout = layout;
            meta.dirty = false;
        }

        // Update children recursively
        let children: Vec<NodeId> = self
            .taffy
            .children(node)
            .map_err(|e| ReactiveError::layout(format!("Failed to get children: {}", e)))?
            .to_vec();

        for child in children {
            self.update_layout_metadata(child)?;
        }

        Ok(())
    }

    /// Generate paint operations from current layout
    pub fn generate_paint_ops(&mut self) -> Result<Vec<PaintOp>> {
        self.paint_ops.clear();

        if let Some(root) = self.root {
            self.paint_node(root, 0.0, 0.0)?;
        }

        Ok(self.paint_ops.clone())
    }

    /// Paint a node and its children
    fn paint_node(&mut self, node: NodeId, parent_x: f32, parent_y: f32) -> Result<()> {
        if let Some(meta) = self.meta.get(&node) {
            let layout = meta.layout;

            // Check if node is absolutely positioned
            let style = self.taffy.style(node).unwrap();
            let is_absolute = matches!(style.position, taffy::style::Position::Absolute);

            // Absolute elements use location directly, relative add parent offset
            let x = if is_absolute {
                layout.location.x
            } else {
                parent_x + layout.location.x
            };
            let y = if is_absolute {
                layout.location.y
            } else {
                parent_y + layout.location.y
            };

            // Generate paint ops based on element type
            match &meta.element.element_type {
                ElementType::Text(content) => {
                    // Extract visual style from CSS classes
                    let text_style = if let Some(class) = &meta.element.class {
                        crate::layout::css::visual_style::extract_visual_style(class)
                    } else {
                        TextStyle::default()
                    };

                    // Safe conversion with saturation to prevent truncation
                    self.paint_ops.push(PaintOp::Text {
                        x: x.min(u16::MAX as f32) as u16,
                        y: y.min(u16::MAX as f32) as u16,
                        content: content.clone(),
                        style: text_style,
                    });
                }
                ElementType::Layout(_)
                    // Layout nodes might have borders or backgrounds
                    if layout.size.width > 0.0 && layout.size.height > 0.0 => {
                        // Extract visual style from CSS classes for background and borders
                        let box_style = if let Some(class) = &meta.element.class {
                            crate::layout::css::visual_style::extract_box_style(class)
                        } else {
                            BoxStyle::default()
                        };

                        // Use Box paint op for backgrounds and borders
                        self.paint_ops.push(PaintOp::Box {
                            x: x.min(u16::MAX as f32) as u16,
                            y: y.min(u16::MAX as f32) as u16,
                            width: layout.size.width.min(u16::MAX as f32) as u16,
                            height: layout.size.height.min(u16::MAX as f32) as u16,
                            style: box_style,
                        });
                    }
                _ => {}
            }

            // Paint children
            let children: Vec<NodeId> = self
                .taffy
                .children(node)
                .map_err(|e| {
                    ReactiveError::layout(format!("Failed to get children for paint: {}", e))
                })?
                .to_vec();

            for child in children {
                self.paint_node(child, x, y)?;
            }
        }

        Ok(())
    }

    /// Convert Element to Taffy Style
    fn element_to_style(&self, element: &Element) -> Style {
        let mut style = Style::default();

        // Parse class using proper CSS parser
        if let Some(class) = &element.class {
            use crate::layout::css::colors::apply_color_utilities;
            use crate::layout::css::layout::apply_position;
            use crate::layout::css::spacing::{apply_gap, apply_margin, apply_padding};
            use crate::layout::style::StyleBuilder;

            let mut sb = StyleBuilder::new();

            // Parse each token in the class string
            for token in class.split_whitespace() {
                // Try position utilities first (includes absolute, left-X, top-Y, etc.)
                if let Some(new_sb) = apply_position(token, sb.clone()) {
                    sb = new_sb;
                    continue;
                }

                // Try padding utilities
                if let Some(new_sb) = apply_padding(token, sb.clone()) {
                    sb = new_sb;
                    continue;
                }

                // Try margin utilities
                if let Some(new_sb) = apply_margin(token, sb.clone()) {
                    sb = new_sb;
                    continue;
                }

                // Try gap utilities
                if let Some(new_sb) = apply_gap(token, sb.clone()) {
                    sb = new_sb;
                    continue;
                }

                // Try color utilities (text, background, etc.)
                if let Some(new_sb) = apply_color_utilities(token, sb.clone()) {
                    sb = new_sb;
                    continue;
                }

                // Basic layout utilities
                match token {
                    "flex" => {
                        sb = sb.display_flex();
                    }
                    "flex-row" => {
                        sb = sb
                            .display_flex()
                            .direction(crate::layout::style::Direction::Row);
                    }
                    "flex-col" => {
                        sb = sb
                            .display_flex()
                            .direction(crate::layout::style::Direction::Column);
                    }
                    "grid" => {
                        sb = sb.display_grid();
                    }
                    "w-full" => {
                        sb = sb.width_percent(100.0);
                    }
                    "h-full" => {
                        sb = sb.height_percent(100.0);
                    }
                    "relative" => {
                        sb = sb.position_relative();
                    }
                    _ => {}
                }
            }

            // Apply the parsed style to Taffy
            style = sb.build();
        }

        // Set defaults based on element type
        match &element.element_type {
            ElementType::Layout(layout_type) => {
                use crate::component::LayoutType;
                match layout_type {
                    LayoutType::Flex => {
                        style.display = Display::Flex;
                        style.flex_direction = FlexDirection::Column;
                    }
                    LayoutType::Grid => {
                        style.display = Display::Grid;
                    }
                    LayoutType::Stack => {
                        style.display = Display::Flex;
                        style.flex_direction = FlexDirection::Column;
                    }
                    LayoutType::Absolute => {
                        style.position = Position::Absolute;
                    }
                }
            }
            ElementType::Text(_) => {
                // Text nodes are leaf nodes
                style.display = Display::Flex;
            }
            _ => {}
        }

        style
    }

    /// Get current paint operations
    pub fn get_paint_ops(&self) -> &[PaintOp] {
        &self.paint_ops
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.taffy.clear();
        self.node_map.clear();
        self.reverse_map.clear();
        self.meta.clear();
        self.dirty_nodes.clear();
        self.paint_ops.clear();
        self.root = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_manager_creation() {
        let manager = LayoutManager::new(80, 24);
        assert!(manager.root.is_none());
        assert!(manager.node_map.is_empty());
    }

    #[test]
    fn test_insert_root_node() {
        let mut manager = LayoutManager::new(80, 24);
        let mut element = Element::text("Hello");
        element.key = Some("root".to_string());

        let result = manager.insert_node(None, 0, element);
        assert!(result.is_ok());
        assert!(manager.root.is_some());
        assert_eq!(manager.node_map.len(), 1);
    }

    #[test]
    fn test_update_node() {
        let mut manager = LayoutManager::new(80, 24);
        let mut element = Element::text("Hello");
        element.key = Some("text1".to_string());

        manager.insert_node(None, 0, element.clone()).unwrap();

        element = Element::text("World");
        element.key = Some("text1".to_string());

        let result = manager.update_node(LayoutKey::new("text1"), element);
        assert!(result.is_ok());
        assert!(!manager.dirty_nodes.is_empty());
    }

    #[test]
    fn test_remove_node() {
        let mut manager = LayoutManager::new(80, 24);
        let mut element = Element::text("Hello");
        element.key = Some("text1".to_string());

        manager.insert_node(None, 0, element).unwrap();
        assert_eq!(manager.node_map.len(), 1);

        let result = manager.remove_node(LayoutKey::new("text1"));
        assert!(result.is_ok());
        assert_eq!(manager.node_map.len(), 0);
    }

    #[test]
    fn test_compute_layout() {
        let mut manager = LayoutManager::new(80, 24);
        let mut root = Element::layout(crate::component::LayoutType::Flex);
        root.key = Some("root".to_string());

        let mut child = Element::text("Hello");
        child.key = Some("child".to_string());
        root.children.push(child);

        manager.insert_node(None, 0, root).unwrap();

        let result = manager.compute_dirty_layouts();
        assert!(result.is_ok());
        assert!(manager.dirty_nodes.is_empty());
    }

    #[test]
    fn test_generate_paint_ops() {
        let mut manager = LayoutManager::new(80, 24);
        let mut element = Element::text("Hello");
        element.key = Some("text1".to_string());

        manager.insert_node(None, 0, element).unwrap();
        manager.compute_dirty_layouts().unwrap();

        let result = manager.generate_paint_ops();
        assert!(result.is_ok());

        let ops = result.unwrap();
        assert!(!ops.is_empty());
    }
}
