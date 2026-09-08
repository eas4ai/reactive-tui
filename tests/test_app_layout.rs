use reactive_tui::app::RootComponent;
use reactive_tui::backend::DebugBackend;
use reactive_tui::prelude::*;

struct TestLayoutComponent;

impl RootComponent for TestLayoutComponent {
    fn render(&self) -> Element {
        div()
            .class("flex flex-col w-40 h-10 gap-2")
            .children(vec![
                div()
                    .class("absolute left-10 top-3 w-8 h-1")
                    .child(div().class("w-full h-full").text("ABS").build())
                    .build(),
                div()
                    .class("w-8 h-1")
                    .child(div().class("w-full h-full").text("TOP").build())
                    .build(),
                div()
                    .class("w-8 h-1")
                    .child(div().class("w-full h-full").text("BTM").build())
                    .build(),
            ])
            .build()
    }
}

#[test]
fn test_app_uses_proper_layout_system() {
    // Create a debug backend to capture output
    let backend = DebugBackend::new(50, 15);

    // Build the app - but we can't test through App since render() is private
    // Instead, test the backend directly
    use reactive_tui::backend::Backend;
    let mut backend = backend;

    // Render the component
    let element = TestLayoutComponent.render();
    backend.render_full(&element).unwrap();

    // Check positions using the screen content methods
    // "TOP" should be at top (y=0)
    let mut found_top = false;
    let mut found_abs = false;

    for y in 0..15 {
        for x in 0..50 {
            if let Some(ch) = backend.char_at(x, y) {
                if x == 0 && y == 0 && ch == 'T' {
                    // Check if TOP is at the top
                    if let (Some('O'), Some('P')) =
                        (backend.char_at(x + 1, y), backend.char_at(x + 2, y))
                    {
                        found_top = true;
                        println!("Found TOP at ({}, {})", x, y);
                    }
                }
                if x == 10 && y == 3 && ch == 'A' {
                    // Check if ABS is at absolute position
                    if let (Some('B'), Some('S')) =
                        (backend.char_at(x + 1, y), backend.char_at(x + 2, y))
                    {
                        found_abs = true;
                        println!("Found ABS at ({}, {})", x, y);
                    }
                }
            }
        }
    }

    // Print the screen for debugging
    println!("\nRendered screen:");
    for y in 0..8 {
        print!("|");
        for x in 0..20 {
            if let Some(ch) = backend.char_at(x, y) {
                print!("{}", ch);
            } else {
                print!(" ");
            }
        }
        println!("|");
    }

    assert!(found_top, "TOP text should be at the top of the screen");
    assert!(found_abs, "ABS text should be at absolute position (10, 3)");
}
