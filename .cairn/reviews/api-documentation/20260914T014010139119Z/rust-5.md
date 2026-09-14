Example from docs/unix-input.md:30

```rust,no_run
#[cfg(unix)]
fn input_stream(
    tty: &reactive_tui::platform::unix::UnixTty,
) -> reactive_tui::error::Result<reactive_tui::platform::InputReceiver<Vec<u8>>> {
    tty.spawn_input_thread()
}
```
