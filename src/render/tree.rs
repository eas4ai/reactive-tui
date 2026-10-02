use crate::component::{AnyComponentInstance, Element};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::OnceLock;

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

    /// Detach owned children for bounded, iterative tree destruction.
    fn take_children(&mut self) -> Vec<Box<dyn RenderNode>> {
        Vec::new()
    }

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

    /// Element data used when reconstructing the render tree. Implementations
    /// may omit children here; `as_element` retains its complete-subtree view.
    #[doc(hidden)]
    fn element_data(&self) -> Option<&Element> {
        self.as_element()
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
    complete_element: OnceLock<Element>,
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
            complete_element: OnceLock::new(),
            children: Vec::new(),
            dirty: true,
            component_instance: None,
        }
    }

    /// Set the children of this node
    pub fn with_children(mut self, children: Vec<Box<dyn RenderNode>>) -> Self {
        self.clear_complete_element();
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

    fn clear_complete_element(&mut self) {
        if let Some(element) = self.complete_element.take() {
            drop_element_iteratively(element);
        }
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

    fn take_children(&mut self) -> Vec<Box<dyn RenderNode>> {
        self.clear_complete_element();
        std::mem::take(&mut self.children)
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
        if self.children.is_empty() {
            return Some(&self.element);
        }
        Some(
            self.complete_element
                .get_or_init(|| reconstruct_element(self).unwrap_or_else(|| self.element.clone())),
        )
    }

    fn element_data(&self) -> Option<&Element> {
        Some(&self.element)
    }

    fn equals(&self, other: &dyn RenderNode) -> bool {
        if let Some(other_element) = other.as_element() {
            self.as_element() == Some(other_element)
        } else {
            false
        }
    }
}

impl Drop for ElementNode {
    fn drop(&mut self) {
        self.clear_complete_element();
        // Automatic cleanup: unregister component instance if present
        if self.component_instance.is_some()
            && crate::component::registry::get_global_registry()
                .unregister_instance(&self.key)
                .is_err()
        {
            // Log error in debug mode, but don't panic during drop
            #[cfg(debug_assertions)]
            log::warn!("Failed to unregister component instance during ElementNode drop");
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

    fn take_children(&mut self) -> Vec<Box<dyn RenderNode>> {
        std::mem::take(&mut self.children)
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

    /// Rebuild the complete Element represented by element and fragment nodes.
    pub fn root_element(&self) -> Option<Element> {
        reconstruct_element(self.root()?)
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
        let mut stack = vec![(node, Vec::new())];
        while let Some((node, path)) = stack.pop() {
            self.node_map.insert(node.key().clone(), path.clone());
            for (index, child) in node.children().iter().enumerate().rev() {
                let mut child_path = path.clone();
                child_path.push(index);
                stack.push((child.as_ref(), child_path));
            }
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

impl Drop for RenderTree {
    fn drop(&mut self) {
        let mut stack = self.root.take().into_iter().collect::<Vec<_>>();
        while let Some(mut node) = stack.pop() {
            stack.extend(node.take_children());
        }
    }
}

impl Default for RenderTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert App output without instantiating or cloning components a second time.
pub(crate) fn resolved_element_to_render_node(element: Element) -> Box<dyn RenderNode> {
    convert_elements(element, true, false)
}

/// Helper to convert Element tree to RenderNode tree with automatic component instantiation
pub fn element_to_render_node(element: Element) -> Box<dyn RenderNode> {
    convert_elements(element, false, true)
}

fn drop_element_iteratively(root: Element) {
    let mut pending = vec![root];
    while let Some(mut element) = pending.pop() {
        pending.append(&mut element.children);
    }
}

fn reconstruct_element(root: &dyn RenderNode) -> Option<Element> {
    enum Task<'a> {
        Enter(&'a dyn RenderNode),
        Exit(&'a dyn RenderNode, usize),
    }
    let mut tasks = vec![Task::Enter(root)];
    let mut built = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Enter(node) => {
                if node.element_data().is_none() && node.node_type() != "Fragment" {
                    for element in built {
                        drop_element_iteratively(element);
                    }
                    return None;
                }
                tasks.push(Task::Exit(node, node.children().len()));
                for child in node.children().iter().rev() {
                    tasks.push(Task::Enter(child.as_ref()));
                }
            }
            Task::Exit(node, child_count) => {
                let children = built.split_off(built.len() - child_count);
                let mut element = node
                    .element_data()
                    .cloned()
                    .unwrap_or_else(Element::fragment);
                element.children = children;
                built.push(element);
            }
        }
    }
    built.pop()
}

fn convert_elements(root: Element, composite_keys: bool, instantiate: bool) -> Box<dyn RenderNode> {
    enum Task {
        Enter(Element, Option<NodeKey>, usize),
        Exit(Element, NodeKey, usize),
    }
    let mut tasks = vec![Task::Enter(root, None, 0)];
    let mut built: Vec<Box<dyn RenderNode>> = Vec::new();
    while let Some(task) = tasks.pop() {
        match task {
            Task::Enter(mut element, parent, index) => {
                let local = element.key.as_ref().map_or_else(
                    || {
                        if composite_keys {
                            NodeKey::index(index)
                        } else {
                            NodeKey::auto()
                        }
                    },
                    NodeKey::named,
                );
                let key = if composite_keys {
                    parent.map_or_else(
                        || local.clone(),
                        |parent| NodeKey::Composite(Box::new(parent), Box::new(local.clone())),
                    )
                } else {
                    local
                };
                let children = std::mem::take(&mut element.children);
                let child_count = children.len();
                tasks.push(Task::Exit(element, key.clone(), child_count));
                for (index, child) in children.into_iter().enumerate().rev() {
                    tasks.push(Task::Enter(child, Some(key.clone()), index));
                }
            }
            Task::Exit(element, key, child_count) => {
                let children = built.split_off(built.len() - child_count);
                if matches!(
                    &element.element_type,
                    crate::component::ElementType::Fragment
                ) {
                    built.push(Box::new(FragmentNode::new(children).with_key(key)));
                    continue;
                }
                let component_name = match &element.element_type {
                    crate::component::ElementType::Component(name) => Some(name.clone()),
                    _ => None,
                };
                let mut node = ElementNode::new(element.clone())
                    .with_key(key)
                    .with_children(children);
                if instantiate {
                    if let Some(component_name) = component_name {
                        if let Ok(Some(instance)) =
                            crate::component::registry::get_global_registry()
                                .create_by_name(&component_name, element.props.as_ref())
                        {
                            if let Err(_error) = crate::component::registry::get_global_registry()
                                .register_instance_with_element(
                                    node.key().clone(),
                                    instance.clone(),
                                    &element,
                                )
                            {
                                #[cfg(debug_assertions)]
                                log::warn!("Failed to register component instance: {_error}");
                            } else {
                                node = node.with_component_instance(instance);
                            }
                        }
                    }
                }
                built.push(Box::new(node));
            }
        }
    }
    built
        .pop()
        .expect("element conversion always produces one root")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{Element, LayoutType};

    #[derive(Debug)]
    struct CustomNode {
        key: NodeKey,
        children: Vec<Box<dyn RenderNode>>,
        dirty: bool,
    }

    impl RenderNode for CustomNode {
        fn key(&self) -> &NodeKey {
            &self.key
        }
        fn node_type(&self) -> &str {
            "Custom"
        }
        fn children(&self) -> &[Box<dyn RenderNode>] {
            &self.children
        }
        fn take_children(&mut self) -> Vec<Box<dyn RenderNode>> {
            std::mem::take(&mut self.children)
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
            self.key == *other.key()
        }
    }

    #[test]
    fn api019_public_render_tree_keeps_fragment_custom_deep_and_wide_nodes() {
        let wide = (0..2048)
            .map(|index| {
                Box::new(
                    ElementNode::new(Element::text(index.to_string()))
                        .with_key(NodeKey::index(index)),
                ) as Box<dyn RenderNode>
            })
            .collect();
        let custom = CustomNode {
            key: NodeKey::named("custom"),
            children: vec![Box::new(
                FragmentNode::new(wide).with_key(NodeKey::named("fragment")),
            )],
            dirty: true,
        };
        let mut tree = RenderTree::new();
        tree.set_root(Box::new(custom));
        assert_eq!(
            tree.find_node(&NodeKey::named("custom"))
                .unwrap()
                .node_type(),
            "Custom"
        );
        assert_eq!(
            tree.find_node(&NodeKey::named("fragment"))
                .unwrap()
                .children()
                .len(),
            2048
        );
        assert_eq!(
            tree.find_node(&NodeKey::index(2047)).unwrap().node_type(),
            "Text"
        );

        let mut deep = Element::text("leaf").with_key("leaf");
        for index in (0..256).rev() {
            deep = Element::layout(LayoutType::Flex)
                .with_key(format!("depth-{index}"))
                .with_child(deep);
        }
        tree.set_root(element_to_render_node(deep));
        let root = tree.root().unwrap();
        assert!(root.element_data().unwrap().children.is_empty());
        let snapshot = root.as_element().unwrap();
        assert!(std::ptr::eq(snapshot, root.as_element().unwrap()));
        let mut descendant = snapshot;
        for _ in 0..256 {
            descendant = descendant.children.first().unwrap();
        }
        assert!(matches!(
            &descendant.element_type,
            crate::component::ElementType::Text(text) if text == "leaf"
        ));
        assert_eq!(
            tree.find_node(&NodeKey::named("leaf")).unwrap().node_type(),
            "Text"
        );
        let rebuilt = tree.root_element().unwrap();
        let mut cursor = &rebuilt;
        for _ in 0..256 {
            cursor = cursor.children.first().unwrap();
        }
        assert!(matches!(
            &cursor.element_type,
            crate::component::ElementType::Text(_)
        ));

        let mut node = ElementNode::new(Element::layout(LayoutType::Flex))
            .with_children(vec![Box::new(ElementNode::new(Element::text("old")))]);
        assert_eq!(node.as_element().unwrap().children.len(), 1);
        node = node.with_children(vec![Box::new(ElementNode::new(Element::text("new")))]);
        assert!(matches!(
            &node.as_element().unwrap().children[0].element_type,
            crate::component::ElementType::Text(text) if text == "new"
        ));
        drop(node.take_children());
        assert!(node.as_element().unwrap().children.is_empty());

        // Conversion, snapshot assembly and owned snapshot destruction must
        // not consume stack proportional to tree depth.
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let mut element = Element::text("bottom");
                for _ in 0..2048 {
                    element = Element::layout(LayoutType::Flex).with_child(element);
                }
                let mut tree = RenderTree::new();
                tree.set_root(element_to_render_node(element));
                let mut cursor = tree.root().unwrap().as_element().unwrap();
                for _ in 0..2048 {
                    cursor = &cursor.children[0];
                }
                assert!(matches!(
                    &cursor.element_type,
                    crate::component::ElementType::Text(text) if text == "bottom"
                ));
                drop(tree);
            })
            .unwrap()
            .join()
            .unwrap();
    }

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
