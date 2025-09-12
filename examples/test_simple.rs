use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;

struct SimpleTest;

impl reactive_tui::app::RootComponent for SimpleTest {
    fn render(&self) -> Element {
        div()
            .child(to_element("Hello World - Simple Test"))
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("Starting simple test...");
    
    // Set the bypass environment variable
    std::env::set_var("REACTIVE_TUI_FORCE_ENABLE", "1");
    
    // Create backend
    let backend = CrosstermBackend::new()?;
    
    // Build and run the app
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(SimpleTest)
        .debug(true)  // Enable debug mode
        .build()?;
    
    println!("App created successfully, starting run...");
    app.run()
}