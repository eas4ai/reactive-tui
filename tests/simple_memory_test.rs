use reactive_tui::component::{Component, Element, Props, registry::{register_component, global_active_count, global_cleanup_all}};
use reactive_tui::render::tree::element_to_render_node;

#[derive(Clone, PartialEq, Default)]
struct SimpleProps {
    value: i32,
}

impl Props for SimpleProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct SimpleComponent;

impl Component for SimpleComponent {
    type Props = SimpleProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(format!("value: {}", props.value))
    }
}

#[test]
fn test_basic_component_creation() {
    // Clean up any existing components
    global_cleanup_all().unwrap();

    // Register component
    register_component::<SimpleComponent>("SimpleComponent").unwrap();

    // Create element with component
    let element = Element::component_with_props("SimpleComponent", SimpleProps { value: 42 });

    // Convert to render node - should automatically instantiate component
    let render_node = element_to_render_node(element);

    // Verify component was created and registered
    assert_eq!(global_active_count().unwrap(), 1);

    // Drop render node - should automatically cleanup
    drop(render_node);
    
    // Verify component was cleaned up
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
fn test_multiple_components() {
    // Ensure clean state
    global_cleanup_all().unwrap();

    // Re-register component (in case it was cleared)
    let _ = register_component::<SimpleComponent>("SimpleComponent");

    // Create multiple components
    let elements = vec![
        Element::component_with_props("SimpleComponent", SimpleProps { value: 1 }),
        Element::component_with_props("SimpleComponent", SimpleProps { value: 2 }),
        Element::component_with_props("SimpleComponent", SimpleProps { value: 3 }),
    ];

    let render_nodes: Vec<_> = elements.into_iter()
        .map(element_to_render_node)
        .collect();

    // Verify all components were created
    let count = global_active_count().unwrap();
    println!("Active count after creating 3 components: {}", count);
    assert_eq!(count, 3);

    // Drop all nodes
    drop(render_nodes);

    // Verify all components were cleaned up
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
fn test_unregistered_component() {
    // Ensure clean state
    global_cleanup_all().unwrap();

    // Don't register the component
    
    // Create element with unregistered component
    let element = Element::component_with_props("UnregisteredComponent", SimpleProps { value: 42 });
    let render_node = element_to_render_node(element);

    // Should not create any component instances
    assert_eq!(global_active_count().unwrap(), 0);

    // Should not crash when dropped
    drop(render_node);
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
fn test_global_cleanup() {
    // Ensure clean state
    global_cleanup_all().unwrap();

    // Re-register component (in case it was cleared)
    let _ = register_component::<SimpleComponent>("SimpleComponent");

    // Create multiple components without dropping them
    let _nodes: Vec<_> = (0..5)
        .map(|i| {
            let element = Element::component_with_props("SimpleComponent", SimpleProps { value: i });
            element_to_render_node(element)
        })
        .collect();

    // Verify components were created
    let count = global_active_count().unwrap();
    println!("Active count after creating 5 components: {}", count);
    assert_eq!(count, 5);

    // Global cleanup should clean up all components
    let cleaned_count = global_cleanup_all().unwrap();
    assert_eq!(cleaned_count, 5);
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
fn test_concurrent_component_registration() {
    use std::sync::Arc;
    use std::thread;

    global_cleanup_all().unwrap();

    // Test concurrent registration of the same component
    let handles: Vec<_> = (0..10).map(|i| {
        thread::spawn(move || {
            let component_name = format!("ConcurrentComponent{}", i);
            register_component::<SimpleComponent>(component_name)
        })
    }).collect();

    // All registrations should succeed (different names)
    for handle in handles {
        assert!(handle.join().unwrap().is_ok());
    }

    // Test concurrent registration of same name (should fail for duplicates)
    let handles: Vec<_> = (0..5).map(|_| {
        thread::spawn(|| {
            register_component::<SimpleComponent>("DuplicateComponent")
        })
    }).collect();

    let results: Vec<_> = handles.into_iter()
        .map(|h| h.join().unwrap())
        .collect();

    // Only one should succeed, others should fail with duplicate error
    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures = results.iter().filter(|r| r.is_err()).count();

    assert_eq!(successes, 1, "Exactly one registration should succeed");
    assert_eq!(failures, 4, "Four registrations should fail as duplicates");
}
