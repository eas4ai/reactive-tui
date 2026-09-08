# Rust API completion audit

Status: Observed, 2026-09-08
Revision: 2a052f996cca1ca0283d1df80b496263b10d6ddf
Scope: public Rust modules, application integration, and Rust implementations behind the C boundary.
This is an assessment, not a new implementation commitment. No production code or agreed requirement changed.

The renderer, embedded terminal and wakeup foundations work on their verified paths.
The largest remaining problem is the missing connection between component instances,
hooks, layout, input routing and painting. Many widgets have substantive implementations,
but exposing their types does not make them usable through App automatically.

“Missing” below means code is absent. “Partial” means code accepts an operation but
omits part of its promised behavior. “Integration gap” means useful code exists but
the application path does not connect it. “Unverified” does not mean broken.

## Priority findings and completion tests

Priority 0 is a memory-safety repair. Priority 1 restores basic framework behavior.
Priority 2 completes feature families after their dependencies work. These priorities
are recommendations, not authorization to retire or implement APIs.

| ID / priority | What exists and what is wrong | Evidence | Completion test |
| --- | --- | --- | --- |
| RAPI-01 / 0 | Legacy native integer/float/bool signal constructors allocate different Signal<T> types, but their common destructor casts all of them to Signal<String>. The newer tagged signal family exists and has its own destructor. | [constructors/destructor](../src/ffi/reactive.rs:330); known binding-audit backlog. Source-confirmed; unsafe call deliberately not executed. | Give every supported creation/get/set/destroy sequence a matching allocation type; make cross-family use unambiguous; test all types with an appropriate memory-safety checker. Preserve or explicitly migrate the old ABI. |
| RAPI-02 / 1 | Named Rust components are instantiated and registered, but their render output is not expanded into the App frame. SuprTUI receives the unresolved Element first. Render-tree construction then creates instances, including a fresh clone for registration, without rendering their output. | [App render order](../src/app.rs:354), [bridge](../src/component/bridge.rs:6), [tree conversion](../src/render/tree.rs:379). Reproduced: registered component render calls = 0; plain text control paints. | A registered component and nested child paint their output; a keyed child retains state across redraw/reorder; removal unmounts once; prop updates reach the same instance; unknown names produce a defined result. |
| RAPI-03 / 1 | Hooks have reusable indexed storage, but generated component render methods never reset the hook index. Each render allocates another state slot. use_memo evaluates the function again but returns the old stored signal. Effect/context lifecycle is also incomplete: cleanup clears IDs; no-runtime effects immediately run cleanup; context storage has no parent lookup. | [macro generation](../reactive-tui-macros/src/lib.rs:108), [hook reset/storage](../src/reactive/hooks.rs:33), [effect fallback](../src/reactive/hooks.rs:200), [memo](../src/reactive/hooks.rs:306). State-slot and stale-memo cases reproduced. | Repeated component renders preserve state without growing slots; memo values update under the chosen dependency semantics; effects run/clean up at documented lifecycle points; child context inherits and overrides correctly; timers use the App-owned runtime. |
| RAPI-04 / 1 | ElementBuilder::on_click discards the callback. EventRouter has real nodes/handlers/hit testing, but App does not create a matching event tree from rendered elements. App also uses a separate declarative FocusManager: IDs change each render; trap collection creates different child IDs from the main walk; next/previous outside a trap return immediately; stale-trap restoration is checked after stale traps are removed. | [callback](../src/builder/core.rs:242), [event router](../src/event/router.rs:91), [App input](../src/app.rs:178), [declarative focus](../src/app/focus_manager.rs:58). Callback disposal, unstable IDs and missing trapped autofocus reproduced. | A keyboard/mouse click invokes the supplied callback once; keyed focus survives redraw; Tab/Shift-Tab work with and without traps; opening/closing nested dialogs traps/restores focus; hit bounds match painted layout. |
| RAPI-05 / 1 | TextEditor::insert_text advances its cursor by UTF-8 byte length, but GapBuffer stores Vec<char> and deletion uses character positions. | [insertion](../src/editor/text_editor.rs:97), [buffer representation](../src/editor/gap_buffer.rs:29). Reproduced: insert_text("界") then backspace leaves "界"; ASCII control deletes correctly. | Use one documented position model across insertion, deletion, movement and selection; test CJK, emoji, combining sequences, multiline edits and mixed insert_char/insert_text calls. |
| RAPI-06 / 1 | Clipboard calls run real external tools but wait without deadlines and often ignore nonzero exit status. Missing-backend copy also returns success. A frozen clipboard process can freeze its caller. | [clipboard backend](../src/hooks/clipboard.rs:71); prior 300-second wl-copy stalls retained in the backlog. Source rechecked; no live clipboard mutation performed. | Stalled tools time out and are reaped; nonzero exits return actionable errors; unavailable clipboard is distinguishable from successful copying; verify each claimed desktop backend separately. |
| RAPI-07 / 2 | CSS/Taffy layout and cell styling work, but state variants are applied unconditionally. Several typography tokens return an unchanged builder. Gradient/animation metadata has separate implementation paths and is not carried by the Element-to-NodeSpec bridge. | [focus variants](../src/layout/css/focus.rs:69), [hover variants](../src/layout/css/variants.rs:138), [typography](../src/layout/css/typography.rs:23), [gradient props](../src/builder/core.rs:330), [bridge](../src/component/bridge.rs:6). Reproduced: focus:bg-red-500 applies without focus; uppercase leaves lower-case text. | Focus/hover/disabled styles depend on actual node state; promised text transforms/overflow change output; gradients/animations reach the supported painter. Explicit terminal approximations such as font-size labels should be documented rather than mistaken for missing pixel-font rendering. |
| RAPI-08 / 2 | Real widget state/render/event code exists, including TextInput, checkbox/select, table/tree and menus. Some builders still emit descriptive placeholder text. Several controls use default bounds or omit advertised panels and mouse behavior. The shared component/event gap prevents automatic integration even where a widget's own logic works. | [TextInput event implementation](../src/widgets/input/text_input.rs:1136), [context-menu builder](../src/builder/widgets/menu.rs:782), [data table](../src/widgets/display/data_table.rs:243), [table layout assumptions](../src/widgets/display/table.rs:595). Source-confirmed; not every widget interaction was exercised. | First prove button + TextInput + checkbox + select end to end. Then test tables/trees/menus with actual viewport sizes, selection, scrolling, callbacks and empty/disabled states. Audit advanced panels individually instead of calling the entire catalog absent. |
| RAPI-09 / 2 | Concrete dialog types exist, but DialogEngine exposes construction/show/close rather than a complete event/update/render interface. enable_async stores Some(()); open/close events are not emitted; close_dialog discards the result. DialogBuffer ignores its z-index parameter. | [engine](../src/widgets/dialog/mod.rs:296), [close result](../src/widgets/dialog/mod.rs:369), [buffer ordering](../src/widgets/dialog/dialog_buffer.rs:54). Uncompiled FFI update is a success placeholder at [ffi/dialog.rs](../src/ffi/dialog.rs:457). | Open a dialog, paint it, interact, receive the actual result, close and restore focus; test stacking, maximum-dialog behavior, cancellation and async completion. Only then expose the native dialog binding. |
| RAPI-10 / 2 | Animation timing/easing/spring machinery exists. Generic typed keyframes return the previous value instead of interpolating; untyped conversion uses T::default. Relative API values assume a base of zero. ScreenManager handles switching hotkeys but does not route other input to the active screen; transition progress is read into an unused variable, so its fade path does not apply opacity. | [keyframes](../src/animation/keyframes.rs:27), [relative values](../src/animation/api.rs:453), [screen events/transitions](../src/screen/manager.rs:219). Typed midpoint reproduced as 0 for 0→10 at 50%. | Intermediate keyframe values and easing are correct; relative values use actual current properties; animation values visibly affect frames; screen transition midpoints differ from both endpoints; active-screen events are delivered. |
| RAPI-11 / 2 | Image decoding/protocol implementations exist in widgets/display/image. Parallel lower-level image paths are incomplete: platform ImageSource::Path samples a synthetic pattern, encoded-memory input is treated as grayscale, and core image placement emits sampled background colors instead of graphics protocol output. HTTP sources explicitly return an error. | [widget image APIs](../src/widgets/display/image/mod.rs:7), [platform sampling](../src/platform/image.rs:423), [core placement](../src/core/surface.rs:2220). Source-confirmed; real host graphics protocols untested here. | Select the supported image path; compare decoded pixels to a known image; verify placement, clipping, cleanup and fallback on each selected terminal protocol. Decide separately whether URL loading belongs in the API. |
| RAPI-12 / 2 | Tokio is an optional dependency/default feature, but display/adaptive imports it unconditionally. | [manifest](../Cargo.toml:39), [import](../src/display/adaptive.rs:8). cargo check --locked --no-default-features fails with E0433. | Either make the dependency mandatory and correct the feature contract, or make the no-default configuration compile and verify its behavior. Run the documented feature matrix. |
| RAPI-13 / 2 | Several public backend/terminal paths remain alongside the recovered SuprTUI and embedded paths. App's legacy first-render path never initializes previous_tree, and uses 11 synthetic insert patches to request repaint. The native App builder selects Debug/Crossterm, not SuprTUI. Old terminal/PTy code is separate from the verified Ghostty-backed embedded module. | [legacy App branch](../src/app.rs:379), [native App backend](../src/ffi/app.rs:185), [legacy terminal](../src/terminal/terminal_impl.rs:41), [public modules](../src/lib.rs:90). Source-confirmed; no claim that every old terminal operation fails. | Name the supported application/backend path, provide explicit adapters or migration for others, and test incremental updates and terminal restoration through every retained public entry point. Do not silently remove old APIs. |
| RAPI-14 / 2 | Binding absence and native incompleteness are different. Rust editor/layout implementations exist, but ffi/editor.rs and ffi/layout.rs are not included in ffi/mod.rs; dialog FFI is also excluded and incomplete. A generic stateful foreign-language component/event bridge still needs design and implementation. | [compiled FFI module list](../src/ffi/mod.rs:10), existing [binding migration](binding-typescript-migration.md). | After native semantics work, choose which APIs need foreign-language access, define state/event/callback ownership, then add consumer tests. Do not recreate wrappers around missing native behavior. |
| RAPI-15 / 2 | Public documentation overstates catalog readiness. Markdown is compiled normally but hidden under cfg(doc)/cfg(doctest). Props derive advertises validation while generated validate returns true. Ignored examples and unit tests do not establish complete user workflows. | [crate claims/module gating](../src/lib.rs:5), [Props macro](../reactive-tui-macros/src/lib.rs:266), [older recon](recon.md). | Publish a supported-API matrix; make documented examples compile; expose/document Markdown consistently; specify and enforce Props validation or narrow its claim. Clearly mark manual adapters and platform limits. |

## Coverage across the public surface

This is a breadth audit with focused reproductions, not a proof of every public
function or an exhaustive memory-safety review. Areas with no new finding are
not certified merely because no placeholder was found.

| Public area | Assessment |
| --- | --- |
| app, backend, embedded | Current SuprTUI/Ghostty/wakeup paths retain prior acceptance evidence; component expansion and legacy alternatives need the work above. |
| component, builder, procedural macros | Registry concurrency and lookup isolation were repaired; lifecycle integration, hook slots and interactive builders remain incomplete. |
| reactive, hooks | Signal-to-App wakes and the shared scheduler have acceptance coverage; component-owned state/effects/context and gesture hooks need completion. |
| event, ui | Real router, focus and hit-test primitives exist; their connection to App's painted tree is missing. ui::Updater is a marker trait, not a working update dispatcher. |
| layout | Taffy layout and basic painting have tested behavior; dynamic state styling and selected accepted tokens do not. |
| core, render, screen, vdom | Substantial buffers, tree/diff and node models exist; multiple rendering routes and unimplemented screen transitions require a coherent public contract. |
| widgets::input/display/layout/menu/dialog | Substantial implementations with uneven integration and per-widget gaps; neither “all missing” nor “production-ready catalog” is accurate. |
| editor | Real editing and gap-buffer code; Unicode position mismatch reproduced. Broader undo/selection/grapheme behavior still needs a dedicated audit. |
| markdown, syntax | Real comrak/syntect-backed conversion/highlighting paths inspected. No new runtime defect established; rendering integration, documentation coverage and large-input behavior are not certified. |
| platform, terminal, escape | Multiple existing parsers/platform implementations; only the recovered declared paths have current end-to-end evidence. Windows/macOS/legacy terminal behavior requires targeted verification. |
| display, theme | Real performance/theme helpers exist. Optional-Tokio build defect confirmed; multi-App global context isolation and theme propagation need focused verification. |
| ffi | Declarations/layouts now agree with native exports; this is not behavioral proof for all 210 functions. Legacy signal destruction is the immediate safety issue. |
| error | Real Rust/FFI errors exist; several higher-level wrappers still discard errors/results as identified above. No comprehensive error-path audit claimed. |

## Recommended implementation sequence

1. **Repair native signal ownership.** Keep this small and independently verifiable.
2. **Complete the Rust component runtime.** Resolve nested components before painting,
   retain one instance per stable key, reset hook slots, own effect cleanup and
   connect component state updates to the already-working App scheduler/waker.
3. **Complete input and focus.** Connect callbacks, layout bounds, routing and stable
   focus identity. Prove a working button and editable input through App.
4. **Repair editor Unicode and clipboard failures**, and settle the optional-Tokio
   build contract. These can be separate bounded commitments.
5. **Complete selected widget families**, starting with dialogs/forms and then
   tables/trees/menus according to actual application needs. Reuse existing logic.
6. **Complete styling, animation/screens and images** against observable terminal
   results; decide which legacy entry points receive adapters versus explicit migration.
7. **Restore selected bindings after the native paths work.** Use the ABI gate and
   real consumers added by the completed binding commitment.

The strongest next foundational deliverable is: a keyed child component renders
inside App, retains state through input and redraw, runs cleanup once when removed,
and receives correct focus and events. This is more useful than adding another
widget declaration while the shared runtime path is incomplete.

## Verification and limits

- Existing baseline: Cairn reported binding-abi-compatibility Done with all 34
  requirements current and passing before this audit. Those checks cover their
  stated renderer, embedded, wakeup, registry, maintenance and ABI contracts.
- New audit: the saved reproduction script was rerun successfully; eight focused
  probes compiled and reproduced the defects described
  above. Their assertions deliberately confirm current incorrect behavior;
  “8 passed” in the log means eight findings reproduced, not eight product fixes.
- The component probe uses the real App and SuprTuiBackend with captured output;
  a VT100 parser verifies the plain-text control. The focus probe compiles the
  actual private focus-manager source in an isolated harness. No production source
  is patched. The initial control incorrectly searched raw ANSI bytes; it was
  corrected to parse the screen before the final recorded run.
- cargo check --locked --no-default-features failed (E0433, unconditional Tokio).
- cargo package --list --locked succeeded; this checks the package file list, not
  a complete publishable archive or downstream build.
- No invalid native pointer/destructor call, live clipboard write, new dependency,
  release, cross-platform certification or feature implementation was performed.
- Graph discovery was unavailable (transport closed). Source maps, direct source
  inspection and focused execution supplied the evidence.

Evidence and rerun instructions are in [rust-api-audit](analysis/rust-api-audit/README.md).
The previous recon/backlog contains issues now fixed by later commitments; it must
not be treated as the current failure list. This report rechecks and separates
remaining issues rather than carrying old failures forward as if still present.

## Audit work tracking

- Complete: map public APIs and distinguish implementation from integration gaps.
- Complete: verify priority findings with focused source and safe probes.
- Complete: produce the prioritized completion report and verify its references.
