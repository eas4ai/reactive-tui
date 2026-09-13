# Supported API and evidence

This matrix covers every public root module and the macro facade. It links the
supported routes to behavior mechanisms and known limits. A module row does not
certify every function it contains. The linked widget, native and entry-point
inventories provide the per-family detail. Check each requirement's latest
receipt under `.cairn/evidence` for the committed input digest and result; old
passes become stale when an input changes. API-018 documentation compilation is
separate from the outstanding API-019 behavior review and API-020 closure.

| Public modules | Supported route and contract | Behavior checks | Limits and manual integration | Review |
| --- | --- | --- | --- | --- |
| `app`, `backend` | Retained App components, complete frames, input, resize and shutdown through SuprTUI; retained Crossterm/DirectTty adapters; DebugBackend memory frames and custom patch backends | [API-016](../.cairn/mechanisms/api-entry-points.md), [RND-001–006](../.cairn/mechanisms/suprtui-renderer.md) | [Entry points](application-entry-points.md) names manual patch/frame ownership; DebugBackend owns no host terminal | Covered by named checks |
| `embedded` | Unix PTY child interpreted by libghostty and composed in App; input, output, resize and cleanup | [EMB-001–006](../.cairn/mechanisms/embedded-terminal.md) | Requires embedded-terminal feature and pinned Zig; [embedded guide](embedded-terminal.md); no Windows embedded-module claim | Covered by named checks |
| `component` | Typed component expansion, keyed identity, prop/state updates, mount/unmount and event metadata | [API-002](../.cairn/mechanisms/api-component-expansion.md), [API-004](../.cairn/mechanisms/api-hook-lifecycle.md), [REG-001–003](../.cairn/evidence/REG-001), [CCH-001–002](../.cairn/evidence/CCH-001) | Unknown Elements retain their documented container behavior; factories must obey the Component contract | Covered by named checks |
| `builder`, `widgets` | Retained controls, text editor/input, tables/trees/charts, menus, dialogs, images and terminal widgets | [API-011](../.cairn/mechanisms/api-widget-behavior.md), [API-012](../.cairn/mechanisms/api-dialog-lifecycle.md), [API-014](../.cairn/mechanisms/api-images.md) | [Widget inventory](widget-acceptance.md) defines each family and public builder; host and accessibility limits below apply | Covered by named checks |
| `macros` | component macro enters owned hook frames; Props derive preserves defaults/builders and evaluates named field rules; CSS macros produce owned styles | [API-003](../.cairn/mechanisms/api-hook-state.md), [API-018](../.cairn/mechanisms/api-documentation.md) | Caller invokes validate; optional fields require Option<T>; explicit defaults take precedence; contract below | Covered by named checks |
| `reactive` | Typed signal owners, stable hook slots, effects/context, owned timers and App wakes | [API-001](../.cairn/mechanisms/api-signal-ownership.md), [API-003](../.cairn/mechanisms/api-hook-state.md), [API-004](../.cairn/mechanisms/api-hook-lifecycle.md), [WAK-001–005](../.cairn/evidence/WAK-001) | Follow owner/thread contracts; no blanket safety for invalid FFI handles | Covered by named checks |
| `hooks` | State/effects/timers and clipboard have focused runtime coverage; public gesture/ref APIs remain in the residual inventory | [API-004](../.cairn/mechanisms/api-hook-lifecycle.md), [API-008](../.cairn/mechanisms/api-clipboard.md), API-019 | Gesture event wiring remains under repair; shared and scoped local-reference checks pass locally, with complete acceptance pending; [residual inventory](residual-api-inventory.md) | API-019 review pending |
| `event` | App registers measured event geometry, callbacks, pointer input and stable nested focus | [API-005](../.cairn/mechanisms/api-event-routing.md), [API-006](../.cairn/mechanisms/api-focus.md) | Low-level EventRouter users register their own nodes/handlers; [event guide](app-events.md) | Covered by named checks |
| `ui` | Public UI helpers and Updater surface | API-019; [residual inventory](residual-api-inventory.md) | Updater dispatch must be demonstrated before this module receives a behavior acceptance claim | API-019 review pending |
| `layout` | CSS/Taffy geometry, state variants, text transformations, gradients, owned styles and terminal-cell transforms | [API-009](../.cairn/mechanisms/api-styling.md), [API-010](../.cairn/mechanisms/api-paint-properties.md), [API-018](../.cairn/mechanisms/api-documentation.md) | [Text styling](text-styling.md); App breakpoints use terminal columns; standalone responsive styles require explicit width; host fonts/glyph bitmaps are unchanged | Covered by named checks |
| `animation`, `screen` | Typed keyframe interpolation, current-value relative animation, retained screen input and visible transition midpoints | [API-013](../.cairn/mechanisms/api-animation-screens.md), [API-010](../.cairn/mechanisms/api-paint-properties.md), API-019 | [Animation guide](ANIMATION_INTEGRATION.md) explains caller-driven ScreenManager updates and approved cell approximations; legacy ID/custom-property/GPU metadata claims remain under review | API-019 review pending |
| `core` | Explicit terminal/surface/renderer ownership, grapheme frame primitives and image placement | [RND-001–006](../.cairn/mechanisms/suprtui-renderer.md), [API-014](../.cairn/mechanisms/api-images.md), [ABI-003](../.cairn/evidence/ABI-003) | App grapheme cells differ from legacy single-Unicode-scalar C surface cells; low-level frame owners must restore their session | Covered by named checks |
| `render`, `vdom` | Render trees, reconciliation, retained legacy patch route and menu node constructors | [API-016](../.cairn/mechanisms/api-entry-points.md), [API-011](../.cairn/mechanisms/api-widget-behavior.md), API-019 | Public RenderTree depth/breadth and nested legacy event coverage remain under review; a patch list is not a terminal frame | API-019 review pending |
| `editor` | Unicode scalar position model, insertion/deletion/movement and native editor access | [API-007](../.cairn/mechanisms/api-editor-unicode.md), [API-017](../.cairn/mechanisms/api-native-components.md), API-019 | Remaining undo/selection/grapheme claims have explicit residual contracts; do not interpret a byte offset as a scalar position | API-019 review pending |
| `markdown`, `syntax` | Public comrak-backed conversion, StyledLine rendering and Syntect highlighting; consistently visible rustdoc | [API-018](../.cairn/mechanisms/api-documentation.md), API-019 | End-to-end styled rendering and bounded large-input behavior require residual acceptance; parsing alone is not that proof | API-019 review pending |
| `platform`, `terminal`, `escape` | Retained platform adapters, child PTYs, parsing and terminal control; native clipboard/terminal records | [API-008](../.cairn/mechanisms/api-clipboard.md), [API-011](../.cairn/mechanisms/api-widget-behavior.md), [API-016](../.cairn/mechanisms/api-entry-points.md), API-019 | [Entry points](application-entry-points.md); [owned Unix input migration](unix-input.md); Windows ConPTY remains selected; parser, raw-mode and worker ownership subcases remain explicit residual work | API-019 review pending |
| `display`, `theme` | Performance metrics, adaptive FPS and theme/color helpers | [API-015](../.cairn/mechanisms/api-features.md), API-019 | [Performance guide](ADAPTIVE_PERFORMANCE.md); App-owned performance handles replace global App routing; focused checks pass locally, while native performance and automatic theme propagation remain under review | API-019 review pending |
| `accessibility` | Owned App accessibility tree, measured semantic controls and assistive activation | [API-011](../.cairn/mechanisms/api-widget-behavior.md) | Guaranteed reader route is Orca with GNOME Terminal; per-widget speech workflows and supported semantics are in the widget inventory | Covered by named checks |
| `ffi` | Compiler-audited C declarations, typed signals, stateful foreign components and editor/layout/dialog controllers | [ABI-001–004](../.cairn/evidence/ABI-001), [API-001](../.cairn/mechanisms/api-signal-ownership.md), [API-017](../.cairn/mechanisms/api-native-components.md) | Requires ffi feature; [native ownership](native-components.md), [C policy](FFI_ABI_POLICY.md), [TypeScript migration](binding-typescript-migration.md); C/TS acceptance is Linux, not every loader target | Covered by named checks |
| `error`, `prelude` | Shared error/result types and the public convenience import facade | [API-018](../.cairn/mechanisms/api-documentation.md), [API-016](../.cairn/mechanisms/api-entry-points.md), [API-017](../.cairn/mechanisms/api-native-components.md) | The facade does not certify every re-export; specific error/ownership contracts remain with the producing API | Covered by named checks |

## Props contract

`#[derive(Props)]` requires a named-field struct satisfying the Props bounds.
`#[prop(validate = rule)]` calls that predicate with a shared reference to its
field. `validate()` succeeds only if all declared rules succeed, evaluating in
field order and stopping at the first false result. No rules adds no constraints.
Construction, builders and App mounting do not invoke validation automatically.

`#[prop(optional)]` requires an explicit `Option<T>` and defaults to `None`.
A string default converts into the field type and takes precedence over optional;
`optional, default` keeps the normal empty Option default. Builders retain the
written field type. Bare validate, duplicate/unknown options and malformed
attributes are rejected. See the [approved decision](decisions/require-named-caller-invoked-props-validation-and-explicit-optional-field-types.md)
and the invalid/valid consumers in the API-018 mechanism.

## Feature and host boundaries

[Feature configurations](feature-configurations.md) defines default, no-default,
individual/combined non-SIMD and nightly SIMD/all-feature builds. Compilation
never replaces the separate native host records. Embedded sessions are Unix-only;
ConPTY is retained for the verified Windows legacy terminal route.

[Image acceptance](image-acceptance.md) names the tested routes: Kitty/Ghostty
protocols, Xterm Sixel, WezTerm inline, external Chafa/Viu, GNOME Terminal fallback
and native platform records. Exact Kitty acceptance uses a private pinned 0.45.0
patch; users of stock Kitty still need a corrected host. iTerm2 3.7 has an approved
color/transparency exception; geometry and cleanup remain required. No result
certifies arbitrary host versions or hardware. HTTP/HTTPS image URLs return an
unsupported-source error; local files and base64 data URLs remain supported.

API-020 must reconcile all pending rows with real passing behavior and the complete
audit. Documentation changes alone cannot close an advertised runtime gap.

Local-reference callers use the approved [local hook scope](local-hooks.md) contract.
