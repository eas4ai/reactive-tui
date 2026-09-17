Example from src/builder/core.rs:131

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::css;

let element = div()
    .styles(css! {
        display: Display::Flex,
        background_color: (0.0, 0.0, 1.0, 1.0),
        padding: 16.0,
    })
    .build();
```
