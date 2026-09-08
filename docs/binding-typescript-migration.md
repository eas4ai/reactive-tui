# TypeScript binding migration

The previous package did not typecheck or load the shared library. The repaired
package uses native Element trees. It does not add the missing dynamic widget
runtime. The following removals are deliberate; they are not compiler exclusions.
All remaining files under src are still included in TypeScript checking.

The complete public before/after signatures, including class members, are recorded
in `binding-typescript-public-api.json` for all 17 original modules.

## Retired modules

### animation.ts

The old animation and group signatures had no compatible native implementation. Use the audited rtui_animation_* and rtui_animation_manager_* low-level API with its documented ownership; the old property enum is not compatible.

Exports: `AnimationType`, `AnimationProperty`, `AnimationOptions`, `Animation`, `AnimationGroup`, `fadeIn`, `fadeOut`, `slide`, `bounce`, `spring`.

### dialog.ts

No dialog symbols are exported by the compiled library. Native dialogs are unsupported in this package.

Exports: `DialogType`, `DialogButton`, `DialogOptions`, `DialogResult`, `Dialog`, `ProgressDialog`.

### data-table.ts

Depends on nonexistent generic component JSON state/render/event functions. Interactive data-table binding is unsupported.

Exports: `FilterType`, `ColumnFilter`, `PaginationConfig`, `TableColumn`, `TableRow`, `DataTableConfig`, `DataTable`.

### input-widgets.ts

Depends on nonexistent generic component JSON state/event functions. Interactive input widget bindings are unsupported.

Exports: `RadioOption`, `RadioButtonGroup`, `Slider`, `DatePicker`, `TimeValue`, `TimePicker`, `ColorValue`, `ColorPicker`, `FileInfo`, `FileSelector`.

### hooks.ts

Depends on an unimplemented component rerender/lifecycle bridge. React-style TypeScript hooks are unsupported; the low-level native signal/hook exports are separate APIs.

Exports: `useState`, `useEffect`, `useMemo`, `useCallback`, `useRef`, `useReducer`, `Context`, `createContext`, `useContext`, `useLayoutEffect`, `useImperativeHandle`, `useDebugValue`, `useId`, `useDeferredValue`, `useTransition`, `useSyncExternalStore`.

## Changed supported paths

- `Terminal()` reads host dimensions; width/height constructor arguments are removed.
  `shutdown()` now disposes the handle; construct a new Terminal to reopen it.
  `flush()` was only a warning/no-op and is removed. Native output calls flush themselves.
- `Surface` stores one Unicode scalar per cell using the actual RTuiCell layout.
  Multi-scalar graphemes and embedded NUL are rejected. Resize recreates empty storage.
- `Renderer.getSurface()` returns a borrowed view. Resize and disposal invalidate views.
- `Component` represents a native static Element. JSON state, rendering and event
  methods have no native implementation and are removed. Use key/class setters and
  Element tree builders. Child insertion transfers ownership; getChild returns an owned clone.
- `ComponentBuilder` uses native builder handles. build consumes the builder.
  Arbitrary props, JSON serialization and dynamic input/grid helpers are removed.
  Supported props are class, key, content, label and text.
- Generic ListBuilder, TableBuilder and InputBuilder are removed. Static button,
  container and text builders remain; interactive handlers and arbitrary data props
  are unsupported. Use ComponentBuilder for native class/key/text/child configuration.
- React-style ThemeContext/ThemeProvider/useTheme are removed because the component
  lifecycle bridge is absent. ThemeManager and pure theme/layout/event helpers remain.
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
