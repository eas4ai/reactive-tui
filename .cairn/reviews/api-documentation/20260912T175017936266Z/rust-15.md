Example from reactive-tui-macros/src/lib.rs:9

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn Counter(hooks: &Hooks) -> Element {
    let count = use_signal(hooks, 0);

    Element::text(&format!("Count: {}", count.get()))
}
```

Example from reactive-tui-macros/src/lib.rs:22

```rust,no_run
use reactive_tui::prelude::*;

#[component]
fn Greeting(hooks: &Hooks, name: String, age: Option<u32>) -> Element {
    let message = if let Some(age) = age {
        format!("Hello {}, you are {} years old!", name, age)
    } else {
        format!("Hello {}!", name)
    };

    Element::text(&message)
}
```

Example from reactive-tui-macros/src/lib.rs:243

```rust
use reactive_tui::prelude::*;

fn non_blank(text: &str) -> bool { !text.trim().is_empty() }

#[derive(Props, Clone, PartialEq)]
struct ButtonProps {
    #[prop(validate = non_blank)]
    text: String,
    #[prop(default)]
    disabled: bool,
    #[prop(default = "primary")]
    variant: String,
    #[prop(optional)]
    icon: Option<String>,
}
let props = ButtonProps::new().with_text("Save".into());
assert!(props.validate());
assert!(!props.with_text(" ".into()).validate());
```
