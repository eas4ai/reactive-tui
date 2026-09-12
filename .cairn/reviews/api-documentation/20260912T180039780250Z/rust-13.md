Example from src/layout/css/css_in_rust.rs:52

```rust,no_run
use reactive_tui::css;

let button_styles = css! {
    display: Display::Flex,
    align_items: AlignItems::Center,
    justify_content: JustifyContent::Center,
    background_color: (0.0, 0.0, 1.0, 1.0), // Blue RGBA
    color: (1.0, 1.0, 1.0, 1.0), // White RGBA
    padding: 12.0,
    opacity: 1.0,
};
```

Example from src/layout/css/css_in_rust.rs:264

```rust,no_run
use reactive_tui::{flex_center, flex_column, absolute_fill};

let flex_center = flex_center!();
let flex_column = flex_column!();
let absolute_fill = absolute_fill!();
```

Example from src/layout/css/css_in_rust.rs:308

```rust,no_run
use reactive_tui::responsive_css;

let responsive_styles = responsive_css! {
    base: {
        display: Display::Block,
        padding: BoxSpacing::all(8.0),
    },
    md: {
        display: Display::Flex,
        padding: BoxSpacing::all(16.0),
    },
    lg: {
        padding: BoxSpacing::all(24.0),
    },
};
```
