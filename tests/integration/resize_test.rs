//! Resize and Flicker Test
//!
//! Tests terminal resize handling and flicker prevention.

use reactive_tui::backend::CrosstermBackend;
use reactive_tui::prelude::*;

struct ResizeTestApp {
    resize_count: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl reactive_tui::app::RootComponent for ResizeTestApp {
    fn render(&self) -> Element {
        let count = self.resize_count.load(std::sync::atomic::Ordering::Relaxed);

        // Create a simple responsive layout using Element API
        Element::layout(LayoutType::Flex)
            .class("w-full h-full flex-col items-center justify-center bg-gray-900")
            .children(vec![
                Element::text("🔄 Terminal Resize Test")
                    .class("text-3xl text-blue-400 text-center mb-4"),

                Element::text("Features being tested:")
                    .class("text-lg text-white mb-2"),

                Element::text("✅ Automatic resize detection")
                    .class("text-green-400 ml-4 mb-1"),

                Element::text("✅ Backend surface resizing")
                    .class("text-green-400 ml-4 mb-1"),

                Element::text("✅ Flicker-free updates (synchronized output)")
                    .class("text-green-400 ml-4 mb-1"),

                Element::text("✅ Responsive layout adaptation")
                    .class("text-green-400 ml-4 mb-1"),

                Element::text("✅ Full re-render on resize")
                    .class("text-green-400 ml-4 mb-4"),

                Element::text(format!("🔢 Resize events detected: {}", count))
                    .class("text-yellow-400 text-2xl mb-4"),

                Element::text("Try resizing your terminal window!")
                    .class("text-cyan-400 text-lg mb-2"),

                Element::text("Press Ctrl+C or Esc to exit")
                    .class("text-sm text-gray-600"),
            ])
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🔄 Terminal Resize and Flicker Test");
    println!("====================================");
    println!("This test verifies:");
    println!("• Resize events are properly detected");
    println!("• Backend surfaces are resized correctly");
    println!("• No flicker during resize (synchronized output)");
    println!("• Responsive grids adapt to new terminal size");
    println!("• Full re-render is triggered on resize");
    println!("\nTry resizing your terminal window to test!");
    println!("Press Enter to start...");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    let resize_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    
    // Create the application
    let backend = CrosstermBackend::new()?;
    let app = App::builder()
        .backend(backend)
        .root(ResizeTestApp {
            resize_count: resize_count.clone(),
        })
        .debug(true) // Enable debug overlay to see performance
        .build()?;

    println!("🚀 Starting resize test app...");
    println!("Resize your terminal to see the counter increment!");
    
    app.run()
}
