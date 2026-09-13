Example from docs/local-hooks.md:10

```rust
use reactive_tui::hooks::{use_local_ref, with_local_hooks};
use reactive_tui::reactive::Hooks;
use std::{cell::Cell, rc::Rc};

with_local_hooks(|| {
    let hooks = Hooks::new();
    let first = use_local_ref(&hooks, Rc::new(Cell::new(1)));
    first.current().set(7);
    drop(first);
    hooks.reset();
    assert_eq!(use_local_ref(&hooks, Rc::new(Cell::new(99))).current().get(), 7);
});
```
