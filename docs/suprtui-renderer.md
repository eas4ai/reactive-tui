# SuprTUI renderer

The first renderer commitment adds `backend::SuprTuiBackend` for applications
inside ANSI truecolor terminals. Run the counter with:

```sh
cargo run --locked --example suprtui_counter
```

Space or `+` increments; `-` decrements. Escape and Ctrl+C quit. Resize the
terminal to recompute the layout. The example includes combining, CJK, and
emoji text. The required dependency is pinned to SuprTUI commit
`7793deb80c5bceecc5d8ed9fc6bc6d2530531774`; no sibling checkout is required.

## Application API

Pass `SuprTuiBackend::new()?` to `App::builder().backend(...)`. It enters raw
mode and the alternate screen. App calls the new, optional
`Backend::render_frame` method with the current root Element on every redraw.
The default returns `false`, preserving the patch path for other backends.

Implement `RootComponent::handle_event` to receive events left unhandled by
the event router. Returning `Handled` or `Consumed` requests a redraw. The
default returns `Ignored`. App retains Escape and Ctrl+C as quit keys.

The renderer paints the Element's classes, text, and children through the
existing CSS/Taffy bridge. It supports the tested flex/grid layout, padding,
colors, text attributes, overlapping layers, and ancestor clipping. Text
lines clip to their content box without splitting wide graphemes. Intrinsic
text sizes use Unicode cell widths. Control characters are omitted from text
output; newlines separate rows. This is not a migration of the entire widget
catalog, image rendering, or embedded terminal sessions.

Existing spacing conventions apply: for example, `p-0.5` is two cells and
`p-1` is four, while `w-3` is three cells. Use `font-bold` for bold text.
Descendants paint at least on their parent's layer so a positioned background
cannot cover its own text. This remains a terminal layer model, not a full
browser CSS stacking-context implementation.

## Output and lifetime

One worker owns SuprTUI's local grapheme pool and frame buffers. Commands use
a rendezvous channel: each presentation waits for its result, and frames do
not accumulate in a queue. The public backend remains `Send + Sync` without
unsafe thread-safety implementations.

`present()` writes and flushes changed frames, returning the original I/O
error when either operation fails. A retry redraws the complete screen;
previously written bytes cannot be rolled back. Output includes cancellation
and synchronized-update reset sequences so a retry can recover from a
truncated terminal sequence. Unchanged frames emit no bytes.

Nonzero resizes replace the frame buffers and force repainting. Zero-size
notifications retain the previous dimensions. Dimensions must fit `u16` and
contain at most 262,144 cells; construction or presentation returns an error
for invalid sizes before allocating buffers.

App calls `shutdown()` on normal exit. The backend also attempts restoration
and joins its worker on drop, including application errors and Rust unwinding.
Explicit shutdown returns cleanup errors and is idempotent. Raw mode already
owned by the caller is preserved. Use one active host-terminal session at a
time. Destructors cannot run after process abort or SIGKILL.

For captured output, use `SuprTuiBackend::with_writer(cols, rows, writer)`.
It owns a `Write + Send + 'static` writer without changing the host session.

## Verification

```sh
sh scripts/check-renderer.sh
```

Integration tests interpret ANSI with an independent VT parser and inject
write/flush failures. The Python pseudo-terminal probe drives App input and
SIGWINCH, inspects exact termios restoration, and checks alternate-screen and
cursor restoration for normal exit, input errors, and panic. The example's
`--probe-error` and `--probe-panic` switches enable these controlled failures
after Space input.

Direct visual checks in Kitty, GNOME Terminal, and Ghostty have not been run.
The pre-existing project-wide failures remain documented in
[recon](recon.md). libghostty-rs embedded sessions are the agreed next direction.
