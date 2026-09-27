# Elements, builders, and the virtual DOM

Crate modules: `builder`, `vdom`

## Purpose

These APIs create UI trees and describe how a new tree differs from the last
tree.

## Main API

- `Element` constructors create text, layout, fragment, empty, named component,
  and typed component values.
- `ElementBuilder` adds classes, text, children, keys, click handlers, focus
  state, and gradient settings.
- Builder functions include `div`, `span`, `p`, `screen`, `container`, headings,
  rows, columns, grids, cards, buttons, and widget builders.
- `IntoElement` converts supported builder values into elements.
- `VNode` represents virtual text, elements, components, and fragments.
- `diff_vnodes`, `PatchList`, and `apply_patches` form the public virtual DOM
  update API.
- `from_vdom` and `mixed_container` combine virtual DOM nodes with elements.

## Basic use

```rust
use reactive_tui::builder::*;

let view = screen()
    .child(
        div()
            .class("flex flex-col gap-1 p-1")
            .child(h1().text("Status").build())
            .child(p().text("Ready").build())
            .build(),
    )
    .build();
```

## Behavior

Builders collect data until `build` returns an `Element`. Class strings are
stored on the element and resolved by the layout and style pipeline. Keys are
preserved for reconciliation. Click handlers and widget props become typed
component or event metadata.

The virtual DOM compares node type, key, props, text, and children. Its patches
can replace, update, insert, remove, or reorder nodes. The application rendering
path uses its own render tree and reconciler after component expansion.

## Limits

- Builder functions return values; they do not render by themselves.
- A class name has no effect unless the utility parser recognizes it.
- Stable keys are required when child order changes and state must follow the
  logical child.
- The element and virtual DOM APIs coexist. Use the conversion helpers at the
  boundary instead of assuming the types are interchangeable.

## Source map

- Builder exports: [`src/builder/mod.rs`](../src/builder/mod.rs)
- Core element builder: [`src/builder/core.rs`](../src/builder/core.rs)
- Mixed element and VDOM builder: [`src/builder/mixed.rs`](../src/builder/mixed.rs)
- Virtual DOM exports: [`src/vdom/mod.rs`](../src/vdom/mod.rs)
- Builder API tests: [`tests/api_widget_behavior/core_builders.rs`](../tests/api_widget_behavior/core_builders.rs)
- Virtual DOM behavior tests: [`tests/api_widget_behavior/vdom.rs`](../tests/api_widget_behavior/vdom.rs)

## Related chapters

- [Applications and components](app-and-components.md)
- [Layout, style, and themes](layout-style-and-themes.md)
- [Rendering and backends](rendering-and-backends.md)

[Back to the manual](README.md)
