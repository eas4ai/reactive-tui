use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// Type of element in the render tree
#[derive(Debug, Clone, PartialEq)]
pub enum ElementType {
    /// A component instance
    Component(String),

    /// A text node
    Text(String),

    /// A layout container (flex, grid, etc.)
    Layout(LayoutType),

    /// A fragment (invisible container)
    Fragment,

    /// Empty/null element
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutType {
    Flex,
    Grid,
    Stack,
    Absolute,
}

/// Represents an element in the render tree
#[derive(Clone)]
pub struct Element {
    pub element_type: ElementType,
    pub props: Arc<dyn Any + Send + Sync>,
    pub children: Vec<Element>,
    pub key: Option<String>,
    /// Optional utility-css style class (e.g., "flex flex-row p-2")
    pub class: Option<String>,
}

impl PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        // For text and other simple elements, just compare type, key and children
        // For components, we currently treat props equality as pointer equality via Arc::ptr_eq.
        // This is a performance tradeoff: equal-but-reallocated props will be considered different,
        // triggering re-renders. If this becomes an issue, consider adding an optional content hash
        // or a Props::eq_dyn fast-path for component props.
        let props_equal = match (&self.element_type, &other.element_type) {
            (ElementType::Component(_), ElementType::Component(_)) => {
                Arc::ptr_eq(&self.props, &other.props)
            }
            _ => true, // For text, layout, fragment, empty - props don't matter
        };

        self.element_type == other.element_type
            && self.key == other.key
            && self.class == other.class
            && self.children == other.children
            && props_equal
    }
}

impl Element {
    /// Create a new component element
    pub fn component(name: impl Into<String>, props: impl Any + Send + Sync + 'static) -> Self {
        Self {
            element_type: ElementType::Component(name.into()),
            props: Arc::new(props),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Create a text element
    pub fn text(content: impl Into<String>) -> Self {
        let text = content.into();
        Self {
            element_type: ElementType::Text(text.clone()),
            props: Arc::new(text),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Create a layout element
    pub fn layout(layout_type: LayoutType) -> Self {
        Self {
            element_type: ElementType::Layout(layout_type),
            props: Arc::new(()),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Create a fragment element
    pub fn fragment() -> Self {
        Self {
            element_type: ElementType::Fragment,
            props: Arc::new(()),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Create an empty element
    pub fn empty() -> Self {
        Self {
            element_type: ElementType::Empty,
            props: Arc::new(()),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Set the key for this element
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the utility-css class string for this element
    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.class = Some(class.into());
        self
    }

    /// Add children to this element
    pub fn with_children(mut self, mut children: Vec<Element>) -> Self {
        self.children.append(&mut children);
        self
    }

    /// Add a single child
    pub fn with_child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    /// Convenience alias for with_child
    pub fn child(self, child: Element) -> Self {
        self.with_child(child)
    }

    /// Convenience alias for with_key
    pub fn key(self, key: impl Into<String>) -> Self {
        self.with_key(key)
    }

    /// Convenience alias for with_class
    pub fn class(self, class: impl Into<String>) -> Self {
        self.with_class(class)
    }
}

impl Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element")
            .field("type", &self.element_type)
            .field("key", &self.key)
            .field("children", &self.children.len())
            .finish()
    }
}
