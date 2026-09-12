Example from src/hooks/refs.rs:99

```rust,no_run
fn Timer(props: &Props, state: &mut State) -> Element {
    let timer_id = use_ref(&hooks, None::<TimerId>);
    
    use_effect(&hooks, move || {
        let id = start_timer();
        timer_id.set_current(Some(id));
        
        Some(Box::new(move || {
            if let Some(id) = timer_id.current() {
                cancel_timer(id);
            }
        }))
    });
    
    Element::text("Timer running...")
}
```

Example from src/hooks/refs.rs:130

```rust,no_run
fn TextInput(props: &Props, state: &mut State) -> Element {
    let input_ref = use_local_ref(&hooks, String::new());
    
    Element::input()
        .on_change(move |e| {
            input_ref.set_current(e.value.clone());
        })
        .on_submit(move |_| {
            let value = input_ref.current();
            submit_form(value);
        })
}
```

Example from src/hooks/refs.rs:189

```rust,no_run
fn FocusableInput(props: &Props, state: &mut State) -> Element {
    let input_ref = use_callback_ref(&hooks, |element: Option<DomElement>| {
        if let Some(el) = element {
            el.focus();
        }
    });
    
    Element::input()
        .ref_callback(input_ref)
        .auto_focus(true)
}
```

Example from src/hooks/refs.rs:246

```rust,no_run
fn FancyButton(props: &ButtonProps, state: &mut State) -> Element {
    let forwarded = use_forwarded_ref(&hooks, props.forward_ref.clone());
    
    Element::button()
        .ref_callback(move |el| {
            if let Some(element) = el {
                forwarded.set_if_exists(element);
            }
        })
        .child(text!(props.label))
}
```

Example from src/hooks/refs.rs:318

```rust,no_run
fn MultiSelect(props: &Props, state: &mut State) -> Element {
    let selected_refs = use_multi_ref(&hooks);
    
    Element::div()
        .children(props.items.iter().map(|item| {
            let item_ref = selected_refs.add_ref(false);
            
            Element::checkbox()
                .on_change(move |checked| {
                    item_ref.set_current(checked);
                })
        }))
        .child(
            Element::button()
                .on_click(move |_| {
                    selected_refs.set_all(false); // Clear all selections
                })
                .child(text!("Clear All"))
        )
}
```
