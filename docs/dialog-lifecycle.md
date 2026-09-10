# Dialog lifecycle

`DialogEngine` owns dialog sessions and presents their existing controls through
App. Its clones control the same sessions; separate engines have separate state.
Render one host per engine with `engine.render()`, or use the engine itself as an
App root. App owns the measured input routes, focus, timers and HTTP workers.
There is no second engine event tree with guessed coordinates.

Run the interactive example with:

```sh
cargo run --locked --example dialog_engine
```

## Opening and results

Prefer `try_show_confirmation`, `try_show_input`, `try_show_autocomplete`,
`try_show_progress`, `try_show_toast` and `try_show_wizard`. They return a dialog
ID or an error without opening a dialog. The existing `show_*` signatures remain;
they return `DialogId::INVALID` (zero) on rejection and expose `last_error()`.
Accepted IDs are unique within the engine and never wrap around.

`take_event()` consumes synchronous `Opened` and `Closed(id, result)` events.
Closing emits the actual result once, including a selected value or cancellation.
Closing an unknown or previously closed ID has no effect. The close callback
runs after removal from the active sessions and after the event is queued.
It may query, open or close another dialog: no engine lock is held during callbacks.
Use `downgrade()` for callbacks stored by the same engine, so they do not create
a strong ownership cycle.

For async consumers, call `enable_async()` before obtaining `events()` or
`completion(id)`. The completion receiver must be taken while the dialog is open;
`completion.result().await` resolves once without requiring Tokio. The engine
retains a single completion value, even if completion occurs before it is awaited.
Dropping a receiver does not cancel the dialog. The synchronous Closed event still
carries the result. Only one completion receiver can be taken per session.

The async event receiver and `take_event()` consume the same queue. They are not
broadcast subscriptions. Poll an event receiver in a separate task rather than
blocking the App thread that paints and handles the dialog.

The event queue holds at most 1,024 events. Opening reserves capacity for its
Closed event. If callers do not drain events, new opens return `EventQueueFull`;
existing dialogs can still close without losing their results. `max_dialogs`
bounds visible sessions, including controls finishing an exit transition.

## Updates, stacking and focus

`update(id, DialogUpdate::Progress(value))` changes the painted progress control;
finite values are clamped to 0..=1, and nonfinite values return an error.
`InputValue(value)` explicitly replaces input text, even when it equals the
original seed. Unrelated redraws preserve edits. `WizardData(data)` supplies the
current data to step validation and the completion callback. Wrong dialog types
and inactive IDs return errors.

`ZIndex(priority)` sets relative stacking priority. Higher values appear above
lower values; ties retain opening order. Toasts initially rank above ordinary
dialogs. App painting and pointer targeting use this same ordering. The engine
assigns consecutive backdrop/body layers starting at `base_z_index` and rejects
opens that exceed its representable layer range. `DialogBuffer` also honors its
numeric z-index and replaces an existing ID without duplicate entries.

The existing modal controls trap and restore focus. Closing nested dialogs
restores their previous control; closing the outer dialog restores the background
opener. The engine-wide `focus_trap` and `escape_to_close` settings apply in addition
to each control's options. Disabling Escape dismissal retains keyboard access to
Cancel and other buttons.

## Animation and cleanup

The engine applies `default_theme.animation` and `animation_duration` at the
shared modal boundary. Fade changes opacity; directional Slide moves in terminal
cells; Center and Scale expand from the center; Bounce changes scale. Custom
animation names require `register_animation(name, callback)`. The callback receives
progress from 0 to 1 and returns a `DialogAnimationFrame`. Unknown names and
out-of-range clock durations reject opening. Nonfinite or invalid custom frames
close with an Error result. Animation callbacks run outside engine locks.

Terminal cells cannot blur glyph outlines. `backdrop_blur` uses the existing
backdrop dimming approximation; disabling it removes that decorative dimming
while retaining modal input blocking and the configured focus behavior.

A user-completed control becomes inactive immediately. Its inert exit transition
may remain painted for `animation_duration`; its result is already available.
The engine removes that retained content at the owned deadline. Programmatic
`close_dialog` and `close_all` tear down immediately. Removing or dropping the App
host cancels its remaining sessions and animation deadlines, even if a controller
clone survives. Input validation and autocomplete work are cancelled on teardown;
a closed session cannot submit a late asynchronous result. Shutdown callbacks
cannot open replacement dialogs while cancellation is in progress.

## Acceptance

The dedicated command is `python3 -B scripts/check-api-dialog-lifecycle.py`.
Its declared inputs and requirements are in
[api-dialog-lifecycle](../.cairn/mechanisms/api-dialog-lifecycle.md).

| Behavior | Executable coverage |
| --- | --- |
| Results, events, async waits, reentry, limits and bounded backlog | `tests/api_dialog_lifecycle.rs` |
| Input and pointer resize, nested focus, all six families, progress and animations | `tests/api_dialog_lifecycle/app.rs` |
| Real remote validation completion and cancellation through the engine | `dialog_engine_http_*` in `tests/api_widget_behavior/dialog_http.rs` |
| Retained transition deadlines and identifier exhaustion | engine unit tests |
| Shared dialog and modal implementation | dialog and modal unit tests |
| Documented App construction | `examples/dialog_engine.rs` build |

The broader control inventory remains in [widget acceptance](widget-acceptance.md).
Native C and TypeScript dialog bindings remain separate API-017 work; this document
describes the Rust engine. Recorded Cairn receipts and the commitment review carry
the acceptance results and failure demonstrations.
