use reactive_tui::prelude::*;

struct LayoutTest;

impl reactive_tui::app::RootComponent for LayoutTest {
    fn render(&self) -> Element {
        div()
            .class("w-full h-full relative bg-gray-900")
            .children(vec![
                // Normal flow element
                div()
                    .class("w-20 h-3 bg-blue-500")
                    .child(to_element("Normal flow"))
                    .build(),
                // Absolute positioned element - should appear at (20, 5)
                div()
                    .class("absolute left-20 top-5 w-15 h-3 bg-red-500")
                    .child(to_element("Absolute @ 20,5"))
                    .build(),
                // Another normal flow element - should appear below first one
                div()
                    .class("w-20 h-3 bg-green-500")
                    .child(to_element("Second normal"))
                    .build(),
                // Flexbox row container
                div()
                    .class("flex flex-row gap-2 mt-2")
                    .children(vec![
                        div().class("w-8 h-2 bg-cyan-500").child(to_element("Flex1")).build(),
                        div().class("w-8 h-2 bg-yellow-500").child(to_element("Flex2")).build(),
                        div().class("w-8 h-2 bg-purple-500").child(to_element("Flex3")).build(),
                    ])
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🔧 Testing layout fix...");
    println!("You should see:");
    println!("- 'Normal flow' at top");
    println!("- 'Absolute @ 20,5' at position (20, 5)");
    println!("- 'Second normal' below first normal");
    println!("- Three flex items side by side");
    println!("Press Q or ESC to quit\n");
    
    // Since we can't run interactively, let's at least compile
    // Create backend
    use reactive_tui::backend::CrosstermBackend;
    let backend = CrosstermBackend::new()?;
    
    // Build and run the app
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(LayoutTest)
        .build()?;
    
    app.run()
}