Example from src/hooks/mouse.rs:23

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn Card(hooks: &Hooks) -> Element {
    let state = use_hover(hooks).get();
    div().class(if state.is_hovered { "bg-gray-100" } else { "bg-white" })
        .child(Element::text("Hover over me")).build()
}
```

Example from src/hooks/mouse.rs:106

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn DraggableItem(hooks: &Hooks) -> Element {
    let drag = use_drag(hooks).get();
    div().class(if drag.is_dragging { "opacity-50" } else { "opacity-100" })
        .child(Element::text(format!("Drag delta: {:?}", drag.drag_delta))).build()
}
```

Example from src/hooks/mouse.rs:168

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn DragAndDropDemo(hooks: &Hooks) -> Element {
    let state = use_drag_and_drop(hooks, DragAndDropOptions {
        drop_zones: vec!["drop-zone-1".into()],
        ..Default::default()
    });
    Element::text(format!("Drop target: {:?}; allowed: {}", state.drop_target, state.can_drop))
}
```

Example from src/hooks/mouse.rs:205

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn MouseTracker(hooks: &Hooks) -> Element {
    let mouse = use_mouse_position(hooks).get();
    Element::text(format!("Mouse position: {:?}", mouse.position))
}
```

Example from src/hooks/mouse.rs:243

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn ClickableItem(hooks: &Hooks) -> Element {
    let clicks = use_clicks(hooks).get();
    Element::text(format!("Clicks: {}; double: {}", clicks.click_count, clicks.is_double_click))
}
```

Example from src/hooks/mouse.rs:279

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn LongPressButton(hooks: &Hooks) -> Element {
    let press = use_long_press(hooks, std::time::Duration::from_millis(800)).get();
    button().child(Element::text(if press.is_long_press { "Long press" } else { "Hold" })).build()
}
```

Example from src/hooks/mouse.rs:352

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn SwipeableCard(hooks: &Hooks) -> Element {
    let gesture = use_gesture(hooks).get();
    let message = match gesture.gesture_type {
        GestureType::Swipe(SwipeDirection::Left) => "Swiped left",
        GestureType::Swipe(SwipeDirection::Right) => "Swiped right",
        _ => "Swipe me",
    };
    Element::text(message)
}
```

Example from src/hooks/mouse.rs:416

```rust,no_run
use reactive_tui::prelude::*;
use reactive_tui::hooks::mouse::*;

#[component]
fn ScrollableContent(hooks: &Hooks) -> Element {
    let wheel = use_wheel(hooks).get();
    Element::text(format!("Wheel delta: {}, {}", wheel.delta_x, wheel.delta_y))
}
```
