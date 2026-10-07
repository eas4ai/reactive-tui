use reactive_tui::app::{App, RootComponent};
use reactive_tui::backend::DebugBackend;
use reactive_tui::component::Element;
use reactive_tui::event::router::NodeId;
use reactive_tui::event::types::KeyCode;
use std::sync::{Arc, Mutex};

/// Test component that tracks its lifecycle and interactions
#[derive(Clone)]
struct TestInteractiveComponent {
    state: Arc<Mutex<ComponentState>>,
}

#[derive(Default, Clone)]
struct ComponentState {
    render_count: usize,
    key_events: Vec<KeyCode>,
    mouse_events: Vec<(i32, i32)>,
    focused: bool,
}

impl TestInteractiveComponent {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ComponentState::default())),
        }
    }

    fn get_state(&self) -> ComponentState {
        self.state.lock().unwrap().clone()
    }
}

impl RootComponent for TestInteractiveComponent {
    fn render(&self) -> Element {
        let mut state = self.state.lock().unwrap();
        state.render_count += 1;

        Element::text(format!(
            "Renders: {} | Keys: {:?} | Mouse: {:?} | Focus: {}",
            state.render_count,
            state.key_events,
            state.mouse_events,
            if state.focused {
                "FOCUSED"
            } else {
                "NOT FOCUSED"
            }
        ))
    }
}

/// Integration test framework for full pipeline testing
struct PipelineTestFramework {
    app: App,
    component: TestInteractiveComponent,
}

impl PipelineTestFramework {
    fn new(width: u16, height: u16) -> Self {
        let backend = DebugBackend::new(width, height);
        let component = TestInteractiveComponent::new();

        let app = App::builder()
            .backend(backend)
            .root(component.clone())
            .build()
            .unwrap();

        Self { app, component }
    }

    /// Get the current component state
    fn component_state(&self) -> ComponentState {
        self.component.get_state()
    }

    /// Test that the app can be created and basic operations work
    fn test_basic_operations(&self) -> bool {
        // Test that we can get the app size
        let size = self.app.size();
        size.0 > 0 && size.1 > 0
    }
}

#[test]
fn test_app_initialization() {
    let framework = PipelineTestFramework::new(80, 24);

    // App should initialize successfully
    assert!(framework.test_basic_operations());

    // Component should be created but not yet rendered (renders happen during app.run())
    let state = framework.component_state();
    assert_eq!(state.render_count, 0); // No renders until app runs
    assert!(state.key_events.is_empty());
    assert!(state.mouse_events.is_empty());
}

#[test]
fn test_component_lifecycle() {
    let framework = PipelineTestFramework::new(80, 24);

    // Component should be created but not rendered until app runs
    let state = framework.component_state();
    assert_eq!(state.render_count, 0);

    // Test that we can call render() on the component directly
    let _element = framework.component.render();

    // After direct render call, count should increase
    let state = framework.component_state();
    assert_eq!(state.render_count, 1);
}

#[test]
fn test_app_size_integration() {
    let framework = PipelineTestFramework::new(100, 50);

    // App should report correct size
    let size = framework.app.size();
    assert_eq!(size, (100, 50));
}

#[test]
fn test_focus_management_api() {
    let mut framework = PipelineTestFramework::new(80, 24);

    // Test focus management API
    let node_id = NodeId::new();
    assert!(
        framework.app.register_focusable(node_id, Some(0)),
        "node registers as focusable"
    );
    assert!(
        framework.app.set_initial_focus(node_id),
        "initial focus lands on the node"
    );
    assert_eq!(framework.app.current_focus(), Some(&node_id));
}

#[test]
fn test_multiple_app_instances() {
    // Test that multiple app instances can coexist
    let framework1 = PipelineTestFramework::new(80, 24);
    let framework2 = PipelineTestFramework::new(100, 30);

    assert_eq!(framework1.app.size(), (80, 24));
    assert_eq!(framework2.app.size(), (100, 30));

    // Both should have components in initial state
    assert_eq!(framework1.component_state().render_count, 0);
    assert_eq!(framework2.component_state().render_count, 0);
}

#[test]
fn test_component_state_isolation() {
    let framework1 = PipelineTestFramework::new(80, 24);
    let framework2 = PipelineTestFramework::new(80, 24);

    let state1 = framework1.component_state();
    let state2 = framework2.component_state();

    // Each component should have its own state
    assert_eq!(state1.render_count, state2.render_count);
    assert_eq!(state1.key_events, state2.key_events);
}

#[test]
fn test_app_builder_pattern() {
    // Test that the App builder pattern works correctly
    let backend = DebugBackend::new(80, 24);
    let component = TestInteractiveComponent::new();

    let app = App::builder()
        .backend(backend)
        .root(component)
        .debug(true)
        .build();

    assert!(app.is_ok());
    let app = app.unwrap();
    assert_eq!(app.size(), (80, 24));
}

#[test]
fn test_error_handling_in_builder() {
    // Test error handling in the builder pattern
    let component = TestInteractiveComponent::new();

    // Building without backend should fail
    let result = App::builder().root(component).build();

    assert!(result.is_err());
}

#[test]
fn test_component_render_consistency() {
    let framework = PipelineTestFramework::new(80, 24);

    // Multiple renders should produce consistent results
    let element1 = framework.component.render();
    let element2 = framework.component.render();

    // Both elements should be valid (we can't easily compare them, but they shouldn't panic)
    // Since the component state changes between renders, the text content will be different
    // but both should be Text elements
    match (&element1.element_type, &element2.element_type) {
        (
            reactive_tui::component::ElementType::Text(_),
            reactive_tui::component::ElementType::Text(_),
        ) => {
            // Both are text elements, which is expected
        }
        _ => panic!("Expected both elements to be Text type"),
    }

    // Render count should increase with each call
    let state = framework.component_state();
    assert_eq!(state.render_count, 2);
}

#[test]
fn test_app_with_different_backends() {
    // Test that the app works with different backend configurations
    let backend1 = DebugBackend::new(80, 24);
    let backend2 = DebugBackend::new(120, 40);

    let component1 = TestInteractiveComponent::new();
    let component2 = TestInteractiveComponent::new();

    let app1 = App::builder()
        .backend(backend1)
        .root(component1)
        .build()
        .unwrap();

    let app2 = App::builder()
        .backend(backend2)
        .root(component2)
        .build()
        .unwrap();

    assert_eq!(app1.size(), (80, 24));
    assert_eq!(app2.size(), (120, 40));
}

#[test]
fn test_focus_management_multiple_nodes() {
    let mut framework = PipelineTestFramework::new(80, 24);

    // Create multiple focusable nodes
    let node1 = NodeId::new();
    let node2 = NodeId::new();
    let node3 = NodeId::new();

    assert!(framework.app.register_focusable(node1, Some(0)));
    assert!(framework.app.register_focusable(node2, Some(1)));
    assert!(framework.app.register_focusable(node3, Some(2)));

    // Set initial focus
    assert!(framework.app.set_initial_focus(node1));
    assert_eq!(framework.app.current_focus(), Some(&node1));

    // Focus moves to each later node in turn
    assert!(framework.app.set_initial_focus(node2));
    assert_eq!(framework.app.current_focus(), Some(&node2));
    assert!(framework.app.set_initial_focus(node3));
    assert_eq!(framework.app.current_focus(), Some(&node3));
}

#[test]
fn test_component_state_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let component = Arc::new(TestInteractiveComponent::new());
    let component_clone = component.clone();

    // Test that component state can be accessed from multiple threads
    let handle = thread::spawn(move || {
        let _element = component_clone.render();
        component_clone.get_state()
    });

    let _element = component.render();
    let state1 = component.get_state();
    let state2 = handle.join().unwrap();

    // Both threads should have incremented the render count
    assert!(state1.render_count > 0);
    assert!(state2.render_count > 0);
}
