use crate::component::{AnyComponentInstance, Element};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// Key for stable component identity across renders
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeKey {
    /// Indexed key for list items
    Index(usize),
    /// String key for named components
    Named(String),
    /// Composite key for nested structures
    Composite(Box<NodeKey>, Box<NodeKey>),
    /// Auto-generated key
    Auto(u64),
}

impl NodeKey {
    /// Create an indexed key for list items
    pub fn index(i: usize) -> Self {
        NodeKey::Index(i)
    }

    /// Create a named key for stable component identity
    pub fn named(name: impl Into<String>) -> Self {
        NodeKey::Named(name.into())
    }

    /// Create an auto-generated unique key
    pub fn auto() -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        NodeKey::Auto(COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }
}

/// Trait for nodes in the render tree
pub trait RenderNode: Debug + Send + Sync {
    /// Get the node's unique key
    fn key(&self) -> &NodeKey;

    /// Get the node's type identifier
    fn node_type(&self) -> &str;

    /// Get children of this node
    fn children(&self) -> &[Box<dyn RenderNode>];

    /// Check if this node needs re-rendering
    fn is_dirty(&self) -> bool;

    /// Mark this node as clean after rendering
    fn mark_clean(&mut self);

    /// Mark this node as dirty (needs re-rendering)
    fn mark_dirty(&mut self);

    /// Check if any child is dirty (for optimization)
    fn has_dirty_children(&self) -> bool {
        self.children().iter().any(|child| child.is_dirty())
    }

    /// Get the underlying element if this is an element node
    fn as_element(&self) -> Option<&Element> {
        None
    }

    /// Get any associated data
    fn data(&self) -> Option<&dyn Any> {
        None
    }

    /// Compare with another node for equality
    fn equals(&self, other: &dyn RenderNode) -> bool;
}

/// Concrete implementation of RenderNode for Elements
#[derive(Debug)]
pub struct ElementNode {
    key: NodeKey,
    element: Element,
    children: Vec<Box<dyn RenderNode>>,
    dirty: bool,
    /// Component instance for component elements (automatic memory management)
    component_instance: Option<AnyComponentInstance>,
}

impl ElementNode {
    /// Create a new element node from an Element
    pub fn new(element: Element) -> Self {
        let key = element
            .key
            .as_ref()
            .map(|k| NodeKey::named(k.clone()))
            .unwrap_or_else(NodeKey::auto);

        Self {
            key,
            element,
            children: Vec::new(),
            dirty: true,
            component_instance: None,
        }
    }

    /// Set the children of this node
    pub fn with_children(mut self, children: Vec<Box<dyn RenderNode>>) -> Self {
        self.children = children;
        self
    }

    /// Set a specific key for this node
    pub fn with_key(mut self, key: NodeKey) -> Self {
        self.key = key;
        self
    }

    /// Set the component instance for this node (for component elements)
    pub fn with_component_instance(mut self, instance: AnyComponentInstance) -> Self {
        self.component_instance = Some(instance);
        self
    }

    /// Get the component instance if this is a component element
    pub fn component_instance(&self) -> Option<&AnyComponentInstance> {
        self.component_instance.as_ref()
    }

    /// Take the component instance (for cleanup)
    pub fn take_component_instance(&mut self) -> Option<AnyComponentInstance> {
        self.component_instance.take()
    }

    /// Check if this node represents a component
    pub fn is_component(&self) -> bool {
        matches!(
            self.element.element_type,
            crate::component::ElementType::Component(_)
        )
    }
}

impl RenderNode for ElementNode {
    fn key(&self) -> &NodeKey {
        &self.key
    }

    fn node_type(&self) -> &str {
        match &self.element.element_type {
            crate::component::ElementType::Component(name) => name,
            crate::component::ElementType::Text(_) => "Text",
            crate::component::ElementType::Layout(layout) => match layout {
                crate::component::LayoutType::Flex => "Flex",
                crate::component::LayoutType::Grid => "Grid",
                crate::component::LayoutType::Stack => "Stack",
                crate::component::LayoutType::Absolute => "Absolute",
            },
            crate::component::ElementType::Fragment => "Fragment",
            crate::component::ElementType::Empty => "Empty",
        }
    }

    fn children(&self) -> &[Box<dyn RenderNode>] {
        &self.children
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn as_element(&self) -> Option<&Element> {
        Some(&self.element)
    }

    fn equals(&self, other: &dyn RenderNode) -> bool {
        if let Some(other_element) = other.as_element() {
            self.element == *other_element
        } else {
            false
        }
    }
}

impl Drop for ElementNode {
    fn drop(&mut self) {
        // Automatic cleanup: unregister component instance if present
        if self.component_instance.is_some()
            && crate::component::registry::get_global_registry()
                .unregister_instance(&self.key)
                .is_err()
        {
            // Log error in debug mode, but don't panic during drop
            #[cfg(debug_assertions)]
            eprintln!("Warning: Failed to unregister component instance during ElementNode drop");
        }
    }
}

/// Fragment node for grouping multiple children without a wrapper
#[derive(Debug)]
pub struct FragmentNode {
    key: NodeKey,
    children: Vec<Box<dyn RenderNode>>,
    dirty: bool,
}

impl FragmentNode {
    /// Create a new fragment node with children
    pub fn new(children: Vec<Box<dyn RenderNode>>) -> Self {
        Self {
            key: NodeKey::auto(),
            children,
            dirty: true,
        }
    }

    /// Set a specific key for this fragment
    pub fn with_key(mut self, key: NodeKey) -> Self {
        self.key = key;
        self
    }
}

impl RenderNode for FragmentNode {
    fn key(&self) -> &NodeKey {
        &self.key
    }

    fn node_type(&self) -> &str {
        "Fragment"
    }

    fn children(&self) -> &[Box<dyn RenderNode>] {
        &self.children
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn equals(&self, other: &dyn RenderNode) -> bool {
        other.node_type() == "Fragment" && other.key() == self.key()
    }
}

/// The render tree structure
pub struct RenderTree {
    root: Option<Box<dyn RenderNode>>,
    // Fast lookup: map NodeKey to path of child indices from root
    node_map: HashMap<NodeKey, Vec<usize>>,
    dirty_nodes: Vec<NodeKey>,
}

impl RenderTree {
    /// Create a new empty render tree
    pub fn new() -> Self {
        Self {
            root: None,
            node_map: HashMap::new(),
            dirty_nodes: Vec::new(),
        }
    }

    /// Set the root node of the tree
    pub fn set_root(&mut self, root: Box<dyn RenderNode>) {
        self.build_node_map(root.as_ref());
        self.root = Some(root);
    }

    /// Take the root node, leaving None in its place
    pub fn take_root(&mut self) -> Option<Box<dyn RenderNode>> {
        self.node_map.clear();
        self.dirty_nodes.clear();
        self.root.take()
    }

    /// Replace the root with a new one, returning the old root
    pub fn replace_root(&mut self, root: Box<dyn RenderNode>) -> Option<Box<dyn RenderNode>> {
        self.build_node_map(root.as_ref());
        self.root.replace(root)
    }

    /// Get the root node
    pub fn root(&self) -> Option<&dyn RenderNode> {
        self.root.as_ref().map(|r| r.as_ref())
    }

    /// Get a mutable reference to the root
    pub fn root_mut(&mut self) -> Option<&mut Box<dyn RenderNode>> {
        self.root.as_mut()
    }

    /// Find a node by its key
    pub fn find_node(&self, key: &NodeKey) -> Option<&dyn RenderNode> {
        let path = self.node_map.get(key)?;
        let mut node_ref: &dyn RenderNode = self.root.as_ref()?.as_ref();
        if node_ref.key() != key {
            for &idx in path {
                let children = node_ref.children();
                node_ref = children.get(idx)?.as_ref();
                if node_ref.key() == key {
                    break;
                }
            }
        }
        Some(node_ref)
    }

    /// Mark a node as dirty
    pub fn mark_dirty(&mut self, key: NodeKey) {
        if !self.dirty_nodes.contains(&key) {
            self.dirty_nodes.push(key);
        }
    }

    /// Get all dirty nodes
    pub fn dirty_nodes(&self) -> &[NodeKey] {
        &self.dirty_nodes
    }

    /// Clear the dirty nodes list
    pub fn clear_dirty(&mut self) {
        self.dirty_nodes.clear();
    }

    /// Build the node map for fast lookups
    fn build_node_map(&mut self, node: &dyn RenderNode) {
        self.node_map.clear();
        let mut path = Vec::new();
        self.add_to_map(node, &mut path);
    }

    fn add_to_map(&mut self, node: &dyn RenderNode, path: &mut Vec<usize>) {
        // Record current node
        self.node_map.insert(node.key().clone(), path.clone());
        // Recurse into children with indexed path
        for (i, child) in node.children().iter().enumerate() {
            path.push(i);
            self.add_to_map(child.as_ref(), path);
            path.pop();
        }
    }

    /// Remove a node from the tree by its key
    pub fn remove_node(&mut self, key: &NodeKey) {
        // Remove from node map
        self.node_map.remove(key);

        // Remove from dirty nodes
        self.dirty_nodes.retain(|k| k != key);

        // Note: The actual node removal from the tree structure
        // is handled by the reconciliation process during tree rebuilding
    }
}

impl Default for RenderTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert App output without instantiating or cloning components a second time.
pub(crate) fn resolved_element_to_render_node(element: Element) -> Box<dyn RenderNode> {
    fn convert(mut element: Element, parent: Option<NodeKey>, index: usize) -> Box<dyn RenderNode> {
        let local = element
            .key
            .as_ref()
            .map_or_else(|| NodeKey::index(index), NodeKey::named);
        let key = parent.map_or_else(
            || local.clone(),
            |parent| NodeKey::Composite(Box::new(parent), Box::new(local.clone())),
        );
        let children = std::mem::take(&mut element.children)
            .into_iter()
            .enumerate()
            .map(|(index, child)| convert(child, Some(key.clone()), index))
            .collect();
        if matches!(
            element.element_type,
            crate::component::ElementType::Fragment
        ) {
            Box::new(FragmentNode::new(children).with_key(key))
        } else {
            Box::new(
                ElementNode::new(element)
                    .with_key(key)
                    .with_children(children),
            )
        }
    }
    convert(element, None, 0)
}

/// Helper to convert Element tree to RenderNode tree with automatic component instantiation
pub fn element_to_render_node(element: Element) -> Box<dyn RenderNode> {
    let children: Vec<Box<dyn RenderNode>> = element
        .children
        .clone()
        .into_iter()
        .map(element_to_render_node)
        .collect();

    match element.element_type {
        crate::component::ElementType::Fragment => Box::new(
            FragmentNode::new(children).with_key(
                element
                    .key
                    .map(NodeKey::named)
                    .unwrap_or_else(NodeKey::auto),
            ),
        ),
        crate::component::ElementType::Component(ref component_name) => {
            let mut node = ElementNode::new(element.clone()).with_children(children);

            // Automatic component instantiation - the core fix!
            if let Ok(Some(instance)) = crate::component::registry::get_global_registry()
                .create_by_name(component_name, element.props.as_ref())
            {
                // Register instance for automatic cleanup tracking with CSS animation support
                if let Err(e) = crate::component::registry::get_global_registry()
                    .register_instance_with_element(node.key().clone(), instance.clone(), &element)
                {
                    #[cfg(debug_assertions)]
                    eprintln!("Warning: Failed to register component instance: {}", e);
                } else {
                    // Successfully created and registered component instance
                    node = node.with_component_instance(instance);
                }
            } else {
                // Component not registered - this is not an error, just means
                // the component will be treated as a regular element
                #[cfg(debug_assertions)]
                eprintln!(
                    "Info: Component '{}' not registered, treating as regular element",
                    component_name
                );
            }

            Box::new(node)
        }
        _ => Box::new(ElementNode::new(element).with_children(children)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Element, LayoutType};

    #[test]
    fn test_node_key() {
        let key1 = NodeKey::index(0);
        let key2 = NodeKey::named("button");
        let key3 = NodeKey::auto();
        let key4 = NodeKey::auto();

        assert_eq!(key1, NodeKey::index(0));
        assert_eq!(key2, NodeKey::named("button"));
        assert_ne!(key3, key4); // Auto keys should be unique
    }

    #[test]
    fn test_element_node() {
        let element = Element::text("Hello");
        let node = ElementNode::new(element);

        assert_eq!(node.node_type(), "Text");
        assert!(node.is_dirty());
        assert_eq!(node.children().len(), 0);
    }

    #[test]
    fn test_fragment_node() {
        let children = vec![
            Box::new(ElementNode::new(Element::text("A"))) as Box<dyn RenderNode>,
            Box::new(ElementNode::new(Element::text("B"))) as Box<dyn RenderNode>,
        ];
        let fragment = FragmentNode::new(children);
        assert_eq!(fragment.node_type(), "Fragment");
        assert_eq!(fragment.children().len(), 2);
    }

    // Additional tests
    #[test]
    fn test_find_node_by_key() {
        let mut tree = RenderTree::new();
        let root_el = Element::layout(LayoutType::Flex).with_children(vec![
            Element::text("A").with_key("a"),
            Element::text("B").with_key("b"),
        ]);
        let root = element_to_render_node(root_el);
        tree.set_root(root);

        let a = tree.find_node(&NodeKey::named("a"));
        assert!(a.is_some());
        assert_eq!(a.unwrap().node_type(), "Text");

        let b = tree.find_node(&NodeKey::named("b"));
        assert!(b.is_some());
        assert_eq!(b.unwrap().node_type(), "Text");

        let none = tree.find_node(&NodeKey::named("nope"));
        assert!(none.is_none());
    }

    #[test]
    fn test_render_tree() {
        let mut tree = RenderTree::new();

        let root = Box::new(ElementNode::new(Element::layout(LayoutType::Flex)));
        tree.set_root(root);

        assert!(tree.root().is_some());
        assert_eq!(tree.root().unwrap().node_type(), "Flex");
    }
}
