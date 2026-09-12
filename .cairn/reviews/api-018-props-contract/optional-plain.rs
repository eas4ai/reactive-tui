use reactive_tui::prelude::*;

#[derive(Props, Clone, PartialEq)]
struct OptionalPlain {
    #[prop(optional)]
    name: String,
}

fn main() {
    let _ = OptionalPlain::new();
}
