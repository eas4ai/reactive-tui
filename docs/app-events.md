# App callbacks and painted bounds

`ElementBuilder::on_click` retains each supplied callback. App invokes callbacks
on the closest interactive ancestor of the target, in registration order. An
activation stops propagation, so a nested button does not also activate its
parent button. The callback runs synchronously on App's event-loop thread,
outside registry and scheduler locks. It may update signals or request redraws.

Keyboard activation accepts an unmodified Enter or Space press on the focused
element. Release and repeat events do not activate it again. Mouse activation
accepts a left-button Down, or an explicitly supplied synthetic Click. Crossterm
produces Down/Up pairs; Up does not activate. An input adapter must not emit both
Down and a synthetic Click for one activation. Left-button activation also focuses
the closest focusable ancestor. Clicks outside the painted target do not activate
the focused element.
Pixel coordinates are left to the root input handler; App does not guess a
pixel-to-cell conversion without terminal cell dimensions.

SuprTUI returns visible cell bounds and paint order from the same layout pass
that painted the frame. App installs those bounds and the frame's callbacks after
successful presentation. Resizing publishes a new frame before accepting another
input event. Later siblings win ties at the same layer; clipped areas do not
receive mouse input. Custom wrappers around SuprTUI should forward Backend's
`painted_nodes()` method along with rendering and resize. Legacy backend integration
is tracked separately by API-016.

Redraw replaces callback registrations. Removing an element releases App's and
the backend's references to its callbacks. Any Element clones retained by the
application still own their callbacks. A caller must avoid strong reference cycles
between its callback and its own state.

## Approved Rust migration

Element now has an owned `metadata` field. Calls to constructors and builders
keep their syntax. Existing struct literals add `metadata: Default::default()`:

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

`on_click` and `primary_button` callbacks now require `Send + Sync`. Use shared
thread-safe state when capturing mutable values:

```rust
use reactive_tui::builder::core::div;
use std::sync::{Arc, Mutex};

let count = Arc::new(Mutex::new(0));
let shared = count.clone();
let button = div().text("Increment").on_click(move || {
    *shared.lock().unwrap() += 1;
}).build();
```

These two source changes were approved in `.cairn/escalations/api-005.md`.
The C ABI is unchanged. Behavior evidence is produced by
`scripts/check-api-event-routing.py`; this document does not claim the remaining
widget and focus-trap remediation is complete.
