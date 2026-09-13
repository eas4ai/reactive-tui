Example from src/builder/widgets/dialog.rs:81

```rust,no_run
use reactive_tui::builder::modal;
use reactive_tui::component::Element;

let modal = modal()
    .title("Settings")
    .content(Element::text("Configure your preferences"))
    .visible(true)
    .size(600, 400)
    .build();
```
