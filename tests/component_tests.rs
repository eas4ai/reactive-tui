use reactive_tui::component::{
    Component, ComponentInstance, Element, ElementType, LifecycleEvent, Props,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[derive(Clone, PartialEq, Debug, Default)]
struct TestProps {
    value: i32,
}

impl Props for TestProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Default)]
struct TestState {
    update_count: usize,
}

struct TestComponent {
    mount_count: Arc<AtomicUsize>,
    render_count: Arc<AtomicUsize>,
}

impl Component for TestComponent {
    type Props = TestProps;
    type State = TestState;

    fn new(_props: Self::Props) -> Self {
        Self {
            mount_count: Arc::new(AtomicUsize::new(0)),
            render_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn update(&mut self, props: &Self::Props, state: &mut Self::State) -> bool {
        state.update_count += 1;
        props.value > 0
    }

    fn render(&self, props: &Self::Props, state: &Self::State) -> Element {
        self.render_count.fetch_add(1, Ordering::SeqCst);
        Element::text(format!(
            "value: {}, updates: {}",
            props.value, state.update_count
        ))
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        if event == LifecycleEvent::Mount {
            self.mount_count.fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[test]
fn test_component_lifecycle() {
    let mut instance = ComponentInstance::<TestComponent>::new(TestProps { value: 5 });

    // Test mounting
    assert!(!instance.lifecycle().is_mounted());
    instance.mount();
    assert!(instance.lifecycle().is_mounted());

    // Test rendering
    let element = instance.render();
    match element.element_type {
        ElementType::Text(text) => {
            assert!(text.contains("value: 5"));
            assert!(text.contains("updates: 0"));
        }
        _ => panic!("Expected text element"),
    }

    // Test prop updates
    let needs_update = instance.update_props(TestProps { value: 10 });
    assert!(needs_update);
    assert_eq!(instance.state().update_count, 1);

    // Test unmounting
    instance.unmount();
    assert!(!instance.lifecycle().is_mounted());
}

#[test]
fn test_element_creation() {
    // Test component element
    let comp = Element::component("MyComponent").with_props(TestProps { value: 42 });
    assert!(matches!(comp.element_type, ElementType::Component(name) if name == "MyComponent"));

    // Test text element
    let text = Element::text("Hello, World!");
    assert!(matches!(text.element_type, ElementType::Text(s) if s == "Hello, World!"));

    // Test layout elements
    let flex = Element::layout(reactive_tui::component::LayoutType::Flex);
    assert!(matches!(
        flex.element_type,
        ElementType::Layout(reactive_tui::component::LayoutType::Flex)
    ));

    // Test fragment
    let fragment = Element::fragment();
    assert!(matches!(fragment.element_type, ElementType::Fragment));

    // Test empty
    let empty = Element::empty();
    assert!(matches!(empty.element_type, ElementType::Empty));
}

#[test]
fn test_element_builder_pattern() {
    let element = Element::component("Container")
        .with_key("container-1")
        .with_child(Element::text("Child 1"))
        .with_children(vec![Element::text("Child 2"), Element::text("Child 3")]);

    assert_eq!(element.key, Some("container-1".to_string()));
    assert_eq!(element.children.len(), 3);
}

#[test]
fn test_element_equality() {
    let elem1 = Element::text("Same").with_key("key1");
    let elem2 = Element::text("Same").with_key("key1");
    let elem3 = Element::text("Different").with_key("key1");
    let elem4 = Element::text("Same").with_key("key2");

    assert_eq!(elem1, elem2);
    assert_ne!(elem1, elem3);
    assert_ne!(elem1, elem4);
}

#[test]
fn test_element_clone() {
    let original = Element::component("Test")
        .with_props(TestProps { value: 123 })
        .with_key("test-key")
        .with_children(vec![Element::text("Child 1"), Element::text("Child 2")]);

    let cloned = original.clone();

    assert_eq!(original.element_type, cloned.element_type);
    assert_eq!(original.key, cloned.key);
    assert_eq!(original.children.len(), cloned.children.len());
    assert!(Arc::ptr_eq(&original.props, &cloned.props));
}

#[test]
fn test_lifecycle_phases() {
    use reactive_tui::component::lifecycle::{Lifecycle, LifecyclePhase};

    let mut lifecycle = Lifecycle::new();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
    assert_eq!(lifecycle.mount_count(), 0);
    assert_eq!(lifecycle.update_count(), 0);

    // Mount
    lifecycle.mount();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Mounting);
    lifecycle.complete_mount();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Mounted);
    assert_eq!(lifecycle.mount_count(), 1);
    assert!(lifecycle.is_mounted());

    // Update
    lifecycle.begin_update();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Updating);
    lifecycle.complete_update();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Mounted);
    assert_eq!(lifecycle.update_count(), 1);

    // Unmount
    lifecycle.unmount();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounting);
    lifecycle.complete_unmount();
    assert_eq!(lifecycle.phase(), LifecyclePhase::Unmounted);
    assert!(!lifecycle.is_mounted());
}

#[test]
fn test_component_needs_update() {
    let mut instance = ComponentInstance::<TestComponent>::new(TestProps { value: 0 });

    assert!(instance.needs_update());
    instance.mark_updated();
    assert!(!instance.needs_update());

    // Updating with different props should mark needs_update
    instance.update_props(TestProps { value: 1 });
    assert!(instance.needs_update());

    // Updating with same props should not mark needs_update
    instance.mark_updated();
    instance.update_props(TestProps { value: 1 });
    assert!(!instance.needs_update());
}
