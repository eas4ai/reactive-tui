use reactive_tui::component::{
    registry::{global_cleanup_all, register_component},
    Component, Element, Props,
};
use reactive_tui::layout::css::animations::extract_css_animations_from_classes;
use reactive_tui::layout::css::manager::{
    clear_all_css_animations_global, get_css_animation_stats_global,
};
use reactive_tui::render::tree::element_to_render_node;

// These integration cases reset the same process-wide registry and animations.
static ANIMATION_TEST: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[derive(Clone, PartialEq, Default)]
struct DebugProps {
    text: String,
}

impl Props for DebugProps {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct DebugComponent;

impl Component for DebugComponent {
    type Props = DebugProps;
    type State = ();

    fn new(_props: Self::Props) -> Self {
        Self
    }

    fn render(&self, props: &Self::Props, _state: &Self::State) -> Element {
        Element::text(&props.text)
    }
}

#[test]
fn test_animate_none_parsing() {
    println!("🔍 Testing animate-none parsing directly");

    // Test the parsing function directly
    let test_cases = vec![
        ("animate-none", vec![]),
        ("animate-pulse", vec!["pulse".to_string()]),
        ("animate-none animate-pulse", vec![]), // animate-none should override
        ("animate-pulse animate-none", vec![]), // animate-none should override
        ("bg-blue-500 animate-none text-white", vec![]),
        (
            "bg-blue-500 animate-pulse text-white",
            vec!["pulse".to_string()],
        ),
    ];

    for (class_str, expected) in test_cases {
        let result = extract_css_animations_from_classes(class_str);
        println!(
            "Class: '{}' -> Animations: {:?} (expected: {:?})",
            class_str, result, expected
        );
        assert_eq!(result, expected, "Failed for class string: '{}'", class_str);
    }

    println!("✅ animate-none parsing works correctly!");
}

#[test]
fn test_animate_none_integration() {
    let _isolation = ANIMATION_TEST.lock().unwrap();
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<DebugComponent>("DebugComponent").unwrap();

    println!("🔍 Testing animate-none integration");

    let initial_stats = get_css_animation_stats_global();
    println!("Initial stats: {:?}", initial_stats);

    // Test 1: Create component with animate-none
    let element = Element::component_with_props(
        "DebugComponent",
        DebugProps {
            text: "No Animation".to_string(),
        },
    )
    .with_class("animate-none bg-gray-500");

    println!("Creating component with class: 'animate-none bg-gray-500'");

    // Check what animations would be extracted
    let extracted = extract_css_animations_from_classes("animate-none bg-gray-500");
    println!("Extracted animations: {:?}", extracted);
    assert_eq!(extracted.len(), 0, "Should extract no animations");

    let _render_node = element_to_render_node(element);

    let final_stats = get_css_animation_stats_global();
    println!("Final stats: {:?}", final_stats);

    let animations_added = final_stats.total_animations - initial_stats.total_animations;
    println!("Animations added: {}", animations_added);

    if animations_added > 0 {
        println!("❌ UNEXPECTED: animate-none should have prevented animations!");
        println!("This suggests the issue is in the component registration, not parsing");
    } else {
        println!("✅ animate-none correctly prevented animations");
    }
}

#[test]
fn test_animate_none_vs_regular() {
    let _isolation = ANIMATION_TEST.lock().unwrap();
    global_cleanup_all().unwrap();
    clear_all_css_animations_global().unwrap();
    register_component::<DebugComponent>("CompareComponent").unwrap();

    println!("🔍 Comparing animate-none vs regular animation");

    let initial_stats = get_css_animation_stats_global();
    println!("Starting stats: {:?}", initial_stats);

    // Test 1: Regular animation
    let regular_element = Element::component_with_props(
        "CompareComponent",
        DebugProps {
            text: "Regular".to_string(),
        },
    )
    .with_class("animate-pulse");

    let _regular_node = element_to_render_node(regular_element);

    let after_regular = get_css_animation_stats_global();
    println!("After regular animation: {:?}", after_regular);

    let regular_added = after_regular.total_animations - initial_stats.total_animations;
    println!("Regular animation added: {}", regular_added);

    // Test 2: animate-none
    let none_element = Element::component_with_props(
        "CompareComponent",
        DebugProps {
            text: "None".to_string(),
        },
    )
    .with_class("animate-none");

    let _none_node = element_to_render_node(none_element);

    let after_none = get_css_animation_stats_global();
    println!("After animate-none: {:?}", after_none);

    let none_added = after_none.total_animations - after_regular.total_animations;
    println!("animate-none added: {}", none_added);

    assert!(regular_added > 0, "Regular animation should add animations");
    assert_eq!(none_added, 0, "animate-none should not add animations");

    println!("✅ Comparison test passed!");
}
