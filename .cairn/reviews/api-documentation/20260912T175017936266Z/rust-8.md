Example from src/hooks/clipboard.rs:180

```rust,no_run
fn CopyButton(props: &Props, state: &State) -> Element {
    let (clipboard, copy, paste) = use_clipboard(&hooks);
    
    Element::button()
        .class("bg-blue-500 hover:bg-blue-700 text-white px-4 py-2")
        .on_click(move |_| copy("Hello, clipboard!"))
        .child(text!("Copy to clipboard"))
}
```

Example from src/hooks/clipboard.rs:252

```rust,no_run
fn TextEditor(props: &Props, state: &mut State) -> Element {
    let (copy, paste) = use_simple_clipboard(&hooks);
    
    Element::textarea()
        .on_key_down(move |e| {
            if e.modifiers.ctrl && e.code == KeyCode::Char('c') {
                copy(&state.selected_text);
            } else if e.modifiers.ctrl && e.code == KeyCode::Char('v') {
                if let Some(text) = paste() {
                    state.insert_text(text);
                }
            }
        })
}
```
