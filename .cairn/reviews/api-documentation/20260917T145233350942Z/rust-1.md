Example from manual/elements-builders-and-vdom.md:26

```rust
use reactive_tui::builder::*;

let view = screen()
    .child(
        div()
            .class("flex flex-col gap-1 p-1")
            .child(h1().text("Status").build())
            .child(p().text("Ready").build())
            .build(),
    )
    .build();
```
