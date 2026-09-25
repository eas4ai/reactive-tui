use reactive_tui::Props;

#[derive(Props, Clone, PartialEq)]
struct Defaults {
    #[prop(optional, default)]
    absent: Option<String>,
    #[prop(optional, default = "guest")]
    named: Option<&'static str>,
}

fn main() {
    let values = Defaults::new();
    assert_eq!(values.absent, None);
    assert_eq!(values.named, Some("guest"));
    let values = values.with_absent(Some("provided".into())).with_named(None);
    assert_eq!(values.absent.as_deref(), Some("provided"));
    assert_eq!(values.named, None);
    assert!(values.validate());
}
