use reactive_tui::component::{
    registry::{global_active_count, global_cleanup_all, global_clear_all, register_component},
    Component, Element, LifecycleEvent, Props,
};
use reactive_tui::render::{tree::element_to_render_node, Reconciler, RenderTree};
use serial_test::serial;
use std::sync::atomic::{AtomicU32, Ordering};

// Test component with lifecycle tracking
static MOUNT_COUNT: AtomicU32 = AtomicU32::new(0);
static UNMOUNT_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, PartialEq, Default)]
struct TestProps {
    value: i32,
}

impl Props for TestProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct TestComponent {
    props: TestProps,
}

impl Component for TestComponent {
    type Props = TestProps;
    type State = ();

    fn new(props: Self::Props) -> Self {
        Self { props }
    }

    fn render(&self, _props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(format!("value: {}", self.props.value))
    }

    fn on_lifecycle(&mut self, event: LifecycleEvent, _state: &mut Self::State) {
        match event {
            LifecycleEvent::Mount => {
                MOUNT_COUNT.fetch_add(1, Ordering::Relaxed);
            }
            LifecycleEvent::Unmount => {
                UNMOUNT_COUNT.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

fn reset_counters() {
    MOUNT_COUNT.store(0, Ordering::Relaxed);
    UNMOUNT_COUNT.store(0, Ordering::Relaxed);
}

fn get_mount_count() -> u32 {
    MOUNT_COUNT.load(Ordering::Relaxed)
}

fn get_unmount_count() -> u32 {
    UNMOUNT_COUNT.load(Ordering::Relaxed)
}

#[test]
#[serial]
fn test_automatic_component_instantiation() {
    reset_counters();
    global_clear_all().unwrap();

    // Register component with unique name for this test
    register_component::<TestComponent>("TestComponentAutoInst").unwrap();

    // Create element with component
    let element = Element::component_with_props("TestComponentAutoInst", TestProps { value: 42 });

    // Convert to render node - should automatically instantiate component
    let render_node = element_to_render_node(element);

    // Verify component was created and registered
    assert_eq!(global_active_count().unwrap(), 1);
    assert_eq!(
        get_mount_count(),
        1,
        "Mount count should be 1, got {}",
        get_mount_count()
    );
    assert_eq!(
        get_unmount_count(),
        0,
        "Unmount count should be 0, got {}",
        get_unmount_count()
    );

    // Drop render node - should automatically cleanup
    drop(render_node);

    // Verify component was cleaned up
    assert_eq!(global_active_count().unwrap(), 0);
    assert_eq!(
        get_unmount_count(),
        1,
        "Unmount count should be 1 after drop, got {}",
        get_unmount_count()
    );
}

#[test]
#[serial]
fn test_automatic_cleanup_during_reconciliation() {
    reset_counters();
    global_clear_all().unwrap();

    register_component::<TestComponent>("TestComponentCleanup").unwrap();

    let mut reconciler = Reconciler::new();

    // Create initial tree with component
    let element1 = Element::component_with_props("TestComponentCleanup", TestProps { value: 1 });
    let root_node1 = element_to_render_node(element1);
    let mut tree1 = RenderTree::new();
    tree1.set_root(root_node1);

    assert_eq!(global_active_count().unwrap(), 1);
    assert_eq!(get_mount_count(), 1);

    // Create new tree without component
    let element2 = Element::text("No component");
    let _root_node2 = element_to_render_node(element2);
    let tree2 = RenderTree::new();
    // Note: We don't set root on tree2 to simulate component removal

    // Perform reconciliation
    let diff_result = reconciler.diff(&tree1, &tree2);

    // Apply patches - should automatically cleanup component
    let _ = reactive_tui::render::reconcile::apply_patches(&diff_result.patches, &mut tree1);

    // Verify component was cleaned up during reconciliation
    assert_eq!(global_active_count().unwrap(), 0);
    assert_eq!(get_unmount_count(), 1);
}

#[test]
#[serial]
fn test_multiple_components_lifecycle() {
    reset_counters();
    global_clear_all().unwrap();

    register_component::<TestComponent>("TestComponentMultiple").unwrap();

    // Create multiple components
    let elements = vec![
        Element::component_with_props("TestComponentMultiple", TestProps { value: 1 }),
        Element::component_with_props("TestComponentMultiple", TestProps { value: 2 }),
        Element::component_with_props("TestComponentMultiple", TestProps { value: 3 }),
    ];

    let render_nodes: Vec<_> = elements.into_iter().map(element_to_render_node).collect();

    // Verify all components were created
    assert_eq!(global_active_count().unwrap(), 3);
    assert_eq!(get_mount_count(), 3);
    assert_eq!(get_unmount_count(), 0);

    // Drop all nodes
    drop(render_nodes);

    // Verify all components were cleaned up
    assert_eq!(global_active_count().unwrap(), 0);
    assert_eq!(get_unmount_count(), 3);
}

#[test]
#[serial]
fn test_component_replacement() {
    reset_counters();
    global_clear_all().unwrap();

    register_component::<TestComponent>("TestComponentReplace").unwrap();

    let _reconciler = Reconciler::new();

    // Create tree with one component
    let element1 = Element::component_with_props("TestComponentReplace", TestProps { value: 1 })
        .with_key("test-key".to_string());
    let root_node1 = element_to_render_node(element1);
    let mut tree1 = RenderTree::new();
    tree1.set_root(root_node1);

    assert_eq!(global_active_count().unwrap(), 1);
    assert_eq!(get_mount_count(), 1);

    // Create tree with different component (different key to avoid collision)
    let element2 = Element::component_with_props("TestComponentReplace", TestProps { value: 2 })
        .with_key("test-key-2".to_string());
    let root_node2 = element_to_render_node(element2);
    let mut tree2 = RenderTree::new();
    tree2.set_root(root_node2);

    // Both trees have their own component instances
    assert_eq!(
        global_active_count().unwrap(),
        2,
        "Should have 2 active components (one per tree)"
    );

    // Cleanup
    global_cleanup_all().unwrap();
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
#[serial]
fn test_memory_leak_prevention_under_load() {
    reset_counters();
    global_clear_all().unwrap();

    register_component::<TestComponent>("TestComponentMemLeak").unwrap();

    // Create and destroy many components to test for memory leaks
    for i in 0..1000 {
        let element = Element::component_with_props("TestComponentMemLeak", TestProps { value: i });
        let render_node = element_to_render_node(element);

        // Verify component was created
        assert_eq!(global_active_count().unwrap(), 1);

        // Drop immediately
        drop(render_node);

        // Verify component was cleaned up
        assert_eq!(global_active_count().unwrap(), 0);
    }

    // Verify all lifecycle events were called correctly
    assert_eq!(get_mount_count(), 1000);
    assert_eq!(get_unmount_count(), 1000);
}

#[test]
#[serial]
fn test_unregistered_component_handling() {
    reset_counters();
    global_cleanup_all().unwrap();

    // Don't register the component

    // Create element with unregistered component
    let element = Element::component_with_props("UnregisteredComponent", TestProps { value: 42 });
    let render_node = element_to_render_node(element);

    // Should not create any component instances
    assert_eq!(global_active_count().unwrap(), 0);
    assert_eq!(get_mount_count(), 0);

    // Should not crash when dropped
    drop(render_node);
    assert_eq!(global_active_count().unwrap(), 0);
}

#[test]
#[serial]
fn test_global_cleanup() {
    reset_counters();
    global_clear_all().unwrap();

    register_component::<TestComponent>("TestComponentGlobal").unwrap();

    // Create multiple components without dropping them
    let _nodes: Vec<_> = (0..10)
        .map(|i| {
            let element =
                Element::component_with_props("TestComponentGlobal", TestProps { value: i });
            element_to_render_node(element)
        })
        .collect();

    // Verify components were created
    assert_eq!(global_active_count().unwrap(), 10);
    assert_eq!(get_mount_count(), 10);

    // Global cleanup should clean up all components
    let cleaned_count = global_cleanup_all().unwrap();
    assert_eq!(cleaned_count, 10);
    assert_eq!(global_active_count().unwrap(), 0);
    assert_eq!(get_unmount_count(), 10);
}
