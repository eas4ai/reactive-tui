use reactive_tui::prelude::*;

struct SimpleAbsolute;

impl reactive_tui::app::RootComponent for SimpleAbsolute {
    fn render(&self) -> Element {
        div()
            .class("w-full h-full bg-black relative")
            .children(vec![
                // Test absolute positioning
                div()
                    .class("absolute left-0 top-0 text-red-500")
                    .child(to_element("TL"))
                    .build(),
                div()
                    .class("absolute left-10 top-5 text-green-500")
                    .child(to_element("10,5"))
                    .build(),
                div()
                    .class("absolute left-20 top-10 text-blue-500")
                    .child(to_element("20,10"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    use reactive_tui::backend::CrosstermBackend;
    
    // This will show positions in terminal
    println!("Testing absolute positioning...");
    println!("Should see: TL at (0,0), '10,5' at (10,5), '20,10' at (20,10)");
    
    // Create backend
    let backend = CrosstermBackend::new()?;
    
    // Build and run the app
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(SimpleAbsolute)
        .build()?;
    
    app.run()
}