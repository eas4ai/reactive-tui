//! Test CSS Absolute Positioning
//! 
//! Simple test to verify that absolute positioning with left-X top-Y works

use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;
use reactive_tui::builder::{div, to_element};

/// Simple positioning test
struct PositionTest;

impl reactive_tui::app::RootComponent for PositionTest {
    fn render(&self) -> Element {
        div()
            .class("relative w-screen h-screen bg-black")
            .children(vec![
                // Should appear at (0,0)
                div()
                    .class("absolute left-0 top-0 text-red-500")
                    .child(to_element("TL"))
                    .build(),
                
                // Should appear at (10,5)
                div()
                    .class("absolute left-10 top-5 text-green-500")
                    .child(to_element("10,5"))
                    .build(),
                
                // Should appear at (20,10)
                div()
                    .class("absolute left-20 top-10 text-blue-500")
                    .child(to_element("20,10"))
                    .build(),
                
                // Should appear at (40,15)
                div()
                    .class("absolute left-40 top-15 text-yellow-500")
                    .child(to_element("40,15"))
                    .build(),
            ])
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("🎯 Testing Absolute Positioning...");
    println!("You should see:");
    println!("  'TL' at top-left corner (0,0)");
    println!("  '10,5' at position (10,5)");
    println!("  '20,10' at position (20,10)");
    println!("  '40,15' at position (40,15)");
    println!("Press Q or ESC to quit");
    
    let backend = CrosstermBackend::new()?;
    let demo = PositionTest;
    
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(demo)
        .build()?;
    
    app.run()
}