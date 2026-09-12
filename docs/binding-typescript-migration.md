# TypeScript binding migration

The previous package did not typecheck or load the shared library. The repaired
package uses native Element trees. API-017 adds ForeignComponent state/events,
NativeTextEditor, NativeLayoutStyle, NativeDialogEngine and NativeApp. See
[native components](native-components.md) for their ownership and current acceptance contract. The following removals are deliberate; they are not compiler exclusions.
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
NativeApp.run blocks the JavaScript event loop and consumes the App, including
on error. Native event callbacks may update state during this synchronous run.
