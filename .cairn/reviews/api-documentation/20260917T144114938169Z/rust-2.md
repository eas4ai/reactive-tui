Example from manual/getting-started.md:23

```rust,no_run
use reactive_tui::app::RootComponent;
use reactive_tui::backend::DebugBackend;
use reactive_tui::prelude::*;

struct Root;

impl RootComponent for Root {
    fn render(&self) -> Element {
        div().class("p-1").text("Hello").build()
    }
}

fn main() -> Result<()> {
    let _app = App::builder()
        .backend(DebugBackend::new(80, 24))
        .root(Root)
        .build()?;
    Ok(())
}
```
