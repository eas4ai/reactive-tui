use reactive_tui::prelude::*;

struct DebugApp;

impl reactive_tui::app::RootComponent for DebugApp {
    fn render(&self) -> Element {
        eprintln!("DEBUG: render() called");
        let element = div()
            .class("w-full h-full")
            .child(to_element("Hello World"))
            .build();
        eprintln!("DEBUG: Element created with class: {:?}", element.class);
        element
    }
}

fn main() -> reactive_tui::Result<()> {
    eprintln!("DEBUG: Starting app");
    
    // Try with direct paint_tree
    use reactive_tui::core::surface::Surface;
    use reactive_tui::layout::paint_tree::{layout_and_paint_with, NodeSpec, PaintOptions};
    use std::borrow::Cow;
    
    let mut surface = Surface::new(40, 10);
    let spec = NodeSpec {
        class: Cow::Borrowed("w-20 h-5"),
        text: Some(Cow::Borrowed("TEST DIRECT")),
        children: vec![],
    };
    
    eprintln!("DEBUG: Testing direct paint_tree");
    layout_and_paint_with(&spec, &mut surface, 40, &PaintOptions::default())?;
    
    // Print surface to see if it worked
    eprintln!("DEBUG: Surface contents:");
    for y in 0..5 {
        eprint!("  |");
        for x in 0..25 {
            let cell = surface.get_at(reactive_tui::core::geometry::Point::new(x, y));
            eprint!("{}", cell.ch);
        }
        eprintln!("|");
    }
    
    // Now try with the app
    eprintln!("\nDEBUG: Creating backend");
    use reactive_tui::backend::CrosstermBackend;
    let backend = CrosstermBackend::new()?;
    
    eprintln!("DEBUG: Building app");
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(DebugApp)
        .build()?;
    
    eprintln!("DEBUG: Running app (will fail in non-tty environment)");
    // This will fail in test environment, but we can see debug output
    let _ = app.run();
    
    Ok(())
}