Example from docs/app-events.md:37

```rust
use reactive_tui::component::{Element, ElementType};
use std::sync::Arc;

let element = Element {
    element_type: ElementType::Text("Ready".into()),
    props: Arc::new(()),
    children: Vec::new(),
    key: None,
    class: None,
    focus: None,
    metadata: Default::default(),
};
```

Example from docs/app-events.md:55

```rust
use reactive_tui::builder::core::div;
use std::sync::{Arc, Mutex};

let count = Arc::new(Mutex::new(0));
let shared = count.clone();
let button = div().text("Increment").on_click(move || {
    *shared.lock().unwrap() += 1;
}).build();
```
