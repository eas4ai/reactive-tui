use reactive_tui::prelude::*;

#[derive(Props, Clone, PartialEq)]
struct Flagged {
    #[prop(validate)]
    size: u32,
}

fn main() {
    for size in [0, 1, 24, u32::MAX] {
        println!("size={size}, validate={}", Flagged::new().with_size(size).validate());
    }
}
