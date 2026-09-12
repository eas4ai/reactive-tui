use reactive_tui::prelude::*;

#[derive(Props, Clone, PartialEq)]
struct OptionalExplicit {
    #[prop(optional)]
    name: Option<String>,
}

fn main() {
    assert_eq!(OptionalExplicit::new().name, None);
    assert_eq!(OptionalExplicit::new().with_name(Some("Ada".into())).name.as_deref(), Some("Ada"));
    println!("Explicit Option field: None default and supplied value work");
}
