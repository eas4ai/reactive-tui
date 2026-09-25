use std::any::Any;
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// Type alias for event handler function
type EventHandler = Arc<dyn Fn(&dyn Any) + Send + Sync>;
/// Type alias for event handler map
pub(super) type EventHandlerMap = Arc<HashMap<String, EventHandler>>;

/// Properties delivered to registered components authored with `VNode::element`.
/// Use this as the component's `Props` type; `VNode::component` instead delivers
/// its caller-defined props type unchanged.
#[derive(Clone, Default)]
pub struct VElementProps {
    /// Typed properties supplied by `VElement::prop`.
    pub values: Arc<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// String attributes supplied by `VElement::attr`.
    pub attrs: HashMap<String, String>,
}

impl VElementProps {
    /// Read a property without losing its Rust type.
    pub fn get<T: Any>(&self, name: &str) -> Option<&T> {
        self.values.get(name)?.downcast_ref()
    }
}

impl PartialEq for VElementProps {
    fn eq(&self, other: &Self) -> bool {
        self.attrs == other.attrs
            && self.values.len() == other.values.len()
            && self.values.iter().all(|(key, value)| {
                other
                    .values
                    .get(key)
                    .is_some_and(|other| Arc::ptr_eq(value, other))
            })
    }
}

impl crate::component::Props for VElementProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Key for identifying nodes across renders
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum VNodeKey {
    /// String-based key
    String(String),
    /// Numeric key
    Number(i64),
    /// No key specified
    None,
}

impl From<&str> for VNodeKey {
    fn from(s: &str) -> Self {
        VNodeKey::String(s.to_string())
    }
}

impl From<String> for VNodeKey {
    fn from(s: String) -> Self {
        VNodeKey::String(s)
    }
}

impl From<i64> for VNodeKey {
    fn from(n: i64) -> Self {
        VNodeKey::Number(n)
    }
}

/// Type of virtual node
#[derive(Clone, Debug, PartialEq)]
pub enum VNodeType {
    /// HTML-like element node
    Element,
    /// Text content node
    Text,
    /// Component node
    Component,
    /// Fragment for grouping nodes
    Fragment,
    /// Empty placeholder node
    Empty,
}

/// Virtual DOM node
#[derive(Clone)]
pub enum VNode {
    /// Element node with tag and attributes
    Element(VElement),
    /// Text content node
    Text(VText),
    /// Component node
    Component(VComponent),
    /// Fragment for grouping nodes without wrapper
    Fragment(VFragment),
    /// Empty placeholder node
    Empty,
}

impl VNode {
    /// Get the key of this node
    pub fn key(&self) -> &VNodeKey {
        match self {
            VNode::Element(e) => &e.key,
            VNode::Text(t) => &t.key,
            VNode::Component(c) => &c.key,
            VNode::Fragment(f) => &f.key,
            VNode::Empty => &VNodeKey::None,
        }
    }

    /// Get the type of this node
    pub fn node_type(&self) -> VNodeType {
        match self {
            VNode::Element(_) => VNodeType::Element,
            VNode::Text(_) => VNodeType::Text,
            VNode::Component(_) => VNodeType::Component,
            VNode::Fragment(_) => VNodeType::Fragment,
            VNode::Empty => VNodeType::Empty,
        }
    }

    /// Get children of this node
    pub fn children(&self) -> &[VNode] {
        match self {
            VNode::Element(e) => &e.children,
            VNode::Component(c) => &c.children,
            VNode::Fragment(f) => &f.children,
            VNode::Text(_) | VNode::Empty => &[],
        }
    }

    /// Get mutable children of this node
    /// Returns None for Text and Empty nodes which cannot have children
    pub fn children_mut(&mut self) -> Option<&mut Vec<VNode>> {
        match self {
            VNode::Element(e) => Some(&mut e.children),
            VNode::Component(c) => Some(&mut c.children),
            VNode::Fragment(f) => Some(&mut f.children),
            VNode::Text(_) | VNode::Empty => None, // Text and Empty nodes have no children
        }
    }

    /// Check if this node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, VNode::Empty)
    }

    /// Create an element node
    pub fn element(tag: impl Into<String>) -> VElement {
        VElement::new(tag)
    }

    /// Create a text node
    pub fn text(content: impl Into<String>) -> VNode {
        VNode::Text(VText::new(content))
    }

    /// Create a component node
    pub fn component(name: impl Into<String>) -> VComponent {
        VComponent::new(name)
    }

    /// Create a fragment node
    pub fn fragment() -> VFragment {
        VFragment::new()
    }

    /// Convert this VNode to an Element (for web_api interop)
    pub fn to_element(self) -> crate::component::Element {
        crate::vdom::bridge::vdom_to_element(self)
    }

    /// Create a VNode from a web_api Element (for reverse interop)
    /// Native elements use an opaque VComponent payload to retain their full
    /// behavior, props, styles and focus when converted back with `to_element`.
    pub fn from_element(element: crate::component::Element) -> Self {
        super::bridge::element_to_vdom(element)
    }
}

impl Debug for VNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VNode::Element(e) => write!(f, "{e:?}"),
            VNode::Text(t) => write!(f, "{t:?}"),
            VNode::Component(c) => write!(f, "{c:?}"),
            VNode::Fragment(fr) => write!(f, "{fr:?}"),
            VNode::Empty => write!(f, "Empty"),
        }
    }
}

/// Virtual element node (like HTML elements)
#[derive(Clone)]
pub struct VElement {
    /// Element tag name (e.g., "div", "span")
    pub tag: String,
    /// Unique key for efficient diffing and reconciliation
    pub key: VNodeKey,
    /// HTML-like attributes (id, data-*, etc.)
    pub attrs: HashMap<String, String>,
    /// Typed properties for component communication
    pub props: Arc<HashMap<String, Arc<dyn Any + Send + Sync>>>,
    /// Child virtual nodes
    pub children: Vec<VNode>,
    /// CSS class names
    pub class: Option<String>,
    /// Inline style declarations
    pub style: Option<String>,
    /// Event handler mappings
    pub event_handlers: EventHandlerMap,
}

impl Debug for VElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VElement")
            .field("tag", &self.tag)
            .field("key", &self.key)
            .field("attrs", &self.attrs)
            .field("children", &self.children)
            .field("class", &self.class)
            .field("style", &self.style)
            .finish()
    }
}

impl VElement {
    /// Create a new element
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            key: VNodeKey::None,
            attrs: HashMap::new(),
            props: Arc::new(HashMap::new()),
            children: Vec::new(),
            class: None,
            style: None,
            event_handlers: Arc::new(HashMap::new()),
        }
    }

    /// Set the key
    pub fn key(mut self, key: impl Into<VNodeKey>) -> Self {
        self.key = key.into();
        self
    }

    /// Add an attribute
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.insert(name.into(), value.into());
        self
    }

    /// Add a typed property. Registered components receive `VElementProps` and
    /// can read it with `VElementProps::get` during each render and update.
    pub fn prop<T: Any + Send + Sync + 'static>(
        mut self,
        name: impl Into<String>,
        value: T,
    ) -> Self {
        if let Some(props) = Arc::get_mut(&mut self.props) {
            props.insert(name.into(), Arc::new(value));
        } else {
            // Arc has multiple references, need to clone
            let mut new_props = (*self.props).clone();
            new_props.insert(name.into(), Arc::new(value));
            self.props = Arc::new(new_props);
        }
        self
    }

    /// Set the class
    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Set terminal inline declarations, applied after utility classes.
    /// Lengths use cells (unitless, `px` or `ch`); dimensions also accept `%` and
    /// `auto`. Invalid declarations return a layout error when App renders.
    pub fn style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Add a child
    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = VNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Add an App event handler. The payload downcasts to `crate::event::Event`.
    /// `click` activates on unmodified Enter/Space or left mouse down/click.
    /// `key`, `mouse`, `focus`, `paste` and `custom` observe native bubbling;
    /// native handlers that consume an event stop later bubbling callbacks.
    /// Reusing a name replaces its handler. Disabled elements receive no input.
    pub fn on<F>(mut self, event: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&dyn Any) + Send + Sync + 'static,
    {
        if let Some(handlers) = Arc::get_mut(&mut self.event_handlers) {
            handlers.insert(event.into(), Arc::new(handler));
        } else {
            // Arc has multiple references, need to clone
            let mut new_handlers = (*self.event_handlers).clone();
            new_handlers.insert(event.into(), Arc::new(handler));
            self.event_handlers = Arc::new(new_handlers);
        }
        self
    }

    /// Build into a VNode
    pub fn build(self) -> VNode {
        VNode::Element(self)
    }
}

/// Virtual text node
#[derive(Clone, Debug, PartialEq)]
pub struct VText {
    /// Text content to display
    pub content: String,
    /// Unique key for this node
    pub key: VNodeKey,
}

impl VText {
    /// Create a new text node
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            key: VNodeKey::None,
        }
    }

    /// Set the key
    pub fn key(mut self, key: impl Into<VNodeKey>) -> Self {
        self.key = key.into();
        self
    }
}

/// Virtual component node
#[derive(Clone)]
pub struct VComponent {
    /// Component type name
    pub name: String,
    /// Unique key for this component instance
    pub key: VNodeKey,
    /// Component properties (type-erased)
    pub props: Arc<dyn Any + Send + Sync>,
    /// Child nodes of this component
    pub children: Vec<VNode>,
}

impl VComponent {
    /// Create a new component
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key: VNodeKey::None,
            props: Arc::new(()),
            children: Vec::new(),
        }
    }

    /// Set the key
    pub fn key(mut self, key: impl Into<VNodeKey>) -> Self {
        self.key = key.into();
        self
    }

    /// Set the props
    pub fn props<T: Any + Send + Sync + 'static>(mut self, props: T) -> Self {
        self.props = Arc::new(props);
        self
    }

    /// Add a child
    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = VNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Build into a VNode
    pub fn build(self) -> VNode {
        VNode::Component(self)
    }
}

impl Debug for VComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VComponent")
            .field("name", &self.name)
            .field("key", &self.key)
            .field("children", &self.children.len())
            .finish()
    }
}

/// Virtual fragment node (invisible container)
#[derive(Clone, Debug)]
pub struct VFragment {
    /// Unique key for this fragment
    pub key: VNodeKey,
    /// Child nodes contained in this fragment
    pub children: Vec<VNode>,
}

impl VFragment {
    /// Create a new fragment
    pub fn new() -> Self {
        Self {
            key: VNodeKey::None,
            children: Vec::new(),
        }
    }

    /// Set the key
    pub fn key(mut self, key: impl Into<VNodeKey>) -> Self {
        self.key = key.into();
        self
    }

    /// Add a child
    pub fn child(mut self, child: VNode) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple children
    pub fn children(mut self, children: impl IntoIterator<Item = VNode>) -> Self {
        self.children.extend(children);
        self
    }

    /// Build into a VNode
    pub fn build(self) -> VNode {
        VNode::Fragment(self)
    }
}

impl Default for VFragment {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macros for building virtual DOM trees
#[macro_export]
macro_rules! h {
    // Element with attributes and children
    ($tag:expr, { $($attr:ident : $value:expr),* }, [ $($child:expr),* ]) => {
        {
            let mut elem = $crate::vdom::VNode::element($tag);
            $(
                elem = elem.attr(stringify!($attr), $value);
            )*
            $(
                elem = elem.child($child);
            )*
            elem.build()
        }
    };

    // Element with only children
    ($tag:expr, [ $($child:expr),* ]) => {
        {
            let mut elem = $crate::vdom::VNode::element($tag);
            $(
                elem = elem.child($child);
            )*
            elem.build()
        }
    };

    // Element with only attributes
    ($tag:expr, { $($attr:ident : $value:expr),* }) => {
        {
            let mut elem = $crate::vdom::VNode::element($tag);
            $(
                elem = elem.attr(stringify!($attr), $value);
            )*
            elem.build()
        }
    };

    // Empty element
    ($tag:expr) => {
        $crate::vdom::VNode::element($tag).build()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnode_creation() {
        let node = VNode::element("div")
            .class("container")
            .child(VNode::text("Hello"))
            .build();

        assert_eq!(node.node_type(), VNodeType::Element);
        assert_eq!(node.children().len(), 1);
    }

    #[test]
    fn test_vnode_key() {
        let node = VNode::element("div").key("my-key").build();

        assert_eq!(node.key(), &VNodeKey::String("my-key".to_string()));
    }

    #[test]
    fn test_component_node() {
        let component = VNode::component("Button")
            .props(("label", "Click me"))
            .build();

        assert_eq!(component.node_type(), VNodeType::Component);
    }

    #[test]
    fn test_fragment_node() {
        let fragment = VNode::fragment()
            .child(VNode::text("First"))
            .child(VNode::text("Second"))
            .build();

        assert_eq!(fragment.node_type(), VNodeType::Fragment);
        assert_eq!(fragment.children().len(), 2);
    }

    #[test]
    fn test_h_macro() {
        let node = h!("div", { class: "container", id: "main" }, [
            h!("h1", [VNode::text("Title")]),
            h!("p", [VNode::text("Content")])
        ]);

        assert_eq!(node.node_type(), VNodeType::Element);
        assert_eq!(node.children().len(), 2);
    }
}
