Example from src/hooks/clipboard.rs:180

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn CopyButton(hooks: &Hooks) -> Element {
    let (clipboard, copy, _paste) = use_clipboard(hooks);
    let status = clipboard.get();
    button().on_click(move || copy("Hello, clipboard!"))
        .child(Element::text(if status.error.is_some() { "Copy failed" } else { "Copy" }))
        .build()
}
```

Example from src/hooks/clipboard.rs:254

```rust,no_run
use reactive_tui::hooks::use_simple_clipboard;
use reactive_tui::reactive::Hooks;

fn copy_then_paste(hooks: &Hooks, selected: &str) -> Option<String> {
    let (copy, paste) = use_simple_clipboard(hooks);
    copy(selected);
    paste()
}
// Use use_clipboard when the caller needs the error signal as well.
```
