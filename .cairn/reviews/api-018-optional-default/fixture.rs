use reactive_tui::Props;
#[derive(Props, Clone, PartialEq)]
struct Values {
    #[prop(optional, default)]
    absent: Option<String>,
    #[prop(optional, default = "guest")]
    named: Option<&'static str>,
}
fn main() {
    let values = Values::new();
    assert_eq!(values.absent, None);
    assert_eq!(values.named, Some("guest"));
}
