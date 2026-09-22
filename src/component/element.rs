use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

pub(crate) type ElementFactory =
    Arc<dyn Fn(&dyn Any) -> crate::error::Result<super::AnyComponentInstance> + Send + Sync>;
pub(crate) type LayoutCallback = Arc<dyn Fn(super::LayoutInfo) -> bool + Send + Sync>;

#[derive(Clone, Default)]
pub(crate) struct AccessibilityOptions {
    pub id: Option<String>,
    pub keyboard_only: bool,
    pub screen_reader_only: bool,
    pub focus: bool,
    pub clickable: bool,
    pub focus_event: Option<crate::event::CustomEvent>,
    pub click_event: Option<crate::event::CustomEvent>,
    pub label: Option<String>,
}

impl PartialEq for AccessibilityOptions {
    fn eq(&self, other: &Self) -> bool {
        self.focus == other.focus
            && self.clickable == other.clickable
            && self.id == other.id
            && self.keyboard_only == other.keyboard_only
            && self.screen_reader_only == other.screen_reader_only
            && self.label == other.label
            && self
                .click_event
                .as_ref()
                .map(|event| (&event.name, &event.data))
                == other
                    .click_event
                    .as_ref()
                    .map(|event| (&event.name, &event.data))
            && self
                .focus_event
                .as_ref()
                .map(|event| (&event.name, &event.data))
                == other
                    .focus_event
                    .as_ref()
                    .map(|event| (&event.name, &event.data))
    }
}

/// Native cursor attached to a column of one painted text element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TextCursor {
    pub column: u16,
    pub style: ::suprtui::render::CursorStyle,
}

/// Owned behavior attached to an element independently of its component props.
#[derive(Clone, Default)]
pub struct ElementMetadata {
    pub(crate) component_instances: Vec<u64>,
    /// Retain and paint this subtree without interactive or accessible targets.
    pub(crate) inert: bool,
    pub(crate) image: Option<Arc<crate::widgets::display::image::paint::ImagePaint>>,
    pub(crate) image_fallback: Option<u32>,
    /// A prepared cell grid this element paints in one step.
    pub(crate) cells: Option<Arc<crate::layout::paint_tree::cells::CellGrid>>,
    pub(crate) text_cursor: Option<TextCursor>,
    /// Autofocus descendants and restore on removal without trapping Tab.
    pub(crate) focus_scope: bool,
    pub(crate) factory: Option<ElementFactory>,
    pub(crate) events: Vec<crate::event::router::EventHandlerFn>,
    /// Observe descendant input before its target handles activation.
    pub(crate) capture_events: Vec<crate::event::router::EventHandlerFn>,
    pub(crate) layout: Vec<LayoutCallback>,
    pub(crate) paint_style: Option<Arc<crate::layout::style::StyleSnapshot>>,
    /// Explicit layout and paint styles, independent of component props.
    pub styles: Option<Arc<crate::layout::style::StyleSnapshot>>,
    /// Current custom numeric animation properties, supplied from component state.
    pub animation_values: std::collections::HashMap<String, f32>,
    /// VDOM declarations applied after utility classes, including state variants.
    pub(crate) inline_styles: Option<String>,
    /// Cell background gradient.
    pub gradient: Option<crate::layout::css::gradients::Gradient>,
    /// Cell border gradient.
    pub gradient_border: Option<crate::layout::css::gradients::GradientBorder>,
    /// Disabled nodes do not receive focus or activation.
    pub disabled: bool,
    /// Accessible role, label and state, independent of painted text.
    /// App assigns node identity and children from the retained element tree.
    pub accessibility: Option<crate::accessibility::Node>,
    pub(crate) accessibility_options: Option<Box<AccessibilityOptions>>,
    /// Activation callbacks, invoked in registration order.
    pub on_click: Vec<Arc<dyn Fn() + Send + Sync>>,
}

impl PartialEq for ElementMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.animation_values == other.animation_values
            && self.disabled == other.disabled
            && self.inert == other.inert
            && self.image == other.image
            && match (&self.cells, &other.cells) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b) || a == b,
                (None, None) => true,
                _ => false,
            }
            && self.image_fallback == other.image_fallback
            && self.text_cursor == other.text_cursor
            && self.focus_scope == other.focus_scope
            && self.component_instances == other.component_instances
            && self.accessibility == other.accessibility
            && self.accessibility_options == other.accessibility_options
            && self.capture_events.len() == other.capture_events.len()
            && self
                .capture_events
                .iter()
                .zip(&other.capture_events)
                .all(|(a, b)| Arc::ptr_eq(a, b))
            && self.events.len() == other.events.len()
            && self
                .events
                .iter()
                .zip(&other.events)
                .all(|(a, b)| Arc::ptr_eq(a, b))
            && self.layout.len() == other.layout.len()
            && self
                .layout
                .iter()
                .zip(&other.layout)
                .all(|(a, b)| Arc::ptr_eq(a, b))
            && match (&self.factory, &other.factory) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && self.paint_style == other.paint_style
            && self.styles == other.styles
            && self.inline_styles == other.inline_styles
            && self.gradient == other.gradient
            && self.gradient_border == other.gradient_border
            && self.on_click.len() == other.on_click.len()
            && self
                .on_click
                .iter()
                .zip(&other.on_click)
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }
}

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
    /// Owned callbacks and other runtime metadata.
    pub metadata: ElementMetadata,
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
    /// Optional focus properties for declarative focus management
    pub focus: Option<super::focus::FocusProps>,
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
            && self.metadata == other.metadata
            && self.key == other.key
            && self.class == other.class
            && self.children == other.children
            && self.focus == other.focus
            && props_equal
    }
}

impl Element {
    /// An App-local ID referenced by aria-labelledby and aria-describedby styles.
    /// IDs must be nonempty, contain no whitespace, and be unique in this App.
    pub fn with_accessibility_id(mut self, id: impl Into<String>) -> Self {
        self.metadata
            .accessibility_options
            .get_or_insert_default()
            .id = Some(id.into());
        self
    }
    /// Attach semantic role, label and state for assistive technology.
    /// App owns the accessible node's identity and children.
    pub fn with_accessibility(mut self, node: crate::accessibility::Node) -> Self {
        self.metadata.accessibility = Some(node);
        if let Some(options) = &mut self.metadata.accessibility_options {
            options.label = None;
        }
        self
    }

    /// Set an accessible label without changing the painted text.
    pub fn with_accessibility_label(mut self, label: impl Into<String>) -> Self {
        self.metadata
            .accessibility_options
            .get_or_insert_default()
            .label = Some(label.into());
        self
    }

    /// Construct a typed component without a global registry entry.
    /// Each App owns one instance per stable key. This also supports generic widgets.
    pub fn typed<C: super::Component>(props: C::Props) -> Self {
        Self::typed_with::<C>(props, C::new)
    }

    /// Construct each mounted instance with a callback/configuration factory.
    /// The factory runs once per mount; changed props reach the retained instance.
    pub fn typed_with<C: super::Component>(
        props: C::Props,
        create: impl Fn(C::Props) -> C + Send + Sync + 'static,
    ) -> Self {
        let mut element = Self::component_with_props(std::any::type_name::<C>(), props);
        element.metadata.factory = Some(Arc::new(move |props| {
            let props = props.downcast_ref::<C::Props>().ok_or_else(|| {
                crate::error::ReactiveError::invalid_state(
                    "typed component received incompatible props",
                )
            })?;
            Ok(super::AnyComponentInstance::new(
                super::ComponentInstance::<C>::from_component(create(props.clone()), props.clone()),
            ))
        }));
        element
    }

    /// Paint `grid` at this element's content box in one step. The element
    /// keeps its own size from its styles; the grid is clipped to the box,
    /// and its per-cell colors override the element's foreground where set.
    pub fn with_cells(mut self, grid: Arc<crate::layout::paint_tree::cells::CellGrid>) -> Self {
        self.metadata.cells = Some(grid);
        self
    }

    /// The cell grid this element paints, if any.
    pub fn cells(&self) -> Option<&Arc<crate::layout::paint_tree::cells::CellGrid>> {
        self.metadata.cells.as_ref()
    }

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
            focus: None,
            metadata: ElementMetadata::default(),
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
            focus: None,
            metadata: ElementMetadata::default(),
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
            focus: None,
            metadata: ElementMetadata::default(),
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
            focus: None,
            metadata: ElementMetadata::default(),
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
            focus: None,
            metadata: ElementMetadata::default(),
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
            focus: None,
            metadata: ElementMetadata::default(),
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

    /// Set focus properties for declarative focus management
    pub fn with_focus(mut self, focus: super::focus::FocusProps) -> Self {
        self.focus = Some(focus);
        self
    }

    /// Set whether this element accepts focus and activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.metadata.disabled = disabled;
        self
    }

    /// Set auto focus for this element
    pub fn auto_focus(mut self) -> Self {
        let mut focus = self.focus.unwrap_or_default();
        focus.auto_focus = true;
        self.focus = Some(focus);
        self
    }

    /// Set trap focus for this container
    pub fn trap_focus(mut self) -> Self {
        let mut focus = self.focus.unwrap_or_default();
        focus.trap_focus = true;
        self.focus = Some(focus);
        self
    }

    /// Set tab index for this element
    pub fn tab_index(mut self, index: i32) -> Self {
        let mut focus = self.focus.unwrap_or_default();
        focus.tab_index = index;
        self.focus = Some(focus);
        self
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
            panic!("Expected text element, got: {:?}", text.element_type);
        }

        // Test layout creation
        let layout = Element::layout(LayoutType::Flex);
        assert!(!layout.is_component());
        if let ElementType::Layout(layout_type) = &layout.element_type {
            assert_eq!(*layout_type, LayoutType::Flex);
        } else {
            panic!("Expected layout element, got: {:?}", layout.element_type);
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
        let retrieved_props = element
            .props_as::<ButtonProps>()
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
            focus: None,
            metadata: ElementMetadata::default(),
        };
        let elem4 = Element {
            element_type: ElementType::Component("Test".to_string()),
            props: props1.clone(),
            children: Vec::new(),
            key: None,
            class: None,
            focus: None,
            metadata: ElementMetadata::default(),
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

        let props = element
            .props_as::<TestProps>()
            .expect("Should be able to downcast to TestProps");
        assert_eq!(props.name, "test");
        assert_eq!(props.count, 5);
    }
}
