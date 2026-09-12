//! The approved API-018 contract validates only on explicit caller request.
use reactive_tui::Props;
use std::sync::atomic::{AtomicUsize, Ordering};

static NAME_CHECKS: AtomicUsize = AtomicUsize::new(0);

fn non_blank(value: &str) -> bool {
    NAME_CHECKS.fetch_add(1, Ordering::SeqCst);
    !value.trim().is_empty()
}

mod rules {
    pub fn positive(value: &u32) -> bool {
        *value > 0
    }

    pub fn optional_non_blank(value: &Option<String>) -> bool {
        value.as_ref().is_none_or(|value| !value.trim().is_empty())
    }
}

#[derive(Props, Clone, PartialEq, Debug)]
struct CheckedProps {
    #[prop(default = "guest", validate = non_blank)]
    name: String,
    #[prop(validate = rules::positive)]
    size: u32,
    #[prop(optional, validate = rules::optional_non_blank)]
    label: std::option::Option<String>,
}

#[test]
fn explicit_validation_enforces_each_field_and_preserves_construction() {
    let defaults = CheckedProps::new();
    assert_eq!(defaults.name, "guest");
    assert_eq!(defaults.size, 0);
    assert_eq!(defaults.label, None);
    let invalid_name = defaults.clone().with_name(" \t".into()).with_size(1);
    let valid = defaults.clone().with_size(1);
    let invalid_label = valid.clone().with_label(Some(" ".into()));
    let valid_label = valid.clone().with_label(Some("Help".into()));
    assert_eq!(
        NAME_CHECKS.load(Ordering::SeqCst),
        0,
        "construction and builders must not invoke predicates"
    );
    assert!(!defaults.validate(), "zero size violates its named rule");
    assert!(
        !invalid_name.validate(),
        "the String predicate receives a shared reference"
    );
    assert!(
        !invalid_label.validate(),
        "a present optional value must satisfy its rule"
    );
    assert!(valid.validate());
    assert!(valid_label.validate());
    assert_eq!(NAME_CHECKS.load(Ordering::SeqCst), 5);
    assert_eq!(valid.name, "guest");
    assert_eq!(valid.label, None);
}

#[derive(Props, Clone, PartialEq)]
struct UnconstrainedProps {
    #[prop(default)]
    enabled: bool,
    text: String,
    #[prop(optional)]
    value: Option<u32>,
}

#[test]
fn no_rules_adds_no_constraints_and_optional_builders_keep_option_types() {
    let props = UnconstrainedProps::new();
    assert!(!props.enabled);
    assert!(props.text.is_empty());
    assert_eq!(props.value, None);
    assert!(props.validate());
    let props = props.with_value(Some(7));
    assert_eq!(props.value, Some(7));
    assert!(props.validate());
}

#[derive(Props, Clone, PartialEq)]
struct OptionalDefaults {
    #[prop(optional, default)]
    absent: Option<String>,
    #[prop(optional, default = "guest")]
    named: Option<&'static str>,
}

#[test]
fn optional_fields_preserve_explicit_default_precedence() {
    let props = OptionalDefaults::new();
    assert_eq!(props.absent, None);
    assert_eq!(props.named, Some("guest"));
    let props = props.with_absent(Some("provided".into())).with_named(None);
    assert_eq!(props.absent.as_deref(), Some("provided"));
    assert_eq!(props.named, None);
    assert!(props.validate());
}
