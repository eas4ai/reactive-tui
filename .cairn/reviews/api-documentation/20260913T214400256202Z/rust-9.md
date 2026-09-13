Example from src/builder/widgets/display.rs:216

```rust,no_run
use reactive_tui::builder::popover;
use reactive_tui::component::Element;

let popover = popover()
    .content(Element::text("Helpful information"))
    .trigger(Element::text("Help"))
    .build();
```
