# TypeScript binding migration

The previous package did not typecheck or load the shared library. The repaired
package uses native Element trees. API-017 adds ForeignComponent state/events,
NativeTextEditor, NativeLayoutStyle, NativeDialogEngine and NativeApp. See
[FFI manual](../../manual/ffi-and-typescript.md) for their ownership and current acceptance contract. The following removals are deliberate; they are not compiler exclusions.
All remaining files under src are still included in TypeScript checking.

The complete public before/after signatures, including class members, are recorded
in `binding-typescript-public-api.json` for all 17 original modules.

## Retired modules

### animation.ts

The old animation and group signatures had no compatible native implementation. Use the audited rtui_animation_* and rtui_animation_manager_* low-level API with its documented ownership; the old property enum is not compatible.

Exports: `AnimationType`, `AnimationProperty`, `AnimationOptions`, `Animation`, `AnimationGroup`, `fadeIn`, `fadeOut`, `slide`, `bounce`, `spring`.

### dialog.ts

The old classes remain retired. NativeDialogEngine now exposes the recovered native sessions, their App host and actual completion data.

Exports: `DialogType`, `DialogButton`, `DialogOptions`, `DialogResult`, `Dialog`, `ProgressDialog`.

### data-table.ts

The old DataTable wrapper remains retired. ForeignComponent now provides an explicit state/render/event controller; this does not restore the old nonexistent widget-specific JSON methods.

Exports: `FilterType`, `ColumnFilter`, `PaginationConfig`, `TableColumn`, `TableRow`, `DataTableConfig`, `DataTable`.

### input-widgets.ts

The old input-widget wrappers remain retired. Use ForeignComponent for explicit state/render/event behavior; it does not invent the former widget-specific methods.

Exports: `RadioOption`, `RadioButtonGroup`, `Slider`, `DatePicker`, `TimeValue`, `TimePicker`, `ColorValue`, `ColorPicker`, `FileInfo`, `FileSelector`.

### hooks.ts

The old React-style TypeScript hook wrappers remain retired. ForeignComponent now owns explicit prop/state values and routed callbacks; the low-level native signal/hook exports remain separate APIs.

Exports: `useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useReducer`, `Context`, `createContext`, `useContext`, `useLayoutEffect`, `useImperativeHandle`, `useDebugValue`, `useId`, `useDeferredValue`, `useTransition`, `useSyncExternalStore`.

## Changed supported paths

- `Terminal()` reads host dimensions; width/height constructor arguments are removed.
  `shutdown()` now disposes the handle; construct a new Terminal to reopen it.
  `flush()` was only a warning/no-op and is removed. Native output calls flush themselves.
- `Surface` stores one Unicode scalar per cell using the actual RTuiCell layout.
  Multi-scalar graphemes and embedded NUL are rejected. Resize recreates empty storage.
- `Renderer.getSurface()` returns a borrowed view. Resize and disposal invalidate views.
- `Component` represents a native static Element. JSON state, rendering and event
  methods remain removed from this static class. Use the separate ForeignComponent
  controller for state, props and callbacks, and Element tree builders for its output. Child insertion transfers ownership; getChild returns an owned clone.
- `ComponentBuilder` uses native builder handles. build consumes the builder.
  Arbitrary props, JSON serialization and dynamic input/grid helpers are removed.
  Supported props are class, key, content, label and text.
- Generic ListBuilder, TableBuilder and InputBuilder are removed. Static button,
  container and text builders remain. ForeignComponent supplies retained state and
  routed event callbacks for their Element output. Use ComponentBuilder for native class/key/text/child configuration.
- React-style ThemeContext/ThemeProvider/useTheme remain retired; the new foreign
  controller does not implement these old provider/hook wrappers. ThemeManager and pure theme/layout/event helpers remain.
- Conflicting legacy type exports are available through the `types` namespace;
  conflicting layout exports through `layout`. Direct module imports remain valid.
- Import no longer initializes the library or installs signal/exit handlers. Call
  initialize and cleanup explicitly, disposing all native handles first.
- Low-level `lib` exposes the compiler-audited native signatures. Its arguments follow
  native-api.json and native.h, not the removed ffi-napi/ref-napi declarations.
  Use RTUI_LIBRARY_PATH to select a library; native handles are bigint pointers.
  Never reuse a consumed pointer or pass arbitrary integers as handles.

The animation-demo.ts and widgets-demo.ts examples are retired with their unsupported wrappers.
Native builder text() converts the element itself to text; use child(text()) for a container.

API-017 callback lifetimes are explicit: dispose the App, then its foreign controllers.
Callbacks remain registered until native destruction succeeds; wrong-thread calls
and recursive render/dispatch/disposal are rejected. JSON setters may reenter.
NativeApp.run blocks the JavaScript event loop. The native call borrows the
retained App handle while it runs; another thread may call `rtui_app_quit`.
The TypeScript wrapper destroys the handle after run returns, including on
error. Native event callbacks may update state during this synchronous run.

---

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
foreign component state/events. See the [FFI manual](../../manual/ffi-and-typescript.md) for ownership,
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
- bufferGetCharPtr/FgPtr/BgPtr/AttributesPtr return allocated snapshots. Use the
  corresponding bufferRelease* function. The library retains the allocation metadata;
  the caller length remains in the ABI but does not control deallocation. Interior,
  misaligned and repeated release calls are ignored.
- Optimized buffers and text buffers are tracked owners. Their destroy calls ignore
  null and already-destroyed handles. Element child insertion rejects self-parenting.
- Optional root and effect-cleanup callbacks may be NULL. Required callbacks may not.
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
