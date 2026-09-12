use reactive_tui::prelude::*;

fn non_blank(value: &str) -> bool {
    !value.trim().is_empty()
}

#[derive(Props, Clone, PartialEq)]
struct NamedProps {
    #[prop(validate = non_blank)]
    name: String,
    #[prop(optional)]
    subtitle: Option<String>,
}

fn main() {
    let invalid = NamedProps::new();
    let valid = NamedProps::new().with_name("Ada".into());
    assert!(!non_blank(&invalid.name));
    assert!(non_blank(&valid.name));
    println!("explicit rule: invalid={}, valid={}", invalid.validate(), valid.validate());
    assert!(!invalid.validate(), "Props ignored the explicitly supplied rule");
    assert!(valid.validate());
}
