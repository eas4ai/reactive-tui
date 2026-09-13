Example from src/hooks/refs.rs:99

```rust,no_run
use reactive_tui::hooks::use_ref;
use reactive_tui::reactive::Hooks;

fn remember_value(hooks: &Hooks) {
    let value = use_ref(hooks, String::from("initial"));
    let shared = value.clone();
    shared.set_current("updated".into());
    assert_eq!(value.current(), "updated");
}
```

Example from src/hooks/refs.rs:126

```rust,no_run
use reactive_tui::hooks::{use_local_ref, with_local_hooks};
use reactive_tui::reactive::Hooks;

with_local_hooks(|| {
    let hooks = Hooks::new();
    let value = use_local_ref(&hooks, String::new());
    value.set_current("draft".into());
    hooks.reset();
    assert_eq!(use_local_ref(&hooks, String::new()).current(), "draft");
});
```

Example from src/hooks/refs.rs:189

```rust,no_run
use reactive_tui::hooks::use_callback_ref;
use reactive_tui::reactive::Hooks;

fn selected_id(hooks: &Hooks) {
    let reference = use_callback_ref(hooks, |id: Option<String>| {
        println!("Selected: {id:?}");
    });
    reference.set(Some("entry-1".into()));
    assert_eq!(reference.current().as_deref(), Some("entry-1"));
    reference.set(None);
}
```

Example from src/hooks/refs.rs:253

```rust,no_run
use reactive_tui::hooks::{use_forwarded_ref, Ref};
use reactive_tui::reactive::Hooks;

fn update_parent(hooks: &Hooks, parent: Option<Ref<String>>) {
    let forwarded = use_forwarded_ref(hooks, parent);
    forwarded.set_if_exists("child value".into());
}
```

Example from src/hooks/refs.rs:321

```rust,no_run
use reactive_tui::hooks::use_multi_ref;
use reactive_tui::reactive::Hooks;

fn clear_selections(hooks: &Hooks) {
    let selected = use_multi_ref(hooks);
    let first = selected.add_ref(true);
    let second = selected.add_ref(true);
    selected.set_all(false);
    assert!(!first.current() && !second.current());
    selected.clear();
    assert_eq!(selected.count(), 0);
}
```
