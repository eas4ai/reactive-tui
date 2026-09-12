# C and native loader ABI migration

The compiled Rust exports are the compatibility baseline. Existing native exports
remain available. Recompile C and C++ callers against the repaired headers; old
header declarations were not reliable descriptions of the shared library.

The complete before/after inventory is in `binding-abi-migration.json`: every one
of the 198 original C functions, 58 TypeScript loader functions and 52 C type
aliases is listed. Original record fields and enum values remain recorded in
`binding-abi-baseline.json`. No historical evidence was rewritten.

## Common corrections

- `rtui_terminal_create` takes one output-handle pointer; it does not accept dimensions.
- `getTerminalCapabilities` returns void and writes `RTuiCapabilities`: ten byte-sized
  flags/values, including unicode_level. The former color-count structure was incompatible.
- `createRenderer` takes width and height. `destroyRenderer` takes renderer,
  clear_screen and cursor_y. `render` also takes force. Color pointers refer to
  four floats, not four separate arguments. See native.h for exact parameter order.
- `RTuiCell` contains a uint32_t Unicode scalar, two RGB triples, and seven boolean
  attributes. It does not contain a UTF-8 byte array. On the tested Linux ABI it is
  20 bytes; consumers must use sizeof rather than hard-code that size.
- The legacy div/span/button/p/h1/h2/h3 and builder mutation names now delegate
  to implemented Rust behavior. rtui_card consumes its child elements.
- `RTUI_PERFORMANCE_ADAPTIVE` is removed: value 3 is not a valid Rust variant.
  Use POWER_SAVER, BALANCED or HIGH_PERFORMANCE. Never pass undefined enum values.
- Existing header filenames include native.h and compat.h. Legacy-only style,
  dialog and helper type names remain for source migration; they do not imply
  that corresponding callable symbols exist.
- getRendererSurface previously claimed RTuiBuffer*. Its replacement
  rtui_renderer_get_surface returns a borrowed RTuiSurface*, a different native type.

## Reintroduced native controllers

API-017 adds editor snapshots, validated layout styles, dialog sessions and
foreign component state/events. See [native components](native-components.md) for ownership,
callbacks and acceptance. `rtui_dialog_engine_create` now creates a real engine.
`rtui_dialog_engine_destroy` returns a status. `rtui_dialog_engine_update` now
takes an engine, active ID and JSON changes; it is not the old delta-time no-op.
These declarations had no compiled symbols in the ABI baseline. Existing compiled
function signatures remain unchanged.

## Retired C functions

- `getRendererSurface`: Use rtui_renderer_get_surface with its native signature and ownership; it is not a drop-in alias.
- `rtui_primary_button`: This old symbol remains absent. Use the foreign component event callback and an Element focus target for native interaction.
- `rtui_dialog_engine_has_active_dialogs`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_create_confirmation`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_create_input`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_create_toast`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_create_progress`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_show`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_hide`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_close`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_is_visible`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_get_confirmation_result`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_get_input_text`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_set_progress`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `rtui_dialog_set_progress_message`: This old symbol remains absent; use the native dialog controller open/element/update/close/take_event operations described above.
- `getBufferDims`: Use getBufferWidth and getBufferHeight.
- `createRenderingContext`: No combined context export exists. Create terminal, renderer and buffers separately and release each owner.
- `destroyRenderingContext`: No combined context export exists. Create terminal, renderer and buffers separately and release each owner.
- `rtui_style_builder_create`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_destroy`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_apply_utility_classes`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_display`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_flex_direction`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_justify_content`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_align_items`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_width`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_height`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_padding`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_margin`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_background_color`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_color`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_style_builder_build`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_computed_style_destroy`: Use rtui_native_style_create/apply/destroy for validated inline CSS, or Element utility classes.
- `rtui_terminal_enter_raw_mode`: Use setupTerminal with its native signature and ownership; it is not a drop-in alias.
- `rtui_terminal_exit_raw_mode`: Use destroyTerminal with its native signature and ownership; it is not a drop-in alias.
- `rtui_terminal_clear`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_terminal_set_cursor`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_terminal_set_cursor_visible`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_terminal_write`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_terminal_flush`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_surface_resize`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_surface_clear_default`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_surface_set_text`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_surface_copy_rect`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `writeToBuffer`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_get_dimensions`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_begin_frame`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_end_frame`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_clear_with_color`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_draw_surface`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_draw_surface_rect`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_draw_text`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_fill_rect`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `rtui_renderer_draw_rect`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `logMessage`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.
- `textBufferAppendText`: No matching native export exists. Use the audited functions in native.h; no replacement with this exact behavior is provided.

## Ownership

All calls are synchronous unless the specific native API states otherwise.
Serialize operations on each handle. Null checks are not a general guarantee
that arbitrary, stale or forged non-null pointers are safe.

- Terminal, renderer, independently created surface/buffer, text buffer, builder,
  element, animation/manager, App/AppBuilder, hook and reactive handles have one
  owner. Pair each creation with its matching destroy/free function.
- A renderer surface is borrowed until renderer shutdown/destruction. Do not
  destroy it. TypeScript also invalidates its views after resize.
- Element builder build and child insertion consume their input owners once
  required pointers have passed validation, including a caught native panic.
  Do not reuse/free those pointers. rtui_card and builder_children require
  distinct live children and consume them after validating the complete array.
- Element getters returning strings allocate them; call rtui_string_free or
  rtui_free_string exactly once. Null means the optional value is absent.
- rtui_element_get_child returns an owned clone, not a borrowed child.
- bufferGetCharPtr/FgPtr/BgPtr/AttributesPtr return allocated snapshots; use the
  corresponding bufferRelease* function with the original element count.
- Callbacks retained by native APIs require an explicitly registered Koffi
  callback and caller-managed lifetime. Do not pass ephemeral JS callbacks to
  a retaining API or unregister while native code can still call it.
- Native rtui_* ownership is documented at its Rust definition. This audit
  establishes declarations and exercises the required consumer paths; it does
  not claim every legacy native function is behaviorally complete.

## Current native signal ownership

API-001 repaired the legacy signal ownership defect recorded by the original ABI
audit. Both ordinary RTuiSignal families now allocate a typed FFISignal owner;
either ordinary destructor releases the owner correctly. String, boolean and float
access interoperates. Legacy integers remain int64_t and improved integers remain
C int, so use the matching integer getter. Wrong live types return the documented
error/default; stale or forged pointers remain invalid. RTuiThreadSafeSignal uses
its separate functions and destructor. The original defect evidence is retained.

In the TypeScript loader, rtui_signal_get_string_owned returns a JavaScript string
(or null). Koffi calls rtui_string_free after conversion, so callers must not free
that JavaScript result. Other getters with char** output parameters still require
the explicit native string release described above. Buffer arguments are temporary
storage for synchronous calls; retaining native APIs require stable allocated
storage and an explicitly managed lifetime.

The existing surface API ignores blink/hidden attributes and returns them as false.
Its out-of-range getter returns a default NUL cell. The TypeScript Surface wrapper
rejects out-of-range coordinates before invoking native code.
