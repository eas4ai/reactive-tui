Example from README.md:81

```rust,no_run
use reactive_tui::app::RootComponent;
use reactive_tui::backend::SuprTuiBackend;
use reactive_tui::prelude::*;

struct Root;

impl RootComponent for Root {
    fn render(&self) -> Element {
        div()
            .class("flex-col w-full h-full p-1 bg-blue-900")
            .text("Hello, Reactive TUI!")
            .build()
    }
}

fn main() -> Result<()> {
    App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Root)
        .build()?
        .run()
}
```
