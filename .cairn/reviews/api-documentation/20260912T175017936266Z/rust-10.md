Example from src/hooks/mouse.rs:19

```rust,no_run
fn Card(props: &Props, state: &State) -> Element {
    let hover = use_hover();
    
    Element::new("div")
        .class(if hover.is_hovered { "bg-gray-100" } else { "bg-white" })
        .class("p-4 rounded border")
        .on_mouse_enter(|_| {})
        .on_mouse_leave(|_| {})
        .child(text!("Hover over me!"))
}
```

Example from src/hooks/mouse.rs:100

```rust,no_run
fn DraggableItem(props: &Props, state: &mut State) -> Element {
    let drag = use_drag();
    
    Element::new("div")
        .class(if drag.is_dragging { "opacity-50" } else { "opacity-100" })
        .class("p-4 border rounded cursor-move")
        .style(format!("transform: translate({}px, {}px)",
                       drag.drag_delta.0, drag.drag_delta.1))
        .on_mouse_down(|e| {})
        .on_mouse_move(|e| {})
        .on_mouse_up(|e| {})
        .child(text!("Drag me around!"))
}
```

Example from src/hooks/mouse.rs:166

```rust,no_run
fn DragAndDropDemo(props: &Props, state: &mut State) -> Element {
    let dnd = use_drag_and_drop(DragAndDropOptions {
        drop_zones: vec!["drop-zone-1".to_string(), "drop-zone-2".to_string()],
        ..Default::default()
    });
    
    Element::new("div")
        .class("p-4")
        .children(vec![
            Element::new("div")
                .class("draggable p-4 bg-blue-500 text-white rounded")
                .class(if dnd.drag.is_dragging { "opacity-50" } else { "" })
                .child(text!("Drag me to a drop zone")),
            
            Element::new("div")
                .id("drop-zone-1")
                .class("drop-zone mt-4 p-8 border-2 border-dashed")
                .class(if dnd.is_over_valid_drop && dnd.drop_target == Some("drop-zone-1".to_string()) {
                    "border-green-500 bg-green-50"
                } else {
                    "border-gray-300"
                })
                .child(text!("Drop Zone 1")),
        ])
}
```

Example from src/hooks/mouse.rs:224

```rust,no_run
fn MouseTracker(props: &Props, state: &State) -> Element {
    let mouse = use_mouse_position();
    
    Element::new("div")
        .class("p-4 border")
        .child(text!("Mouse position: {:?}", mouse.position))
}
```

Example from src/hooks/mouse.rs:262

```rust,no_run
fn ClickableItem(props: &Props, state: &mut State) -> Element {
    let clicks = use_clicks();
    
    Element::new("div")
        .class("p-4 border cursor-pointer")
        .on_click(|_| {})
        .child(text!(
            "Clicks: {} {}",
            clicks.click_count,
            if clicks.is_double_click { "(double)" } else { "" }
        ))
}
```

Example from src/hooks/mouse.rs:304

```rust,no_run
fn LongPressButton(props: &Props, state: &mut State) -> Element {
    let long_press = use_long_press(Duration::from_millis(800));
    
    Element::new("button")
        .class("p-4 bg-blue-500 text-white rounded")
        .class(if long_press.is_pressing { "bg-blue-700" } else { "" })
        .on_mouse_down(|_| {})
        .on_mouse_up(|_| {})
        .child(text!(
            "{}",
            if long_press.is_long_press {
                "Long press detected!"
            } else {
                "Hold for long press"
            }
        ))
}
```

Example from src/hooks/mouse.rs:388

```rust,no_run
fn SwipeableCard(props: &Props, state: &mut State) -> Element {
    let gesture = use_gesture();
    
    let message = match &gesture.gesture_type {
        GestureType::Swipe(SwipeDirection::Left) => "Swiped left!",
        GestureType::Swipe(SwipeDirection::Right) => "Swiped right!",
        _ => "Swipe me",
    };
    
    Element::new("div")
        .class("p-8 bg-gray-100 rounded")
        .child(text!("{}", message))
}
```

Example from src/hooks/mouse.rs:454

```rust,no_run
fn ScrollableContent(props: &Props, state: &mut State) -> Element {
    let wheel = use_wheel();
    
    Element::new("div")
        .class("overflow-hidden h-64")
        .style(format!("transform: translateY({}px)", -wheel.delta_y))
        .child(text!("Scroll content here"))
}
```
