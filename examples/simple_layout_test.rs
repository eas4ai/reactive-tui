use reactive_tui::prelude::*;

struct SimpleLayoutTest;

impl reactive_tui::app::RootComponent for SimpleLayoutTest {
    fn render(&self) -> Element {
        // Use only CSS classes we KNOW are supported
        div()
            .class("w-40 h-20 flex flex-col gap-2 bg-blue-500")
            .children(vec![
                div()
                    .class("h-5 bg-red-500")
                    .child(to_element("Top Row - Red"))
                    .build(),
                div()
                    .class("h-5 bg-green-500")
                    .child(to_element("Middle Row - Green"))
                    .build(),
                div()
                    .class("h-5 bg-yellow-500")
                    .child(to_element("Bottom Row - Yellow"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("Testing with supported CSS classes...");
    println!("You should see:");
    println!("- Red row with text");
    println!("- Green row with text");
    println!("- Yellow row with text");
    println!("Press Q or ESC to quit\n");
    
    use reactive_tui::backend::CrosstermBackend;
    let backend = CrosstermBackend::new()?;
    
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(SimpleLayoutTest)
        .build()?;
    
    app.run()
}