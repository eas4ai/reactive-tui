use reactive_tui::component::{
    registry::{global_cleanup_all, global_component_performance, register_component},
    Component, Element, Props,
};
use reactive_tui::core::render_stats::{DetailedFrameStats, RenderStatsCollector};
use reactive_tui::render::tree::element_to_render_node;
use std::sync::Mutex;
use std::time::Duration;

// Test isolation: ensure only one test runs at a time when accessing global state
static TEST_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Clone, PartialEq, Default)]
struct PerfTestProps {
    value: i32,
}

impl Props for PerfTestProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct PerfTestComponent;

impl Component for PerfTestComponent {
    type Props = PerfTestProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(format!("perf test: {}", props.value))
    }
}

#[test]
fn test_component_performance_tracking() {
    // Acquire test mutex to prevent concurrent test execution
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // Clean up any previous global state
    global_cleanup_all().unwrap();

    // Register component with unique name for this test
    let component_name = format!("PerfTestComponent_{}", std::process::id());
    register_component::<PerfTestComponent>(&component_name).unwrap();

    // Get initial metrics
    let initial_metrics = global_component_performance().unwrap();
    assert_eq!(initial_metrics.active_components, 0);

    // Create some components
    let mut components = Vec::new();
    for i in 0..10 {
        let element = Element::component_with_props(&component_name, PerfTestProps { value: i });
        let render_node = element_to_render_node(element);
        components.push(render_node);
    }

    // Check metrics after creation
    let after_creation = global_component_performance().unwrap();
    assert_eq!(
        after_creation.active_components - initial_metrics.active_components,
        10
    );
    assert_eq!(
        after_creation.total_created - initial_metrics.total_created,
        10
    );
    assert_eq!(
        after_creation.total_destroyed - initial_metrics.total_destroyed,
        0
    );

    // Verify creation time is tracked
    assert!(after_creation.avg_creation_time > Duration::ZERO);
    assert!(after_creation.total_creation_time > Duration::ZERO);

    println!("✅ Component creation metrics:");
    println!("   Active: {}", after_creation.active_components);
    println!("   Created: {}", after_creation.total_created);
    println!(
        "   Avg creation time: {:?}",
        after_creation.avg_creation_time
    );

    // Drop half the components
    components.truncate(5);

    // Check metrics after partial cleanup
    let after_partial_cleanup = global_component_performance().unwrap();
    assert_eq!(
        after_partial_cleanup.active_components - initial_metrics.active_components,
        5
    );
    assert_eq!(
        after_partial_cleanup.total_created - initial_metrics.total_created,
        10
    );
    assert_eq!(
        after_partial_cleanup.total_destroyed - initial_metrics.total_destroyed,
        5
    );

    // Verify cleanup time is tracked
    assert!(after_partial_cleanup.avg_cleanup_time > Duration::ZERO);
    assert!(after_partial_cleanup.total_cleanup_time > Duration::ZERO);

    println!("✅ Component cleanup metrics:");
    println!("   Active: {}", after_partial_cleanup.active_components);
    println!("   Destroyed: {}", after_partial_cleanup.total_destroyed);
    println!(
        "   Avg cleanup time: {:?}",
        after_partial_cleanup.avg_cleanup_time
    );

    // Drop all components
    components.clear();

    // Final metrics check
    let final_metrics = global_component_performance().unwrap();
    assert_eq!(
        final_metrics.active_components - initial_metrics.active_components,
        0
    );
    assert_eq!(
        final_metrics.total_created - initial_metrics.total_created,
        10
    );
    assert_eq!(
        final_metrics.total_destroyed - initial_metrics.total_destroyed,
        10
    );

    println!("✅ Final metrics:");
    println!("   Total created: {}", final_metrics.total_created);
    println!("   Total destroyed: {}", final_metrics.total_destroyed);
    println!(
        "   Total creation time: {:?}",
        final_metrics.total_creation_time
    );
    println!(
        "   Total cleanup time: {:?}",
        final_metrics.total_cleanup_time
    );
}

#[test]
fn test_render_stats_component_integration() {
    // Acquire test mutex to prevent concurrent test execution
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // Clean up any previous global state
    global_cleanup_all().unwrap();

    // Register component with unique name for this test
    let component_name = format!("RenderStatsComponent_{}", std::process::id());
    register_component::<PerfTestComponent>(&component_name).unwrap();

    // Create a render stats collector
    let mut collector = RenderStatsCollector::new(100);

    // Create initial frame stats
    let frame_stats = DetailedFrameStats {
        frame_time: Duration::from_millis(16),
        timestamp: std::time::Instant::now(),
        ..Default::default()
    };

    // Record initial frame
    collector.record_frame(frame_stats.clone());

    // Get initial metrics (not used but captured for baseline)
    let _initial_metrics = global_component_performance().unwrap();

    // Create some components and update frame stats
    let mut components = Vec::new();
    for i in 0..5 {
        let element = Element::component_with_props(&component_name, PerfTestProps { value: i });
        let render_node = element_to_render_node(element);
        components.push(render_node);
    }

    // Get component metrics
    let comp_metrics = global_component_performance().unwrap();

    // Update frame stats with component metrics
    collector.update_component_metrics(
        comp_metrics.total_created as u32,
        comp_metrics.total_destroyed as u32,
        comp_metrics.active_components,
        comp_metrics.total_creation_time,
        comp_metrics.total_cleanup_time,
    );

    // Get performance metrics from collector
    let perf_metrics = collector.get_metrics();
    assert_eq!(
        perf_metrics.active_components,
        comp_metrics.active_components
    );
    assert_eq!(
        perf_metrics.total_components_created,
        comp_metrics.total_created
    );
    assert_eq!(
        perf_metrics.total_components_destroyed,
        comp_metrics.total_destroyed
    );

    // Get component performance summary
    let summary = collector.component_performance_summary();
    assert_eq!(summary.active_components, comp_metrics.active_components);
    assert_eq!(summary.total_created, comp_metrics.total_created);
    assert_eq!(summary.total_destroyed, comp_metrics.total_destroyed);
    assert!(summary.creation_efficiency > 0.0);

    println!("✅ Render stats integration:");
    println!("   Active components: {}", summary.active_components);
    println!(
        "   Creation efficiency: {:.2} components/ms",
        summary.creation_efficiency
    );
    println!(
        "   Creation rate: {:.2} components/sec",
        summary.creation_rate
    );

    // Cleanup components
    components.clear();

    // Update metrics again
    let final_comp_metrics = global_component_performance().unwrap();
    collector.update_component_metrics(
        0, // no new components created
        final_comp_metrics.total_destroyed as u32 - comp_metrics.total_destroyed as u32, // newly destroyed
        final_comp_metrics.active_components,
        Duration::ZERO, // no new creation time
        final_comp_metrics.total_cleanup_time - comp_metrics.total_cleanup_time, // new cleanup time
    );

    let final_summary = collector.component_performance_summary();
    assert_eq!(
        final_summary.active_components,
        final_comp_metrics.active_components
    );
    assert_eq!(
        final_summary.total_destroyed,
        final_comp_metrics.total_destroyed
    );

    println!("✅ Final render stats:");
    println!("   Components destroyed: {}", final_summary.total_destroyed);
    println!(
        "   Cleanup rate: {:.2} components/sec",
        final_summary.cleanup_rate
    );
}

#[test]
fn test_performance_metrics_accuracy() {
    // Acquire test mutex to prevent concurrent test execution
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|p| p.into_inner());

    // Clean up any previous global state
    global_cleanup_all().unwrap();

    // Register component with unique name for this test
    let component_name = format!("AccuracyTestComponent_{}", std::process::id());
    register_component::<PerfTestComponent>(&component_name).unwrap();

    // Get initial baseline metrics
    let baseline_metrics = global_component_performance().unwrap();

    // Create components in batches and measure
    let batch_size = 10;
    let num_batches = 2;
    let mut all_components = Vec::new();
    let mut total_destroyed = 0;

    for batch in 0..num_batches {
        let mut batch_components = Vec::new();

        for i in 0..batch_size {
            let element = Element::component_with_props(
                &component_name,
                PerfTestProps {
                    value: batch * batch_size + i,
                },
            );
            let render_node = element_to_render_node(element);
            batch_components.push(render_node);
        }

        // Check metrics after each batch
        let metrics = global_component_performance().unwrap();
        let expected_created = (batch + 1) * batch_size;
        let expected_active = expected_created - total_destroyed;
        assert_eq!(
            (metrics.total_created - baseline_metrics.total_created) as i32,
            expected_created
        );
        assert_eq!(
            (metrics.active_components - baseline_metrics.active_components) as i32,
            expected_active
        );

        // Cleanup half of this batch
        let to_destroy = batch_size / 2;
        batch_components.truncate((batch_size - to_destroy) as usize);
        total_destroyed += to_destroy;

        let after_cleanup = global_component_performance().unwrap();
        let expected_active_after = expected_created - total_destroyed;
        assert_eq!(
            (after_cleanup.active_components - baseline_metrics.active_components) as i32,
            expected_active_after
        );

        // Keep remaining components for final verification
        all_components.extend(batch_components);

        println!(
            "Batch {}: Created {}, Active {}, Destroyed {}",
            batch,
            after_cleanup.total_created,
            after_cleanup.active_components,
            after_cleanup.total_destroyed
        );
    }

    let final_metrics = global_component_performance().unwrap();

    // Verify final counts relative to baseline
    assert_eq!(
        (final_metrics.total_created - baseline_metrics.total_created) as i32,
        num_batches * batch_size
    );
    assert_eq!(
        (final_metrics.total_destroyed - baseline_metrics.total_destroyed) as i32,
        num_batches * (batch_size / 2)
    );
    assert_eq!(
        (final_metrics.active_components - baseline_metrics.active_components) as i32,
        num_batches * (batch_size / 2)
    );

    // Verify timing metrics are reasonable
    assert!(final_metrics.avg_creation_time < Duration::from_millis(10)); // Should be fast
    assert!(final_metrics.avg_cleanup_time < Duration::from_millis(10)); // Should be fast

    println!("✅ Performance metrics accuracy test passed:");
    println!("   Total created: {}", final_metrics.total_created);
    println!("   Total destroyed: {}", final_metrics.total_destroyed);
    println!("   Active: {}", final_metrics.active_components);
    println!(
        "   Avg creation time: {:?}",
        final_metrics.avg_creation_time
    );
    println!("   Avg cleanup time: {:?}", final_metrics.avg_cleanup_time);
}
