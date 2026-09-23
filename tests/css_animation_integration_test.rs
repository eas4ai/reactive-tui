use reactive_tui::component::{
    registry::{global_cleanup_all, register_component},
    Component, Element, Props,
};
use reactive_tui::layout::css::manager::{
    clear_all_css_animations_global, get_css_animation_stats_global,
};
use reactive_tui::render::tree::element_to_render_node;
use serial_test::serial;

#[derive(Clone, PartialEq, Default)]
struct AnimatedProps {
    message: String,
}

impl Props for AnimatedProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct AnimatedComponent;

impl Component for AnimatedComponent {
    type Props = AnimatedProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(&props.message)
    }
}

#[test]
#[serial]
fn test_css_animation_integration_basic() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("AnimatedComponentBasic").unwrap();

    // Get initial animation stats
    let initial_stats = get_css_animation_stats_global();
    println!("Initial CSS animation stats: {:?}", initial_stats);

    // Create an element with CSS animation classes
    let element = Element::component_with_props(
        "AnimatedComponentBasic",
        AnimatedProps {
            message: "Hello Animated World!".to_string(),
        },
    )
    .with_class("animate-pulse bg-blue-500 p-4");

    // Convert to render node (this triggers component registration with CSS animations)
    let _render_node = element_to_render_node(element);

    // Check that CSS animations were applied
    let after_stats = get_css_animation_stats_global();
    println!("After creation CSS animation stats: {:?}", after_stats);

    // Should have more active animations now
    assert!(after_stats.total_animations > initial_stats.total_animations);
    assert!(after_stats.active_components > initial_stats.active_components);

    println!("✅ Basic CSS animation integration test passed");
}

#[test]
#[serial]
fn test_multiple_css_animations() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("MultiAnimatedComponent").unwrap();

    // Create elements with different CSS animations
    let pulse_element = Element::component_with_props(
        "MultiAnimatedComponent",
        AnimatedProps {
            message: "Pulse".to_string(),
        },
    )
    .with_class("animate-pulse text-red-500");

    let bounce_element = Element::component_with_props(
        "MultiAnimatedComponent",
        AnimatedProps {
            message: "Bounce".to_string(),
        },
    )
    .with_class("animate-bounce text-green-500");

    let spin_element = Element::component_with_props(
        "MultiAnimatedComponent",
        AnimatedProps {
            message: "Spin".to_string(),
        },
    )
    .with_class("animate-spin text-blue-500");

    // Convert to render nodes
    let _pulse_node = element_to_render_node(pulse_element);
    let _bounce_node = element_to_render_node(bounce_element);
    let _spin_node = element_to_render_node(spin_element);

    // Check animation stats
    let stats = get_css_animation_stats_global();
    println!("Multiple animations stats: {:?}", stats);

    // Should have exactly 3 active components with animations
    assert_eq!(stats.active_components, 3);
    assert_eq!(stats.total_animations, 3);

    println!("✅ Multiple CSS animations test passed");
}

#[test]
#[serial]
fn test_css_animation_cleanup() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("CleanupTestComponent").unwrap();

    // Create and immediately drop an animated component
    {
        let element = Element::component_with_props(
            "CleanupTestComponent",
            AnimatedProps {
                message: "Temporary".to_string(),
            },
        )
        .with_class("animate-ping opacity-75");

        let _render_node = element_to_render_node(element);

        // Check that animations were created
        let during_stats = get_css_animation_stats_global();
        assert!(during_stats.total_animations > 0);
        assert!(during_stats.active_components > 0);

        println!("During creation stats: {:?}", during_stats);
    } // render_node goes out of scope here, triggering cleanup

    // Give some time for cleanup to happen
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Check that animations were cleaned up
    let after_cleanup_stats = get_css_animation_stats_global();
    println!("After cleanup stats: {:?}", after_cleanup_stats);

    // Should have no active animations after cleanup
    assert_eq!(after_cleanup_stats.active_components, 0);
    assert_eq!(after_cleanup_stats.total_animations, 0);

    println!("✅ CSS animation cleanup test passed");
}

#[test]
#[serial]
fn test_css_animation_parsing() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("ParsingTestComponent").unwrap();

    // Test complex class string with multiple animations
    let element = Element::component_with_props("ParsingTestComponent", AnimatedProps {
        message: "Complex".to_string(),
    })
    .with_class("flex items-center animate-pulse hover:animate-bounce bg-gradient-to-r from-blue-500 to-purple-600 animate-spin");

    let _render_node = element_to_render_node(element);

    let stats = get_css_animation_stats_global();
    println!("Complex parsing stats: {:?}", stats);

    // Should have parsed multiple animations (pulse, bounce, spin)
    assert!(stats.total_animations >= 2); // At least pulse and spin, bounce might be hover-only

    println!("✅ CSS animation parsing test passed");
}

#[test]
#[serial]
fn test_animate_none_disables_animations() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("NoAnimationComponent").unwrap();

    // Test that animate-none prevents animations
    let element = Element::component_with_props(
        "NoAnimationComponent",
        AnimatedProps {
            message: "No Animation".to_string(),
        },
    )
    .with_class("animate-none bg-gray-500");

    let _render_node = element_to_render_node(element);

    let stats = get_css_animation_stats_global();
    println!("No animation stats: {:?}", stats);

    // Should have no animations due to animate-none
    assert_eq!(stats.total_animations, 0);

    println!("✅ animate-none test passed");
}

#[test]
#[serial]
fn test_css_animation_available_types() {
    use reactive_tui::layout::css::animations::get_available_css_animations;

    let available = get_available_css_animations();
    println!("Available CSS animations: {:?}", available);

    // Should have the built-in animations
    assert!(available.contains(&"pulse".to_string()));
    assert!(available.contains(&"bounce".to_string()));
    assert!(available.contains(&"spin".to_string()));
    assert!(available.contains(&"ping".to_string()));

    // Should have at least 4 built-in animations
    assert!(available.len() >= 4);

    println!("✅ Available CSS animations test passed");
}

#[test]
#[serial]
fn test_css_animation_component_id_generation() {
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<AnimatedComponent>("IdTestComponent").unwrap();

    // Create multiple instances of the same component
    let element1 = Element::component_with_props(
        "IdTestComponent",
        AnimatedProps {
            message: "Instance 1".to_string(),
        },
    )
    .with_class("animate-pulse")
    .with_key("instance-1");

    let element2 = Element::component_with_props(
        "IdTestComponent",
        AnimatedProps {
            message: "Instance 2".to_string(),
        },
    )
    .with_class("animate-bounce")
    .with_key("instance-2");

    let _node1 = element_to_render_node(element1);
    let _node2 = element_to_render_node(element2);

    let stats = get_css_animation_stats_global();
    println!("Multiple instances stats: {:?}", stats);

    // Should have exactly 2 separate animated components
    assert_eq!(stats.active_components, 2);
    assert_eq!(stats.total_animations, 2);

    println!("✅ Component ID generation test passed");
}
