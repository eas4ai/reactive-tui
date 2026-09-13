# Owned Unix input streams

`UnixTty::spawn_input_thread` returns `InputReceiver<Vec<u8>>`.
`DirectTty::start_async_events` returns `InputReceiver<TerminalEvent>`.
Both receivers live in `reactive_tui::platform` and own their input worker's cleanup.
Keep the terminal owner alive while receiving. Dropping the receiver or final
terminal owner stops and joins the worker, including while input is idle or its
queue is full. Dropping an owned iterator has the same cleanup behavior.

The queue holds at most 64 items. Raw chunks contain at most 4096 bytes each;
parsed events are produced from one input chunk at a time. A full queue stops
further reading until the consumer makes room. Each stream has its own queue and
worker. Multiple streams read from the same terminal; they compete for input and
do not broadcast copies of it. Removing one receiver leaves other streams alive.

`recv`, `try_recv`, `recv_timeout`, `iter`, `try_iter`, and owned/borrowed iteration
retain the usual receive behavior and standard error types. After shutdown,
already queued items remain readable, followed by a disconnection error. As in
the earlier API, EOF or an input error disconnects the stream; the receiver does
not report a separate IO error value.

## Migration

The approved API-019 repair replaces the concrete
`std::sync::mpsc::Receiver<T>` return types. Inferred calls such as
`let input = tty.spawn_input_thread()?; input.recv()` retain their form.
Change explicit stored types, function parameters and return annotations to
`InputReceiver<T>`. For example:

```rust,no_run
#[cfg(unix)]
fn input_stream(
    tty: &reactive_tui::platform::unix::UnixTty,
) -> reactive_tui::error::Result<reactive_tui::platform::InputReceiver<Vec<u8>>> {
    tty.spawn_input_thread()
}
```

Explicit standard iterator types must also migrate: `InputIter<'_, T>` represents
borrowed iteration and `InputIntoIter<T>` represents owned iteration. Both are
exported from `reactive_tui::platform`. The underlying standard receiver cannot be
extracted separately from its cleanup owner.

The implementation has Linux private-PTY lifecycle evidence. Native macOS
verification and the separate legacy parser/signal audit remain API-019 work;
these methods are Unix-only and make no Windows async-input claim. See the
[residual inventory](residual-api-inventory.md) for remaining coverage.
