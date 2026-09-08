//! Simple test of the #[component] macro
//!
//! This demonstrates the component macro functionality.

use reactive_tui::prelude::*;

/// Simple component without props
#[component]
fn HelloWorld(hooks: &Hooks) -> Element {
    Element::text("Hello from a macro component! 🎉")
}

/// Component with state management
#[component]
fn Counter(hooks: &Hooks) -> Element {
    let count = use_signal(hooks, 0);
    Element::text(format!("Count: {}", count.get()))
}

/// Component with props
#[component]
fn Greeting(hooks: &Hooks, name: String, age: Option<u32>) -> Element {
    let message = if let Some(age) = age {
        format!("Hello {}, you are {} years old!", name, age)
    } else {
        format!("Hello {}!", name)
    };
    Element::text(&message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_macro_integration() {
        // Test component creation
        let hello = HelloWorld::element();
        assert!(hello.is_component());
        assert_eq!(hello.component_name(), Some("HelloWorld"));

        let counter = Counter::element();
        assert!(counter.is_component());
        assert_eq!(counter.component_name(), Some("Counter"));

        let greeting = Greeting::element("Alice".to_string(), Some(25));
        assert!(greeting.is_component());
        assert_eq!(greeting.component_name(), Some("Greeting"));

        println!("✅ Component macro integration tests passed!");
    }
}
