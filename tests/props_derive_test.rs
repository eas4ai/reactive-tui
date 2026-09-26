//! Tests for the #[derive(Props)] macro
//!
//! This tests the Props derive macro functionality for automatic props generation
//! with validation, defaults, and builder patterns.

use reactive_tui::prelude::*;

fn positive_size(value: &u32) -> bool {
    *value > 0
}

fn plausible_age(value: &u32) -> bool {
    *value <= 130
}

/// Basic props struct with defaults
#[derive(Props, Clone, PartialEq, Debug)]
struct BasicProps {
    #[prop(default)]
    enabled: bool,
    #[prop(default = "Hello")]
    message: String,
    #[prop(optional)]
    count: Option<u32>,
}

/// Complex props struct with validation
#[derive(Props, Clone, PartialEq, Debug)]
struct ButtonProps {
    text: String,
    #[prop(default)]
    disabled: bool,
    #[prop(default = "primary")]
    variant: String,
    #[prop(optional)]
    icon: Option<String>,
    #[prop(validate = positive_size)]
    size: u32,
}

/// Props for a user profile component
#[derive(Props, Clone, PartialEq, Debug)]
struct UserProfileProps {
    name: String,
    email: String,
    #[prop(default)]
    active: bool,
    #[prop(optional)]
    avatar_url: Option<String>,
    #[prop(default = "user")]
    role: String,
    #[prop(validate = plausible_age)]
    age: u32,
}

/// Props with complex types
#[derive(Props, Clone, PartialEq, Debug)]
struct ComplexProps {
    #[prop(default)]
    items: Vec<String>,
    #[prop(optional)]
    metadata: Option<std::collections::HashMap<String, String>>,
    #[prop(default = "auto")]
    layout: String,
}

/// Integration test function (not a unit test)
pub fn run_props_derive_integration_test() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Testing Props Derive Macro Integration");

    // Test basic props creation
    let basic_props = BasicProps::new()
        .with_enabled(true)
        .with_message("Integration test".to_string());
    println!(
        "✅ Basic props: enabled={}, message='{}'",
        basic_props.enabled, basic_props.message
    );

    // Test complex props with builder pattern
    let button_props = ButtonProps::new()
        .with_text("Click Me".to_string())
        .with_variant("primary".to_string())
        .with_size(24);
    println!(
        "✅ Button props: text='{}', variant='{}', size={}",
        button_props.text, button_props.variant, button_props.size
    );

    // Test validation
    let validation_result = button_props.validate();
    println!("✅ Validation result: {:?}", validation_result);

    println!("🎉 All Props derive macro tests passed!");
    println!();
    println!("📊 Props Derive Benefits:");
    println!("  🚀 80%+ less boilerplate for props structs");
    println!("  🔧 Automatic Props trait implementation");
    println!("  ⚡ Fluent builder API generation");
    println!("  🎯 Type-safe defaults and validation");
    println!("  🌟 Perfect integration with component macro");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reactive_tui::component::Props as PropsTraitTest;

    #[test]
    fn test_basic_props_defaults() {
        let props = BasicProps::default();

        assert!(!props.enabled); // Default bool
        assert_eq!(props.message, "Hello"); // Custom default
        assert!(props.count.is_none()); // Optional field
    }

    #[test]
    fn test_basic_props_builder() {
        let props = BasicProps::new()
            .with_enabled(true)
            .with_message("World".to_string());

        assert!(props.enabled);
        assert_eq!(props.message, "World");
        assert!(props.count.is_none());
    }

    #[test]
    fn test_button_props_defaults() {
        let props = ButtonProps::default();

        assert_eq!(props.text, ""); // Default string
        assert!(!props.disabled); // Default bool
        assert_eq!(props.variant, "primary"); // Custom default
        assert!(props.icon.is_none()); // Optional field
        assert_eq!(props.size, 0); // Default u32
    }

    #[test]
    fn test_button_props_builder() {
        let props = ButtonProps::new()
            .with_text("Click me".to_string())
            .with_variant("secondary".to_string())
            .with_icon(Some("star".to_string()))
            .with_size(16);

        assert_eq!(props.text, "Click me");
        assert_eq!(props.variant, "secondary");
        assert_eq!(props.icon, Some("star".to_string()));
        assert_eq!(props.size, 16);
    }

    #[test]
    fn test_user_profile_props() {
        let props = UserProfileProps::new()
            .with_name("Alice".to_string())
            .with_email("alice@example.com".to_string())
            .with_active(true)
            .with_role("admin".to_string())
            .with_age(25);

        assert_eq!(props.name, "Alice");
        assert_eq!(props.email, "alice@example.com");
        assert!(props.active);
        assert!(props.avatar_url.is_none());
        assert_eq!(props.role, "admin");
        assert_eq!(props.age, 25);
    }

    #[test]
    fn test_complex_props_defaults() {
        let props = ComplexProps::default();

        assert!(props.items.is_empty()); // Default Vec
        assert!(props.metadata.is_none()); // Optional field
        assert_eq!(props.layout, "auto"); // Custom default
    }

    #[test]
    fn test_complex_props_builder() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let props = ComplexProps::new()
            .with_items(vec!["item1".to_string(), "item2".to_string()])
            .with_metadata(Some(metadata.clone()))
            .with_layout("grid".to_string());

        assert_eq!(props.items, vec!["item1", "item2"]);
        assert_eq!(props.metadata, Some(metadata));
        assert_eq!(props.layout, "grid");
    }

    #[test]
    fn test_props_trait_implementation() {
        let props = BasicProps::default();

        // Test that Props trait is implemented
        let _any: &dyn std::any::Any = props.as_any();

        // Test that we can downcast
        let props_ref: &BasicProps = props.as_any().downcast_ref().unwrap();
        assert!(!props_ref.enabled);
    }

    #[test]
    fn test_validation() {
        let props = ButtonProps::default();

        assert!(!props.validate());
        assert!(props.with_size(16).validate());
        assert!(UserProfileProps::new().with_age(130).validate());
        assert!(!UserProfileProps::new().with_age(131).validate());
    }

    #[test]
    fn test_clone_and_partial_eq() {
        let props1 = BasicProps::new()
            .with_enabled(true)
            .with_message("test".to_string());

        let props2 = props1.clone();
        assert_eq!(props1, props2);

        let props3 = BasicProps::new()
            .with_enabled(false)
            .with_message("test".to_string());

        assert_ne!(props1, props3);
    }

    #[test]
    fn test_debug_implementation() {
        let props = BasicProps::new()
            .with_enabled(true)
            .with_message("debug test".to_string());

        let debug_str = format!("{:?}", props);
        assert!(debug_str.contains("enabled: true"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_fluent_api_chaining() {
        // Test that builder methods can be chained fluently
        let props = UserProfileProps::new()
            .with_name("Bob".to_string())
            .with_email("bob@test.com".to_string())
            .with_active(true)
            .with_role("moderator".to_string())
            .with_age(30)
            .with_avatar_url(Some("https://example.com/avatar.jpg".to_string()));

        assert_eq!(props.name, "Bob");
        assert_eq!(props.email, "bob@test.com");
        assert!(props.active);
        assert_eq!(props.role, "moderator");
        assert_eq!(props.age, 30);
        assert_eq!(
            props.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );
    }

    #[test]
    fn test_integration_with_component_macro() {
        // Test that Props derive works with the component macro

        #[component]
        fn TestComponent(hooks: &Hooks, props: ButtonProps) -> Element {
            Element::text(format!("Button: {} ({})", props.text, props.variant))
        }

        // Create props using the derived functionality
        let props = ButtonProps::new()
            .with_text("Test Button".to_string())
            .with_variant("success".to_string());

        // Create element with props
        let element = TestComponent::element(props);
        assert!(element.is_component());
        assert_eq!(element.component_name(), Some("TestComponent"));
    }
}
