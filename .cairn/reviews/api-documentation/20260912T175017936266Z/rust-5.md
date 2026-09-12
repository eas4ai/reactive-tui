Example from src/builder/core.rs:131

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::css;

let element = div()
    .styles(css! {
        display: DisplayType::Flex,
        background_color: Color::Blue,
        padding: Spacing::all(16.0),
    })
    .build();
```
