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

/// Layout type for container elements
///
/// Determines how child elements are arranged within a container.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutType {
    /// Flexbox layout - arranges children in a flexible row or column
    Flex,
    /// CSS Grid layout - arranges children in a two-dimensional grid
    Grid,
    /// Stack layout - layers children on top of each other (z-axis)
    Stack,
    /// Absolute positioning - positions children at specific coordinates
    Absolute,
}

/// Represents an element in the render tree
/// Core element structure for the component system
#[derive(Clone)]
pub struct Element {
    /// Type of element (text, component, etc.)
    pub element_type: ElementType,
    /// Type-erased properties for the element
    pub props: Arc<dyn Any + Send + Sync>,
    /// Child elements
    pub children: Vec<Element>,
    /// Optional unique key for efficient diffing
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
    /// Create a new component element with props
    pub fn component_with_props(
        name: impl Into<String>,
        props: impl Any + Send + Sync + 'static,
    ) -> Self {
        Self {
            element_type: ElementType::Component(name.into()),
            props: Arc::new(props),
            children: Vec::new(),
            key: None,
            class: None,
        }
    }

    /// Create a new component element without props (uses unit type)
    pub fn component(name: impl Into<String>) -> Self {
        Self {
            element_type: ElementType::Component(name.into()),
            props: Arc::new(()),
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

    /// Set props for this element (builder pattern)
    pub fn with_props<T: Any + Send + Sync + 'static>(mut self, props: T) -> Self {
        self.props = Arc::new(props);
        self
    }

    /// Convenience alias for with_props
    pub fn props<T: Any + Send + Sync + 'static>(self, props: T) -> Self {
        self.with_props(props)
    }

    /// Builder method to set children (replaces existing)
    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children = children;
        self
    }

    /// Check if this is a component element
    pub fn is_component(&self) -> bool {
        matches!(self.element_type, ElementType::Component(_))
    }

    /// Get the component name if this is a component element
    pub fn component_name(&self) -> Option<&str> {
        match &self.element_type {
            ElementType::Component(name) => Some(name),
            _ => None,
        }
    }

    /// Get props as Any for downcasting
    pub fn props_any(&self) -> &Arc<dyn Any + Send + Sync> {
        &self.props
    }

    /// Try to downcast props to a specific type
    pub fn props_as<T: Any>(&self) -> Option<&T> {
        self.props.downcast_ref::<T>()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        // Test component creation
        let component = Element::component("Button");
        assert!(component.is_component());
        assert_eq!(component.component_name(), Some("Button"));
        assert_eq!(component.children.len(), 0);
        assert!(component.key.is_none());
        assert!(component.class.is_none());

        // Test text creation
        let text = Element::text("Hello World");
        assert!(!text.is_component());
        assert_eq!(text.component_name(), None);
        if let ElementType::Text(content) = &text.element_type {
            assert_eq!(content, "Hello World");
        } else {
            assert!(false, "Expected text element, got: {:?}", text.element_type);
        }

        // Test layout creation
        let layout = Element::layout(LayoutType::Flex);
        assert!(!layout.is_component());
        if let ElementType::Layout(layout_type) = &layout.element_type {
            assert_eq!(*layout_type, LayoutType::Flex);
        } else {
            assert!(false, "Expected layout element, got: {:?}", layout.element_type);
        }

        // Test fragment creation
        let fragment = Element::fragment();
        assert!(matches!(fragment.element_type, ElementType::Fragment));

        // Test empty creation
        let empty = Element::empty();
        assert!(matches!(empty.element_type, ElementType::Empty));
    }

    #[test]
    fn test_element_with_props() {
        #[derive(Debug, PartialEq)]
        struct ButtonProps {
            label: String,
            disabled: bool,
        }

        let props = ButtonProps {
            label: "Click me".to_string(),
            disabled: false,
        };

        let element = Element::component_with_props("Button", props);
        assert!(element.is_component());

        // Test props downcasting
        let retrieved_props = element.props_as::<ButtonProps>()
            .expect("Should be able to downcast to ButtonProps");
        assert_eq!(retrieved_props.label, "Click me");
        assert!(!retrieved_props.disabled);

        // Test failed downcast
        assert!(element.props_as::<String>().is_none());
    }

    #[test]
    fn test_element_builder_pattern() {
        let element = Element::component("Container")
            .with_key("main-container")
            .with_class("flex flex-col p-4")
            .with_child(Element::text("Title"))
            .with_child(Element::text("Content"));

        assert_eq!(element.key, Some("main-container".to_string()));
        assert_eq!(element.class, Some("flex flex-col p-4".to_string()));
        assert_eq!(element.children.len(), 2);
    }

    #[test]
    fn test_element_convenience_methods() {
        let element = Element::layout(LayoutType::Grid)
            .key("grid-layout")
            .class("grid-cols-2 gap-4")
            .child(Element::text("Cell 1"))
            .child(Element::text("Cell 2"));

        assert_eq!(element.key, Some("grid-layout".to_string()));
        assert_eq!(element.class, Some("grid-cols-2 gap-4".to_string()));
        assert_eq!(element.children.len(), 2);
    }

    #[test]
    fn test_element_with_children() {
        let children = vec![
            Element::text("First"),
            Element::text("Second"),
            Element::text("Third"),
        ];

        let element = Element::fragment().with_children(children);
        assert_eq!(element.children.len(), 3);

        // Test children() method (replaces existing)
        let new_children = vec![Element::text("New child")];
        let element = element.children(new_children);
        assert_eq!(element.children.len(), 1);
    }

    #[test]
    fn test_element_equality() {
        // Test identical elements
        let elem1 = Element::text("Hello").with_key("text1");
        let elem2 = Element::text("Hello").with_key("text1");
        assert_eq!(elem1, elem2);

        // Test different text content
        let elem3 = Element::text("World").with_key("text1");
        assert_ne!(elem1, elem3);

        // Test different keys
        let elem4 = Element::text("Hello").with_key("text2");
        assert_ne!(elem1, elem4);

        // Test different classes
        let elem5 = Element::text("Hello").with_key("text1").with_class("bold");
        assert_ne!(elem1, elem5);
    }

    #[test]
    fn test_component_props_equality() {
        #[derive(Debug)]
        struct Props {
            #[allow(dead_code)]
            value: i32,
        }

        let props1 = Arc::new(Props { value: 42 });

        let elem1 = Element::component_with_props("Test", Props { value: 42 });
        let elem2 = Element::component_with_props("Test", Props { value: 42 });

        // Different Arc instances with same content should not be equal
        assert_ne!(elem1, elem2);

        // Same Arc instance should be equal
        let elem3 = Element {
            element_type: ElementType::Component("Test".to_string()),
            props: props1.clone(),
            children: Vec::new(),
            key: None,
            class: None,
        };
        let elem4 = Element {
            element_type: ElementType::Component("Test".to_string()),
            props: props1.clone(),
            children: Vec::new(),
            key: None,
            class: None,
        };
        assert_eq!(elem3, elem4);
    }

    #[test]
    fn test_layout_types() {
        let flex = Element::layout(LayoutType::Flex);
        let grid = Element::layout(LayoutType::Grid);
        let stack = Element::layout(LayoutType::Stack);
        let absolute = Element::layout(LayoutType::Absolute);

        assert_ne!(flex, grid);
        assert_ne!(grid, stack);
        assert_ne!(stack, absolute);
    }

    #[test]
    fn test_element_debug() {
        let element = Element::component("Button")
            .with_key("btn1")
            .with_child(Element::text("Click me"));

        let debug_str = format!("{:?}", element);
        assert!(debug_str.contains("Button"));
        assert!(debug_str.contains("btn1"));
        assert!(debug_str.contains("children: 1"));
    }

    #[test]
    fn test_props_with_builder() {
        #[derive(Debug, PartialEq)]
        struct TestProps {
            name: String,
            count: usize,
        }

        let element = Element::component("Test").props(TestProps {
            name: "test".to_string(),
            count: 5,
        });

        let props = element.props_as::<TestProps>()
            .expect("Should be able to downcast to TestProps");
        assert_eq!(props.name, "test");
        assert_eq!(props.count, 5);
    }
}
