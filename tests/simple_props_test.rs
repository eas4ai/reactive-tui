//! Simple test for Props derive macro

use reactive_tui::prelude::*;

#[derive(Props, Clone, PartialEq)]
struct SimpleProps {
    name: String,
    #[prop(default)]
    enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_props() {
        let props = SimpleProps::default();
        assert!(!props.enabled);
    }
}
