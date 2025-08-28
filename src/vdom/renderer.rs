use super::diff::diff_vnodes;
use super::node::{VComponent, VElement, VFragment, VNode};
use super::patch::{Patch, PatchApplier, PatchList};
use crate::component::element::Element;
use std::collections::HashMap;

/// Type alias for component function
type ComponentFn = Box<dyn Fn(&dyn std::any::Any) -> VNode + Send + Sync>;

/// Context for rendering virtual DOM to real elements
pub struct RenderContext {
    /// Component instances by name
    components: HashMap<String, ComponentFn>,
}

impl RenderContext {
    /// Create a new render context
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component
    pub fn register_component<F>(&mut self, name: impl Into<String>, render: F)
    where
        F: Fn(&dyn std::any::Any) -> VNode + Send + Sync + 'static,
    {
        self.components.insert(name.into(), Box::new(render));
    }

    /// Render a component by name
    pub fn render_component(&self, name: &str, props: &dyn std::any::Any) -> Option<VNode> {
        self.components.get(name).map(|render| render(props))
    }
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Virtual DOM renderer
pub struct VDomRenderer {
    /// Current virtual DOM tree
    current: Option<VNode>,
    /// Render context
    context: RenderContext,
    /// Node index for patch application
    node_index: usize,
}

impl VDomRenderer {
    /// Create a new renderer
    pub fn new() -> Self {
        Self {
            current: None,
            context: RenderContext::new(),
            node_index: 0,
        }
    }

    /// Create with a custom context
    pub fn with_context(context: RenderContext) -> Self {
        Self {
            current: None,
            context,
            node_index: 0,
        }
    }

    /// Render a new virtual DOM tree and get patches
    pub fn render(&mut self, new_tree: VNode) -> PatchList {
        let patches = match &self.current {
            Some(old_tree) => diff_vnodes(old_tree, &new_tree),
            None => {
                // First render, create everything
                let mut patches = PatchList::new();
                patches.push(Patch::Insert {
                    index: 0,
                    parent: 0,
                    node: new_tree.clone(),
                });
                patches
            }
        };

        self.current = Some(new_tree);
        patches
    }

    /// Update the virtual DOM and get patches
    pub fn update<F>(&mut self, updater: F) -> PatchList
    where
        F: FnOnce(&mut VNode),
    {
        match &mut self.current {
            Some(current) => {
                let old = current.clone();
                updater(current);
                diff_vnodes(&old, current)
            }
            None => PatchList::new(),
        }
    }

    /// Get the current virtual DOM tree
    pub fn current(&self) -> Option<&VNode> {
        self.current.as_ref()
    }

    /// Clear the virtual DOM
    pub fn clear(&mut self) {
        self.current = None;
        self.node_index = 0;
    }

    /// Convert a virtual node to a real element
    pub fn vnode_to_element(&mut self, vnode: &VNode) -> Element {
        match vnode {
            VNode::Element(el) => self.velement_to_element(el),
            VNode::Text(text) => Element::text(&text.content),
            VNode::Component(comp) => self.vcomponent_to_element(comp),
            VNode::Fragment(frag) => self.vfragment_to_element(frag),
            VNode::Empty => Element::empty(),
        }
    }

    /// Convert a virtual element to a real element
    fn velement_to_element(&mut self, velement: &VElement) -> Element {
        let mut element = Element::layout(match velement.tag.as_str() {
            "flex" => crate::component::element::LayoutType::Flex,
            "grid" => crate::component::element::LayoutType::Grid,
            "stack" => crate::component::element::LayoutType::Stack,
            "absolute" => crate::component::element::LayoutType::Absolute,
            _ => crate::component::element::LayoutType::Flex,
        });

        // Set class
        if let Some(class) = &velement.class {
            element = element.class(class.clone());
        }

        // Add children
        for child in &velement.children {
            element = element.child(self.vnode_to_element(child));
        }

        // Set key
        if let super::node::VNodeKey::String(key) = &velement.key {
            element = element.key(key.clone());
        }

        element
    }

    /// Convert a virtual component to a real element
    fn vcomponent_to_element(&mut self, vcomponent: &VComponent) -> Element {
        // Try to render the component
        if let Some(rendered) = self
            .context
            .render_component(&vcomponent.name, &*vcomponent.props)
        {
            self.vnode_to_element(&rendered)
        } else {
            // Fallback: create a component element with children
            let mut element = Element::component(vcomponent.name.clone(), ());

            for child in &vcomponent.children {
                element = element.child(self.vnode_to_element(child));
            }

            element
        }
    }

    /// Convert a virtual fragment to a real element
    fn vfragment_to_element(&mut self, vfragment: &VFragment) -> Element {
        let mut element = Element::fragment();

        for child in &vfragment.children {
            element = element.child(self.vnode_to_element(child));
        }

        element
    }
}

impl Default for VDomRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Bridge between virtual DOM patches and real element updates
pub struct ElementPatchApplier<'a> {
    elements: &'a mut Vec<Element>,
}

impl<'a> ElementPatchApplier<'a> {
    /// Create a new element patch applier
    pub fn new(elements: &'a mut Vec<Element>) -> Self {
        Self { elements }
    }
}

impl<'a> PatchApplier for ElementPatchApplier<'a> {
    type Node = Element;

    fn apply_patch(&mut self, patch: &Patch, nodes: &mut Vec<Self::Node>) {
        match patch {
            Patch::Replace { index, new, .. } => {
                if let Some(node) = nodes.get_mut(*index) {
                    // Convert VNode to Element
                    let mut renderer = VDomRenderer::new();
                    *node = renderer.vnode_to_element(new);
                }
            }

            Patch::Insert { index, node, .. } => {
                let mut renderer = VDomRenderer::new();
                let element = renderer.vnode_to_element(node);

                if *index >= nodes.len() {
                    nodes.push(element);
                } else {
                    nodes.insert(*index, element);
                }
            }

            Patch::Remove { index, .. } => {
                if *index < nodes.len() {
                    nodes.remove(*index);
                }
            }

            Patch::Move { from, to, .. } => {
                if *from < nodes.len() && *to < nodes.len() {
                    let node = nodes.remove(*from);
                    nodes.insert(*to, node);
                }
            }

            Patch::SetText { index, text } => {
                if *index < nodes.len() {
                    nodes[*index] = Element::text(text);
                }
            }

            Patch::SetClass { index, class } => {
                if let Some(node) = nodes.get_mut(*index) {
                    if let Some(class) = class {
                        *node = node.clone().class(class.clone());
                    }
                }
            }

            _ => {
                // Other patches not applicable to Element
            }
        }
    }

    fn create_node(&mut self, vnode: &VNode) -> Self::Node {
        let mut renderer = VDomRenderer::new();
        renderer.vnode_to_element(vnode)
    }

    fn get_node(&self, index: usize) -> Option<&Self::Node> {
        self.elements.get(index)
    }

    fn get_node_mut(&mut self, index: usize) -> Option<&mut Self::Node> {
        self.elements.get_mut(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::ElementType;

    #[test]
    fn test_renderer_first_render() {
        let mut renderer = VDomRenderer::new();

        let tree = VNode::element("div")
            .class("container")
            .child(VNode::text("Hello"))
            .build();

        let patches = renderer.render(tree.clone());

        assert_eq!(patches.len(), 1);
        assert!(matches!(patches.patches()[0], Patch::Insert { .. }));
        assert!(renderer.current().is_some());
    }

    #[test]
    fn test_renderer_update() {
        let mut renderer = VDomRenderer::new();

        let tree1 = VNode::element("div").child(VNode::text("Hello")).build();

        renderer.render(tree1);

        let tree2 = VNode::element("div").child(VNode::text("World")).build();

        let patches = renderer.render(tree2);

        // Should have a SetText patch
        assert!(
            patches
                .patches()
                .iter()
                .any(|p| matches!(p, Patch::SetText { .. }))
        );
    }

    #[test]
    fn test_vnode_to_element_conversion() {
        let mut renderer = VDomRenderer::new();

        let vnode = VNode::element("flex")
            .class("container")
            .child(VNode::text("Hello"))
            .build();

        let element = renderer.vnode_to_element(&vnode);

        assert_eq!(
            element.element_type,
            ElementType::Layout(crate::component::element::LayoutType::Flex)
        );
        assert_eq!(element.class, Some("container".to_string()));
        assert_eq!(element.children.len(), 1);
    }

    #[test]
    fn test_component_rendering() {
        let mut context = RenderContext::new();

        // Register a simple component
        context.register_component("Button", |_props| {
            VNode::element("div")
                .class("button")
                .child(VNode::text("Click me"))
                .build()
        });

        let mut renderer = VDomRenderer::with_context(context);

        let component = VNode::component("Button").build();
        let element = renderer.vnode_to_element(&component);

        // Should render the component's output
        assert_eq!(element.class, Some("button".to_string()));
    }
}
