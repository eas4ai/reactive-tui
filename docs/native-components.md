# Native controller contract

This is the API-017 acceptance contract. Implementation is pending until its
compiled consumers pass; it is not a readiness claim.

The new C declarations are in `reactive_tui/native.h`. TypeScript exposes
`NativeTextEditor`, `NativeLayoutStyle`, `NativeDialogEngine`, `ForeignComponent`
and `NativeApp` from its package. Existing static Element/builders remain valid.

## Editor and layout

An editor owns the recovered Rust TextEditor. Content is UTF-8. Movement and
selection use its complete-grapheme rules; a newline is crossed atomically.
Insertion replaces selection. Backward/forward deletion removes selection or
one grapheme. Viewport dimensions are terminal cells, each at most 65535 and
their product at most the renderer's cell limit. Its Element is an owned styled
snapshot, including cursor/selection and complete graphemes; a later edit does
not mutate an earlier snapshot. The caller routes events into editing methods
and requests a new snapshot when state changes. No editor configuration, undo
or mode is invented around an absent native method.

`NativeLayoutStyle` parses the existing native inline CSS grammar. Invalid
properties, nonfinite values and invalid dimensions return an error. Applying
a style copies its snapshot into an Element; the style handle may then be
disposed. The normal layout engine computes and paints flex/grid, spacing,
size and color. The controller does not run a separate layout implementation.

## Dialogs

An engine owns the recovered Rust dialog controller. `element()` supplies its
normal App host. `open(options)` accepts confirmation, input, toast, progress,
autocomplete and wizard options; the typed schema specifies supported fields
and rejects unknown fields. Limits and failed operations return errors.
`update(id, changes)` changes an active dialog without recreating its controls.
`close(id, result)` retains the real result and data. `takeEvent()` drains native
Opened/Closed events; a missing event is distinct from an error. Results and
strings are owned copies. Closing twice returns NotFound. Disposing cancels
remaining dialogs and disables their retained hosts. Independent engines have
independent IDs, queues and state. App shutdown cancels mounted dialogs.

## Foreign components

Each controller owns one logical prop/state pair as valid JSON (at most 1 MiB
per value). Separate controllers are isolated; Element clones reference the
same controller. Its state survives redraw, reorder and temporary unmount.
Native code tracks reads during component rendering, so setters wake its App.

Creation retains render/event/dispose callbacks and opaque caller userdata.
The render callback receives borrowed prop/state JSON and returns an owned
Element through an output pointer. The native caller consumes any returned
Element on success or error. Success without an Element is InvalidState.
The optional event callback receives borrowed event/prop/state JSON and writes
whether it handled the event. Return zero for success or a native error code.
Event JSON preserves key name/character, press/release/repeat, modifiers,
mouse coordinates/button/kind, paste text, resize, focus and custom input.
No callback may unwind across C; TypeScript catches exceptions and returns an
error to native code, then rethrows the original exception at its outer call.

Callbacks and controller operations are confined to the creating thread.
Wrong-thread use returns InvalidState before calling foreign code. Reads and
prop/state setters may reenter during a callback; recursive render/dispatch
and disposal while a callback is running return InvalidState. Native locks
are released before every foreign callback. Render/event callback errors are
observable and stop an App through its normal error/restoration path.

`destroy` ends callback access and invokes the dispose callback exactly once,
on the creating thread. Userdata and registered function pointers must remain
valid until this succeeds. Old Element snapshots may still be freed afterward,
but cannot invoke disposed callbacks. Native handles are invalid after disposal;
TypeScript prevents reuse and supports repeated `dispose()`. Raw C handles,
including old existing APIs, must not be fabricated, freed twice or accessed
concurrently with disposal. Strings returned via `char **` are individually
owned and require `rtui_string_free`.

## Application ownership

The TypeScript App adapter uses the existing blocking native App and SuprTUI
selector. Its registered root callback remains alive until run returns or
the unrun App is disposed. Run consumes the App even on error. The synchronous
run owns terminal input; JavaScript timer callbacks do not execute during it.
Updates from native event callbacks are supported. The consumer disposes its
controllers after App shutdown, then unregisters callbacks and releases userdata.

Acceptance: `scripts/check-api-native-components.py` first checks C/Rust/Koffi
signatures and layouts, then compiles real C and TypeScript consumers. PTYs
check painted frames, layout positions, editor edits, state/prop updates,
dialog input/results, normal/error restoration, and cleanup. Direct consumers
check invalid input, independent ownership, reentry and callback failure.
