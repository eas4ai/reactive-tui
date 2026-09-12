Example from src/lib.rs:22

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::backend::DebugBackend;

struct MyComponent;

impl reactive_tui::app::RootComponent for MyComponent {
    fn render(&self) -> Element {
        Element::text("Hello, World!")
    }
}

fn main() -> Result<()> {
    let app = App::builder()
        .backend(DebugBackend::new(80, 24))
        .root(MyComponent)
        .build()?;
    // Your app logic here
    Ok(())
}
```
