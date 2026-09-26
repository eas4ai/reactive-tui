use reactive_tui::component::{
    registry::{global_cleanup_all, global_component_performance, register_component},
    Component, Element, Props,
};
use reactive_tui::render::tree::element_to_render_node;
use std::sync::Mutex;
use std::time::Duration;

// These tests assert whole-global-registry counts, so isolate their full
// setup/create/assert/drop transactions. registry_concurrency tests races.
static GLOBAL_METRICS_TEST: Mutex<()> = Mutex::new(());

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
        Element::text(format!("simple: {}", props.value))
    }
}

#[test]
fn test_performance_metrics_basic() {
    let _test_guard = GLOBAL_METRICS_TEST.lock().unwrap();
    global_cleanup_all().unwrap();
    register_component::<SimpleComponent>("SimpleComponent").unwrap();

    // Get initial metrics
    let initial = global_component_performance().unwrap();
    println!(
        "Initial metrics: active={}, created={}, destroyed={}",
        initial.active_components, initial.total_created, initial.total_destroyed
    );

    // Create one component
    let element = Element::component_with_props("SimpleComponent", SimpleProps { value: 42 });
    let _render_node = element_to_render_node(element);

    // Check metrics
    let after_create = global_component_performance().unwrap();
    println!(
        "After create: active={}, created={}, destroyed={}",
        after_create.active_components, after_create.total_created, after_create.total_destroyed
    );

    assert_eq!(after_create.active_components, 1);
    assert!(after_create.total_created >= 1);
    assert!(after_create.avg_creation_time >= Duration::ZERO);

    println!("✅ Basic performance metrics test passed");
}

#[test]
fn test_performance_metrics_cleanup() {
    let _test_guard = GLOBAL_METRICS_TEST.lock().unwrap();
    global_cleanup_all().unwrap();
    register_component::<SimpleComponent>("CleanupComponent").unwrap();

    // Create and immediately drop a component
    {
        let element = Element::component_with_props("CleanupComponent", SimpleProps { value: 123 });
        let _render_node = element_to_render_node(element);
        // render_node goes out of scope here
    }

    // Check metrics
    let metrics = global_component_performance().unwrap();
    println!(
        "After cleanup: active={}, created={}, destroyed={}",
        metrics.active_components, metrics.total_created, metrics.total_destroyed
    );

    assert_eq!(metrics.active_components, 0);
    assert!(metrics.total_created >= 1);
    assert!(metrics.total_destroyed >= 1);
    assert!(metrics.avg_cleanup_time >= Duration::ZERO);

    println!("✅ Performance metrics cleanup test passed");
}
