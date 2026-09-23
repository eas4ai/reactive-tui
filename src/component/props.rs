use std::any::Any;

/// Trait for component properties.
/// Props must be cloneable and comparable for efficient diffing.
///
/// The [`crate::Props`] derive adds `new()`, defaults, `with_<field>()` builders
/// and an inherent `validate() -> bool`. Validation is an explicit caller action;
/// neither this trait nor App invokes it. Predicates receive a shared reference
/// to their field. Rules run in field order and stop at the first false result.
/// A field without a rule imposes no extra constraint.
///
/// ```
/// use reactive_tui::Props;
/// fn non_blank(value: &str) -> bool { !value.trim().is_empty() }
/// #[derive(Props, Clone, PartialEq)]
/// struct GreetingProps {
///     #[prop(default = "guest", validate = non_blank)]
///     name: String,
///     #[prop(optional)]
///     subtitle: Option<String>,
/// }
/// let props = GreetingProps::new();
/// assert!(props.validate());
/// assert_eq!(props.subtitle, None);
/// assert!(!props.with_name(" ".into()).validate());
/// ```
///
/// Migrate bare validation annotations by naming a predicate:
///
/// ```compile_fail
/// use reactive_tui::Props;
/// #[derive(Props, Clone, PartialEq)]
/// struct InvalidProps {
///     #[prop(validate)]
///     name: String,
/// }
/// ```
///
/// Optional fields must be written as `Option<T>`; a derive cannot rewrite them:
///
/// ```compile_fail
/// use reactive_tui::Props;
/// #[derive(Props, Clone, PartialEq)]
/// struct InvalidProps {
///     #[prop(optional)]
///     name: String,
/// }
/// ```
///
/// A named rule must return `bool`:
///
/// ```compile_fail
/// use reactive_tui::Props;
/// fn invalid_rule(_: &str) -> u32 { 1 }
/// #[derive(Props, Clone, PartialEq)]
/// struct InvalidProps {
///     #[prop(validate = invalid_rule)]
///     name: String,
/// }
/// ```
pub trait Props: Clone + PartialEq + Send + Sync + 'static {
    /// Create default props
    fn default_props() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }

    /// Convert to Any for type erasure
    fn as_any(&self) -> &dyn Any;
}

/// Empty props for components that don't need properties
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EmptyProps;

impl Props for EmptyProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Common props that many components might use
#[derive(Clone, PartialEq, Debug)]
pub struct CommonProps {
    /// Unique identifier for the component
    pub id: Option<String>,
    /// CSS class names for styling
    pub class: Option<String>,
    /// Whether the component is visible
    pub visible: bool,
    /// Whether the component can receive focus
    pub focusable: bool,
}

impl Default for CommonProps {
    fn default() -> Self {
        Self {
            id: None,
            class: None,
            visible: true,
            focusable: true,
        }
    }
}

impl Props for CommonProps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Props with children support
#[derive(Clone, PartialEq, Debug)]
pub struct PropsWithChildren<P: Props> {
    /// The component's properties
    pub props: P,
    /// Child elements to render inside this component
    pub children: Vec<crate::component::Element>,
}

impl<P: Props> Props for PropsWithChildren<P> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::Element;

    #[test]
    fn test_empty_props() {
        let props1 = EmptyProps;
        let props2 = EmptyProps;

        // Should be equal
        assert_eq!(props1, props2);

        // Should be cloneable
        let props3 = props1.clone();
        assert_eq!(props1, props3);

        // Should convert to Any
        let any_ref = props1.as_any();
        assert!(any_ref.downcast_ref::<EmptyProps>().is_some());

        // Should have default
        let default_props = EmptyProps;
        assert_eq!(props1, default_props);

        // Should use default_props trait method
        let trait_default = EmptyProps::default_props();
        assert_eq!(props1, trait_default);
    }

    #[test]
    fn test_common_props() {
        let props = CommonProps {
            id: Some("test-id".to_string()),
            class: Some("test-class".to_string()),
            visible: false,
            focusable: false,
        };

        // Should be cloneable
        let cloned = props.clone();
        assert_eq!(props, cloned);

        // Should convert to Any
        let any_ref = props.as_any();
        assert!(any_ref.downcast_ref::<CommonProps>().is_some());

        // Test default
        let default_props = CommonProps::default();
        assert_eq!(default_props.id, None);
        assert_eq!(default_props.class, None);
        assert!(default_props.visible);
        assert!(default_props.focusable);

        // Should not equal default
        assert_ne!(props, default_props);
    }

    #[test]
    fn test_common_props_equality() {
        let props1 = CommonProps {
            id: Some("test".to_string()),
            class: Some("class".to_string()),
            visible: true,
            focusable: true,
        };

        let props2 = CommonProps {
            id: Some("test".to_string()),
            class: Some("class".to_string()),
            visible: true,
            focusable: true,
        };

        let props3 = CommonProps {
            id: Some("different".to_string()),
            class: Some("class".to_string()),
            visible: true,
            focusable: true,
        };

        assert_eq!(props1, props2);
        assert_ne!(props1, props3);
    }

    #[test]
    fn test_props_with_children() {
        #[derive(Clone, PartialEq, Debug)]
        struct TestProps {
            value: i32,
        }

        impl Props for TestProps {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let children = vec![Element::text("Child 1"), Element::text("Child 2")];

        let props_with_children = PropsWithChildren {
            props: TestProps { value: 42 },
            children: children.clone(),
        };

        // Should be cloneable
        let cloned = props_with_children.clone();
        assert_eq!(props_with_children, cloned);

        // Should convert to Any
        let any_ref = props_with_children.as_any();
        assert!(any_ref
            .downcast_ref::<PropsWithChildren<TestProps>>()
            .is_some());

        // Test equality
        let props_with_children2 = PropsWithChildren {
            props: TestProps { value: 42 },
            children: children.clone(),
        };
        assert_eq!(props_with_children, props_with_children2);

        // Test inequality with different props
        let props_with_children3 = PropsWithChildren {
            props: TestProps { value: 43 },
            children: children.clone(),
        };
        assert_ne!(props_with_children, props_with_children3);

        // Test inequality with different children
        let props_with_children4 = PropsWithChildren {
            props: TestProps { value: 42 },
            children: vec![Element::text("Different child")],
        };
        assert_ne!(props_with_children, props_with_children4);
    }

    #[test]
    fn test_custom_props_trait() {
        #[derive(Clone, PartialEq, Debug, Default)]
        struct CustomProps {
            name: String,
            count: usize,
            enabled: bool,
        }

        impl Props for CustomProps {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let props = CustomProps {
            name: "test".to_string(),
            count: 5,
            enabled: true,
        };

        // Should implement Props trait
        let any_ref = props.as_any();
        assert!(any_ref.downcast_ref::<CustomProps>().is_some());

        // Should use default_props
        let default_props = CustomProps::default_props();
        assert_eq!(default_props.name, "");
        assert_eq!(default_props.count, 0);
        assert!(!default_props.enabled);

        // Should be cloneable and comparable
        let cloned = props.clone();
        assert_eq!(props, cloned);
    }

    #[test]
    fn test_props_type_erasure() {
        #[derive(Clone, PartialEq, Debug)]
        struct Props1 {
            value: i32,
        }

        #[derive(Clone, PartialEq, Debug)]
        struct Props2 {
            text: String,
        }

        impl Props for Props1 {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        impl Props for Props2 {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let props1 = Props1 { value: 42 };
        let props2 = Props2 {
            text: "hello".to_string(),
        };

        // Should be able to downcast correctly
        let any1 = props1.as_any();
        let any2 = props2.as_any();

        assert!(any1.downcast_ref::<Props1>().is_some());
        assert!(any1.downcast_ref::<Props2>().is_none());

        assert!(any2.downcast_ref::<Props2>().is_some());
        assert!(any2.downcast_ref::<Props1>().is_none());
    }

    #[test]
    fn smoke_test_props_send_sync() {
        // Test that props can be sent across threads
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<EmptyProps>();
        assert_sync::<EmptyProps>();

        assert_send::<CommonProps>();
        assert_sync::<CommonProps>();

        #[derive(Clone, PartialEq)]
        struct TestProps {
            value: i32,
        }
        impl Props for TestProps {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        assert_send::<TestProps>();
        assert_sync::<TestProps>();
    }
}
