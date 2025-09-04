use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;

struct ColorTest;

impl reactive_tui::app::RootComponent for ColorTest {
    fn render(&self) -> Element {
        div()
            .class("w-full h-full bg-black p-2")
            .children(vec![
                // Test different text colors
                div()
                    .class("text-red-500")
                    .child(to_element("Red text"))
                    .build(),
                div()
                    .class("text-green-500")
                    .child(to_element("Green text"))
                    .build(),
                div()
                    .class("text-blue-500")
                    .child(to_element("Blue text"))
                    .build(),
                // Test background colors
                div()
                    .class("bg-yellow-500 text-black")
                    .child(to_element("Yellow background"))
                    .build(),
                // Test bold and italic
                div()
                    .class("text-white font-bold")
                    .child(to_element("Bold white text"))
                    .build(),
                div()
                    .class("text-cyan-500 italic")
                    .child(to_element("Italic cyan text"))
                    .build(),
                // Test combined
                div()
                    .class("text-purple-400 bg-gray-800 font-bold underline")
                    .child(to_element("Combined styles"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🎨 Testing CSS color rendering...");
    println!("You should see:");
    println!("- Red, green, blue text");
    println!("- Yellow background");
    println!("- Bold and italic text");
    println!("Press Q or ESC to quit");
    
    // Create backend
    let backend = CrosstermBackend::new()?;
    
    // Build and run the app
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(ColorTest)
        .build()?;
    
    app.run()
}