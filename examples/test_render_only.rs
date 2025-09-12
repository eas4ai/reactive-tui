use reactive_tui::prelude::*;
use reactive_tui::backend::CrosstermBackend;

struct SimpleTest;

impl reactive_tui::app::RootComponent for SimpleTest {
    fn render(&self) -> Element {
        println!("DEBUG: render() called");
        div()
            .class("w-full h-full")
            .child(
                div()
                    .class("text-white w-20 h-2")
                    .text("Hello World")
                    .build()
            )
            .build()
    }
}

fn main() -> reactive_tui::Result<()> {
    println!("Testing render only...");
    
    // Check if we're in a TTY
    if !atty::is(atty::Stream::Stdout) {
        println!("Not running in a TTY - skipping test");
        return Ok(());
    }
    
    println!("Creating backend...");
    let backend = CrosstermBackend::new()?;
    
    println!("Building app...");
    let app = reactive_tui::app::App::builder()
        .backend(backend)
        .root(SimpleTest)
        .debug(true)
        .build()?;
    
    println!("App built successfully!");
    println!("This is where the hang usually occurs - in app.run()");
    
    // Instead of calling app.run(), let's try to manually call the initial render
    // But we can't access the private render method, so let's just see if run() hangs
    
    println!("Calling app.run() - this might hang...");
    app.run()
}
