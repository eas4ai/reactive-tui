Example from src/hooks/timer.rs:12

```rust,no_run
use reactive_tui::prelude::*;
use std::time::Duration;

#[component]
fn Counter(hooks: &Hooks) -> Element {
    let count = use_signal(hooks, 0);
    let tick_count = count.clone();
    use_interval(hooks, Duration::from_secs(1), move || {
        tick_count.update(|count| *count += 1);
    });
    Element::text(format!("Count: {}", count.get()))
}
```

Example from src/hooks/timer.rs:40

```rust,no_run
use reactive_tui::prelude::*;
use std::time::Duration;

#[component]
fn DelayedMessage(hooks: &Hooks) -> Element {
    let visible = use_signal(hooks, false);
    let delayed = visible.clone();
    use_timeout(hooks, Duration::from_secs(3), move || delayed.set(true));
    Element::text(if visible.get() { "Time's up!" } else { "Waiting..." })
}
```

Example from src/hooks/timer.rs:70

```rust,no_run
use reactive_tui::hooks::use_debounce;
use reactive_tui::reactive::Hooks;
use std::time::Duration;

fn queue_search(hooks: &Hooks, query: String) {
    let search = use_debounce(hooks, Duration::from_millis(300), |term: String| {
        println!("Search for {term}");
    });
    search.call(query);
}
```

Example from src/hooks/timer.rs:100

```rust,no_run
use reactive_tui::hooks::use_throttle;
use reactive_tui::reactive::Hooks;
use std::time::Duration;

fn report_scroll(hooks: &Hooks, position: i32) {
    let report = use_throttle(hooks, Duration::from_secs(1), |position: i32| {
        println!("Scroll position: {position}");
    });
    report.call(position);
}
```
