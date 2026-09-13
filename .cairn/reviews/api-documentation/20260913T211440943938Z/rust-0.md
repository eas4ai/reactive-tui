Example from README.md:28

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::app::RootComponent;
use reactive_tui::backend::SuprTuiBackend;

struct Hello;
impl RootComponent for Hello {
    fn render(&self) -> Element {
        div().class("p-2 bg-blue-500 text-white")
            .child(Element::text("Hello, Reactive-TUI!"))
            .build()
    }
}

fn main() -> Result<()> {
    App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Hello)
        .build()?
        .run()
}
```

Example from README.md:61

```rust
use reactive_tui::component::Element;
use reactive_tui::widgets::Image;

fn photo() -> Element {
    Image::from_file("photo.jpg")
        .with_max_size(40, 12)
        .with_preserve_aspect(true)
        .into_element()
}
```
