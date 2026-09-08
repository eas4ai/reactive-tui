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

## Retired C functions

- `getRendererSurface`: Use rtui_renderer_get_surface with its native signature and ownership; it is not a drop-in alias.
- `rtui_primary_button`: Rust on_click does not install a callable handler. Use rtui_button for a static styled element; native click callbacks remain unsupported.
- `rtui_dialog_engine_create`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_engine_destroy`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_engine_update`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_engine_has_active_dialogs`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_create_confirmation`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_create_input`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_create_toast`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_create_progress`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_show`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_hide`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_close`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_is_visible`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_get_confirmation_result`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_get_input_text`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_set_progress`: No dialog API is compiled into the library; unsupported.
- `rtui_dialog_set_progress_message`: No dialog API is compiled into the library; unsupported.
- `getBufferDims`: Use getBufferWidth and getBufferHeight.
- `createRenderingContext`: No combined context export exists. Create terminal, renderer and buffers separately and release each owner.
- `destroyRenderingContext`: No combined context export exists. Create terminal, renderer and buffers separately and release each owner.
- `rtui_style_builder_create`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_destroy`: No native style-builder API is compiled; use Element class strings.
- `rtui_apply_utility_classes`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_display`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_flex_direction`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_justify_content`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_align_items`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_width`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_height`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_padding`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_margin`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_background_color`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_color`: No native style-builder API is compiled; use Element class strings.
- `rtui_style_builder_build`: No native style-builder API is compiled; use Element class strings.
- `rtui_computed_style_destroy`: No native style-builder API is compiled; use Element class strings.
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

## Existing native signal limitation

The legacy rtui_signal_string_create/int_create/float_create/bool_create functions allocate
separate Signal<T> types. rtui_signal_destroy currently assumes Signal<String>;
do not use it for int/float/bool handles. Do not mix those handles with the newer
rtui_signal_new_* family. Use the newer family with rtui_signal_destroy_new,
which uses a tagged FFISignal allocation. The legacy ownership defect is recorded
in the backlog; no mismatched or stale pointer was invoked during this audit.

In the TypeScript loader, rtui_signal_get_string_owned returns a JavaScript string
(or null). Koffi calls rtui_string_free after conversion, so callers must not free
that JavaScript result. Other getters with char** output parameters still require
the explicit native string release described above. Buffer arguments are temporary
storage for synchronous calls; retaining native APIs require stable allocated
storage and an explicitly managed lifetime.

The existing surface API ignores blink/hidden attributes and returns them as false.
Its out-of-range getter returns a default NUL cell. The TypeScript Surface wrapper
rejects out-of-range coordinates before invoking native code.
