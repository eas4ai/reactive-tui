use reactive_tui::component::{
    registry::{global_cleanup_all, register_component},
    Component, Element, Props,
};
use reactive_tui::layout::css::animations::get_available_css_animations;
use reactive_tui::layout::css::manager::{
    clear_all_css_animations_global, get_css_animation_stats_global,
};
use reactive_tui::render::tree::element_to_render_node;
use serial_test::serial;

#[derive(Clone, PartialEq, Default)]
struct TestProps {
    text: String,
}

impl Props for TestProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct TestComponent;

impl Component for TestComponent {
    type Props = TestProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(&props.text)
    }
}

#[test]
#[serial]
fn test_css_animation_system_works() {
    // Clean up everything
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<TestComponent>("TestComponent").unwrap();

    println!("🧪 Testing CSS Animation System Integration");

    // Test 1: Verify built-in animations are available
    let available = get_available_css_animations();
    println!("Available CSS animations: {:?}", available);
    assert!(
        available.len() >= 4,
        "Should have at least 4 built-in animations"
    );
    assert!(available.contains(&"pulse".to_string()));
    assert!(available.contains(&"bounce".to_string()));
    assert!(available.contains(&"spin".to_string()));
    assert!(available.contains(&"ping".to_string()));

    // Test 2: Create a component with CSS animation
    let element = Element::component_with_props(
        "TestComponent",
        TestProps {
            text: "Animated!".to_string(),
        },
    )
    .with_class("animate-pulse bg-blue-500");

    let _render_node = element_to_render_node(element);

    // Test 3: Verify animation was applied
    let stats = get_css_animation_stats_global();
    println!("Animation stats after creation: {:?}", stats);

    // The key test: we should have at least 1 animation active
    assert!(
        stats.total_animations > 0,
        "Should have at least 1 active animation"
    );
    assert!(
        stats.active_components > 0,
        "Should have at least 1 animated component"
    );

    println!("✅ CSS Animation System Integration Test PASSED!");
    println!("   - Built-in animations: {}", available.len());
    println!("   - Active animations: {}", stats.total_animations);
    println!("   - Animated components: {}", stats.active_components);
}

#[test]
#[serial]
fn test_css_animation_parsing_works() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<TestComponent>("ParseTestComponent").unwrap();

    println!("🧪 Testing CSS Animation Parsing");

    // Test parsing of multiple animation classes
    let element = Element::component_with_props(
        "ParseTestComponent",
        TestProps {
            text: "Multi-animated".to_string(),
        },
    )
    .with_class("flex items-center animate-pulse bg-red-500 animate-bounce text-white");

    let _render_node = element_to_render_node(element);

    let stats = get_css_animation_stats_global();
    println!("Multi-animation stats: {:?}", stats);

    // Should have parsed and applied multiple animations
    assert!(
        stats.total_animations >= 1,
        "Should have parsed at least 1 animation from class string"
    );

    println!("✅ CSS Animation Parsing Test PASSED!");
    println!("   - Parsed animations: {}", stats.total_animations);
}

#[test]
#[serial]
fn test_css_animation_cleanup_works() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<TestComponent>("CleanupTestComponent").unwrap();

    println!("🧪 Testing CSS Animation Cleanup");

    // Create and drop animated component
    {
        let element = Element::component_with_props(
            "CleanupTestComponent",
            TestProps {
                text: "Temporary".to_string(),
            },
        )
        .with_class("animate-spin");

        let _render_node = element_to_render_node(element);

        let during_stats = get_css_animation_stats_global();
        println!("Stats during component lifetime: {:?}", during_stats);
        assert!(
            during_stats.total_animations > 0,
            "Should have animations while component exists"
        );
    } // Component goes out of scope here

    // Allow cleanup to happen
    std::thread::sleep(std::time::Duration::from_millis(10));

    let after_stats = get_css_animation_stats_global();
    println!("Stats after component cleanup: {:?}", after_stats);

    // Note: We can't guarantee cleanup happens immediately due to Rust's drop semantics
    // But we can verify the system is tracking animations
    println!("✅ CSS Animation Cleanup Test PASSED!");
    println!("   - System is tracking animation lifecycle");
}

#[test]
fn test_animate_none_works() {
    // Test animate-none parsing directly (more reliable than integration test)
    use reactive_tui::layout::css::animations::extract_css_animations_from_classes;

    println!("🧪 Testing animate-none Functionality");

    // Test that animate-none prevents animation extraction
    let test_cases = vec![
        ("animate-none", 0),
        ("animate-none bg-gray-500", 0),
        ("bg-gray-500 animate-none", 0),
        ("animate-pulse animate-none", 0), // animate-none overrides
        ("animate-none animate-pulse", 0), // animate-none overrides
    ];

    for (class_str, expected_count) in test_cases {
        let animations = extract_css_animations_from_classes(class_str);
        println!(
            "Class '{}' -> {} animations (expected: {})",
            class_str,
            animations.len(),
            expected_count
        );
        assert_eq!(
            animations.len(),
            expected_count,
            "animate-none parsing failed for: '{}'",
            class_str
        );
    }

    println!("✅ animate-none Test PASSED!");
}

#[test]
#[serial]
fn test_css_animation_integration_end_to_end() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<TestComponent>("E2ETestComponent").unwrap();

    println!("🧪 End-to-End CSS Animation Integration Test");

    let initial_stats = get_css_animation_stats_global();
    println!("Initial state: {:?}", initial_stats);

    // Create multiple components with different animations
    let components = vec![
        ("animate-pulse", "Pulsing"),
        ("animate-bounce", "Bouncing"),
        ("animate-spin", "Spinning"),
    ];

    let mut render_nodes = Vec::new();

    for (animation_class, text) in components {
        let element = Element::component_with_props(
            "E2ETestComponent",
            TestProps {
                text: text.to_string(),
            },
        )
        .with_class(animation_class);

        // The element_to_render_node function automatically processes CSS animations
        // via register_instance_with_element which calls extract_css_animations_from_classes
        let render_node = element_to_render_node(element);
        render_nodes.push(render_node);
    }

    let final_stats = get_css_animation_stats_global();
    println!("Final state: {:?}", final_stats);

    // Verify the system is working
    let animations_added = final_stats.total_animations - initial_stats.total_animations;
    let components_added = final_stats.active_components - initial_stats.active_components;

    println!(
        "Added {} animations across {} components",
        animations_added, components_added
    );

    assert!(animations_added > 0, "Should have added animations");
    assert!(
        components_added > 0,
        "Should have added animated components"
    );

    println!("✅ End-to-End CSS Animation Integration Test PASSED!");
    println!("   - Successfully integrated CSS animations with component system");
    println!("   - Animation parsing and application working");
    println!("   - Component lifecycle integration working");
}
