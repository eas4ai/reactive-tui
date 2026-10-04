# Core framework static review

Reviewed source: `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. All anchors below are original-relative paths and refer to that frozen source. The live repository was not used as the source of findings.

This was a read-only source audit. No tests, builds, Cargo commands, Clippy, rust-analyzer, benchmarks, lint gates or Sudus commands ran. “Confirmed” means the triggering behavior follows from inspected code; none of the reproductions below was executed. No production files were changed. The only outputs are this report and `core-coverage.json`.

The scope contains 108 Rust files and 43,624 physical source lines, including embedded tests and documentation. Every scoped file received structural navigation with the focused Ripwire map, file/symbol maps and Tilth. The final ledger records 75 focused-deep and 33 structural inspections. These depths describe source inspection, not a claim to have deeply read every line. Tests were mapped and selectively read when they bear directly on a finding. Ordinary Animation, accessibility and graphics internals overlap other reviewers; their findings are not duplicated here. The further pass below reviews the separate component-owned hook animation driver.

## Confirmed defects

## C01 — A dropped old render tree can unregister the replacement component

**Severity:** medium. **Confidence:** high. **Type:** lifecycle/ownership defect. **Reach:** public automatic render conversion and Screen/ScreenManager bookkeeping; App's resolved render conversion avoids it.

**Anchor:** `src/render/tree.rs:228`; registration at `src/render/tree.rs:539`, replacement at `src/component/registry.rs:269`, caller at `src/screen/mod.rs:112`.

`element_to_render_node` uses non-composite keys and automatically constructs registered components. It puts `instance.clone()` into the global registry and keeps the original instance in the node. Cloning constructs a fresh component (`src/component/instance.rs:242`). The registry calls Mount and replaces any entry with the same key. ElementNode::drop later unregisters by key alone; it does not check that the registry entry is the instance this node registered.

**Trigger:** register a component factory, set a Screen's content to that component with key `panel`, then set new content retaining key `panel`. The second conversion registers a new tracked instance and disposes the previous one. Assigning `self.render_tree = new_tree` at `src/screen/mod.rs:129` drops the old node, which removes and unmounts the newly registered instance. Two live Screens with the same local key also share this registry address. The same sequence can be performed directly with two automatic render trees.

**Why not neutralized:** Screen uses the automatic converter for both set_content and get_patches_since_last_render (`src/screen/mod.rs:161`), rather than the non-instantiating resolved converter. Its separate ScreenRuntime mounts its own actual component at `src/screen/runtime.rs:59`, so this bookkeeping also constructs extra components and sends extra mount/unmount callbacks. Cleanup exists and defeats an unbounded unkeyed-instance leak hypothesis; the confirmed issue is replacement ownership and duplicate lifecycle work.

**Consequence:** spurious mount/unmount side effects, duplicate constructor work and premature destruction of the current global tracked instance. **Recommendation:** make Screen bookkeeping non-instantiating and keep component ownership in ScreenRuntime. Where automatic conversion remains public, use an owner token/generation and unregister only the matching entry; namespace identities by tree. **Verification limit:** static constructor/registration/drop trace; no executed lifecycle counter or memory profile.

## C02 — Component::poll_change is never polled by the managed runtimes

**Severity:** medium. **Confidence:** high. **Type:** unsupported async behavior. **Reach:** mounted Components in App and ScreenRuntime.

**Anchor:** `src/component/mod.rs:87`; wrapper at `src/component/instance.rs:301`; managed render path at `src/component/runtime.rs:177`.

The Component trait advertises async state changes through `poll_change(Pin<&mut Self>, Context)`. Its AnyComponent wrapper implements that projection, but the inspected App, ComponentRuntime and ScreenRuntime paths never call poll_change_any. Static symbol usage navigation found only the declaration/wrapper for that method, not a managed-runtime consumer.

**Trigger:** implement a Component whose pending future is first polled in poll_change and registers the supplied waker. Mount it through App. No first poll occurs, so it cannot register a waker or observe completion through this advertised API. Re-rendering does not introduce polling.

**Why not neutralized:** hooks/timers/signals form a different working update path; implementing this trait method does not connect to them automatically. Manual access to an AnyComponent outside the managed runtime does not repair the managed API's contract.

**Consequence:** async components written to the trait contract remain pending indefinitely. **Recommendation:** poll retained instances with an App-owned Context, arrange wake-driven repolls and rerender on Ready, or explicitly remove/deprecate this managed async contract. **Verification limit:** complete production ComponentRuntime and App loop inspection plus static usage search; no executor run.

## C03 — ScreenRuntime dispatch never feeds component mouse hooks

**Severity:** medium. **Confidence:** high. **Type:** missing integration. **Reach:** ScreenManager screens using use_hover/use_drag/use_clicks/use_mouse_position and related hooks.

**Anchor:** `src/screen/runtime.rs:153`; working App counterpart at `src/app.rs:338`.

Components register mouse-hook signals in their runtime's MouseEventProcessor. App resolves the component under the pointer and calls `components.process_mouse_event` before normal routing. ScreenRuntime::process_event calls only EventRouter, including for its synthesized click; it never calls the ComponentRuntime processor.

**Trigger:** mount a component that renders hook hover/click state inside a ScreenManager screen. Deliver mouse Move/Down/Up to the manager. Ordinary element event handlers can run, while the mouse-hook signals remain at their initial values.

**Why not neutralized:** ScreenRuntime uses the same ComponentRuntime hook registration machinery, but normal EventRouter callbacks do not update its processor. Preparing/rendering a screen does not feed mouse events either. ScreenManager does not expose this private processor for callers to supply the missing integration.

**Consequence:** hook-based interactive behavior differs between App and ScreenManager. **Recommendation:** use the same component-target resolution and processor dispatch in both runtimes, including the synthesized click. **Verification limit:** inspected ScreenRuntime's entire production implementation and App's corresponding dispatch; no interactive screen run.

## C04 — Mouse-position hooks keep is_inside=true after the pointer leaves

**Severity:** medium. **Confidence:** high. **Type:** stale state. **Reach:** normal App mouse-hook path.

**Anchor:** `src/hooks/processor.rs:415`; hover-owner transition at `src/hooks/processor.rs:433`; caller at `src/app.rs:341`.

handle_mouse_move updates position_states only for the component currently under the pointer, setting is_inside=true. On a change of owner or a miss, it clears the previous hover signal but never clears the previous position signal.

**Trigger:** keep component A mounted, move into it, then move into component B or outside all hit targets. A's mouse-position state retains its previous position and is_inside=true. Its separate hover signal can correctly become false.

**Why not neutralized:** App feeds raw motion to the processor before routing. Router-synthesized element Enter/Leave handlers are not fed back into it. The processor's explicit handle_leave path does not make ordinary raw Move owner transitions update position_states.

**Consequence:** position-driven tooltips, coordinate displays and controls treat a departed pointer as still inside. **Recommendation:** clear the previous owner's position/inside state during owner transitions, independently of whether a hover hook was registered; track position ownership explicitly. **Verification limit:** static two-Move trace; no terminal event replay.

## C05 — Several conditional CSS variants apply their styles unconditionally

**Severity:** medium. **Confidence:** high. **Type:** incorrect style resolution and unsupported feature claim. **Reach:** normal App class styling.

**Anchor:** `src/layout/css/variants.rs:167`; group variants at `src/layout/css/variants.rs:197`; filtering at `src/app/event_tree.rs:106`.

The active, visited, first, last, odd, even, group-hover, group-focus and group-active helpers all apply the base utility without inspecting the named condition. Group processing strips the prefix and applies it. App's state/viewport filter recognizes focus, focus-within, hover, disabled and four viewport prefixes; it breaks on these other prefixes and leaves them for the CSS parser.

**Trigger:** render `bg-black group-hover:bg-red-500` while no group is hovered: the group style applies anyway. Give every child `first:p-8`: every child receives the first-child padding. No ancestor/group/position input exists in these helpers.

**Why not neutralized:** the App filter explicitly preserves unknown prefixes, and the optimizer delegates them to variants. The parser's success is not evidence that the condition was evaluated. The tests at `src/layout/css/variants.rs:266` and `:285` assert only that a result exists, not whether the style is conditional.

**Consequence:** states and sibling selectors change the initial appearance and cannot express the advertised conditional behavior. **Recommendation:** resolve them with explicit element/ancestor/sibling state or reject unsupported prefixes. Remove the nine identical context-named wrappers until they carry real context. **Verification limit:** parser/filter source trace; no rendered screenshot or executed assertion.

## C06 — CSS-in-Rust display:none does not hide anything

**Severity:** medium. **Confidence:** high. **Type:** silently ignored property value.

**Anchor:** `src/layout/css/css_in_rust.rs:113`.

The Display IntoCssValue implementation returns the input StyleBuilder unchanged for Display::None, with a comment saying visibility handles it. It emits no hidden/display-none state. The separate class utility for hidden has an actual layout representation; this macro path does not select it.

**Trigger:** attach the style produced by `css! { display: Display::None }` to an otherwise visible element. The builder retains its normal display state, so the element still occupies layout/paint space.

**Why not neutralized:** downstream styling receives an ordinary builder snapshot, with no record of the ignored None request to interpret as visibility. **Consequence:** conditionally hidden content remains visible and interactive according to its other metadata. **Recommendation:** set the actual display-none representation and verify geometry/paint/hit behavior, or reject the unsupported enum value. **Verification limit:** static value-to-builder trace; the macro example was not compiled or rendered.

## C07 — Public Memo values never recompute after construction

**Severity:** medium. **Confidence:** high. **Type:** broken computed-signal contract. **Reach:** `reactive::signal::Memo`, not the separate render-time use_memo hook.

**Anchor:** `src/reactive/signal.rs:308`; private recompute at `src/reactive/signal.rs:323`; weak proof at `src/reactive/signal.rs:383`.

Memo computes an initial value, recomputes once during construction, and then get only reads its stored signal. track_dependency stores IDs in a set, but no subscriber or runtime consumes that set. recompute is private and has no production caller after construction.

**Trigger:** construct a Memo computing twice a cloned Signal initially containing 1; set that Signal to 5; read Memo. Its stored value is still 2. Calling track_dependency does not install any notification path.

**Why not neutralized:** Signal notifications reach subscribers/App wake subscriptions, while Memo does not register one. The unit test explicitly invokes the private `doubled.recompute()` after changing the source, a step unavailable to external callers; that proves the calculation function, not automatic recomputation.

**Consequence:** public computed values remain stale forever. **Recommendation:** subscribe and manage dependency ownership, or recompute lazily on dependency versions. If intentionally one-shot, name/document it accordingly and remove the unused dependency facade. **Verification limit:** source-level reproduction and selected test inspection; test not run.

## C08 — RuntimeContext effects are disconnected from signals and consumed after one run

**Severity:** medium. **Confidence:** high. **Type:** broken reactive effect contract. **Reach:** public ReactiveRuntime/RuntimeContext/Effect; not Hooks' separate owned effects.

**Anchor:** `src/reactive/runtime.rs:315`; effect storage at `src/reactive/effect.rs:38`; consumption at `src/reactive/effect.rs:94`; response-to-change claim at `src/reactive/effect.rs:28`.

RuntimeContext::create_signal returns an ordinary Signal without associating it with the runtime. Signal reads/writes use their subscriber/App-wake machinery, not ReactiveRuntime::track_signal/signal_changed. Static usages of those two runtime methods are definitions and tests rather than the public Signal path. Independently, RuntimeContext::create_effect accepts FnOnce and Effect::run takes its stored closure permanently on the first execution.

**Trigger:** create a context Signal and an effect reading it. Registration runs the closure once. Update the Signal: no runtime effect is scheduled. Even manually supplying dependency tracking and signal_changed cannot make the original closure run twice; the next run can execute cleanup and clear dependencies without an effect body.

**Why not neutralized:** registering an EffectId and maintaining queues cannot replace a consumed FnOnce or wire an unassociated Signal. Hooks have a distinct effect implementation and do not repair these exported APIs.

**Consequence:** advertised response-to-signal side effects do not happen; manual notification can prematurely perform cleanup. **Recommendation:** choose one reusable effect/dependency mechanism with explicit lifetime ownership and connect reads/writes to it, or document these as one-shot operations and remove reactive claims/queues. **Verification limit:** full relevant production runtime/effect/signal source trace; no reactive program run.

## C09 — Public VDOM diff ignores node names, element props and event handlers

**Severity:** medium. **Confidence:** high. **Type:** incomplete reconciliation. **Reach:** public vdom diff API; native App conversion bypasses this diff.

**Anchor:** `src/vdom/diff.rs:223`; element diff at `src/vdom/diff.rs:129`; component diff at `src/vdom/diff.rs:169`; coarse kinds at `src/vdom/node.rs:120`.

can_patch compares only the coarse VNodeType enum and key. For elements, the diff handles attrs/class/style/children but ignores tag, typed props and event_handlers. For components it compares props identity and children but ignores component name.

**Trigger:** diff two same-key elements whose only difference is tag A versus B: no replacement or tag update is emitted. The same holds for a changed element typed prop or handler. Diff same-key components with different names and unchanged props/children: no component replacement occurs.

**Why not neutralized:** node_type returns Element/Component rather than the tag/name. Event patch variants existing in patch.rs are not generated by this branch. The working native VNode-to-Element bridge is a separate path; this finding does not allege that ordinary App prop updates use this broken diff.

**Consequence:** clients applying this public diff retain the wrong component type, stale props or stale handlers. **Recommendation:** include tag/name in compatibility and diff all behavior-bearing fields, or narrow/deprecate the standalone API. **Verification limit:** static emitted-patch reasoning; no patch applier or runtime execution inspected outside the owned scope.

## C10 — HandlerLookup aliases distinct node IDs modulo 256

**Severity:** medium. **Confidence:** high. **Type:** wrong-target lookup. **Reach:** public event::cache API; ordinary EventRouter does not currently use it.

**Anchor:** `src/event/cache.rs:185`; untagged retrieval at `src/event/cache.rs:206`; insertion at `src/event/cache.rs:218`.

HandlerLookup maps node_id to node_id % 256 and stores only an Arc<HandlerChain> in the resulting bucket. It records no original NodeId and has no collision fallback.

**Trigger:** insert a chain for NodeId(1), then a different chain for NodeId(257), with the same event discriminant and phase. get(NodeId(1), ...) returns the chain for 257. Looking up any colliding node can also return a chain for which that node never had a registration.

**Why not neutralized:** NodeId is not constrained to 0..255, and public insert/get perform no validation. The nearby PathCache checks the full IDs before accepting an inline hit, demonstrating that collision-safe lookup is already available conceptually. Static usages show HandlerLookup is disconnected from EventRouter, limiting present internal reach but not its exported behavior.

**Consequence:** consumers can deliver input to another element or run an unintended callback. **Recommendation:** retain/compare the full lookup key and use a collision fallback, or use an exact-key map. **Verification limit:** arithmetic and storage proof; no executed lookup test.

## C11 — HandlerChain reports handled and captured input as ignored

**Severity:** medium. **Confidence:** high. **Type:** incorrect dispatch result. **Reach:** public event::cache API; ordinary EventRouter does not currently use it.

**Anchor:** `src/event/cache.rs:76`.

execute returns immediately for Consumed, discards Captured and Handled, then returns Ignored unconditionally. Its documented role is executing handlers in priority order, not intentionally erasing their outcomes.

**Trigger:** add one handler that returns Handled and execute it. Its side effect occurs, but execute returns Ignored. A handler returning Captured receives the same treatment. Normal EventRouter converts both Handled and Captured to an aggregate Handled result while permitting the documented propagation (`src/event/router.rs:369`).

**Why not neutralized:** EventResult carries meaningful distinctions in normal router dispatch. No separate out-parameter carries the lost result. Unused internal status limits current reach but callers of this public helper cannot recover the result without wrapping every handler themselves.

**Consequence:** callers may run fallback actions despite a handled event, including one handled in the capture phase. **Recommendation:** preserve the router's result/propagation semantics in this helper, or explicitly give it a different contract/name. **Verification limit:** direct control-flow proof; no dispatch execution.

## C12 — aspect-auto does not remove an existing aspect constraint

**Severity:** low. **Confidence:** high. **Type:** incorrect reset utility.

**Anchor:** `src/layout/css/containers.rs:106`.

The aspect-auto branch says it removes the constraint but returns the builder unchanged.

**Trigger:** apply aspect-square, then aspect-auto to the same builder (as with the class list `aspect-square aspect-auto`). The previous aspect_ratio remains stored. **Why not neutralized:** StyleBuilder::aspect_ratio sets that field (`src/layout/style.rs:1505`); the reset branch never clears it and the later layout receives the existing constraint.

**Consequence:** auto sizing remains constrained after the reset utility. **Recommendation:** clear the underlying Option or provide a reset builder method. **Verification limit:** static sequential utility application, not a measured layout.

## Misleading implementation claims

These are source-backed accuracy findings, not allegations about authorship or intent.

## C13 — The CSS macro does not validate property names or property-specific types at compile time

**Severity:** low. **Confidence:** high. **Type:** unsupported safety claim.

**Anchor:** `src/layout/css/css_in_rust.rs:1`; macro at `src/layout/css/css_in_rust.rs:67`; generic conversion at `src/layout/css/css_in_rust.rs:84`.

The module advertises compile-time validation and type safety. The macro accepts any identifier and expression, stringifies the identifier and sends both to one IntoCssValue trait. Implementations runtime-match property strings and silently return the builder for unmatched properties. A Display value in `opacity: Display::Flex`, or a misspelled property supplied with an accepted value type, is accepted by these expansion/trait rules and then silently ignored.

**Trigger/reasoning:** use one of those mismatched declarations. There is no property-specific macro arm or type bound to reject it. **Why not neutralized:** a value implementing a generic conversion trait proves only that the value type has some conversion, not that this property/value pair is valid. **Consequence:** readers rely on compile-time rejection that the implementation does not provide. **Recommendation:** typed property-specific expansion or diagnostics for invalid pairs, or state the narrower runtime-conversion contract. **Verification limit:** inspected expansion and trait rules; no compilation was performed, so this is not a reported compiler result.

## C14 — The advertised compile-time perfect component lookup is an empty runtime HashMap

**Severity:** low. **Confidence:** high. **Type:** inaccurate optimization claim/disconnected abstraction.

**Anchor:** `src/component/cache.rs:14`; lazy initializer at `src/component/cache.rs:34`.

CommonComponents is documented as compile-time TypeIds in a perfect hash map. Its field is a standard HashMap, initialized empty at runtime in Lazy. A comment says registry registration populates it, but common_components exposes only a shared immutable reference and static source navigation finds no production consumer/population route. register requires mutable access. The adjacent cache statistics also require manual hit/miss recording and never record evictions.

**Trigger:** ask common_components().get for a normally registered component: this code has never populated its map. **Why not neutralized:** ComponentRegistry maintains separate factory/name maps; the inspected registration implementation does not mutate COMMON_COMPONENTS. **Consequence:** claimed optimization/counters can be mistaken for evidence of a working registry fast path. **Recommendation:** remove or connect the unused facade, correct the HashMap/runtime description and tie statistics to actual operations. **Verification limit:** initializer, API mutability and static usages; no performance measurements or claim that all caches are ineffective.

## C15 — The poll wrapper's pointer checks do not verify pin stability

**Severity:** low. **Confidence:** high. **Type:** misleading defensive code/ornamental safety check.

**Anchor:** `src/component/instance.rs:304`.

self_ptr and self_mut_ptr are computed from the identical expression `self.as_ref().get_ref() as *const ComponentInstanceWrapper<C>` immediately beside each other, so their comparison cannot detect relocation. The following aligned/non-null check is performed on a pointer just produced from a valid Rust mutable reference. Comments cite the first check as additional pin-safety verification, but the real unsafe projection must rely on a structural no-move invariant.

**Trigger/reasoning:** every valid invocation takes the same pointer twice; the relocation branch adds no observed safety condition. **Why not neutralized:** no intervening operation makes the second expression independently test a lifetime/pinning property. **Consequence:** maintenance code and the safety justification give a false impression of a runtime guard; this is not a demonstrated UB finding. **Recommendation:** remove the tautological checks, document the actual projection invariant, and use a pin-projection abstraction if a real structural guarantee is needed. **Verification limit:** expression/unsafe-block inspection only; no memory-safety violation claimed.

## Design risks and unpromoted candidates

These are kept out of the confirmed finding count; they require contract/reach decisions or additional cross-lane evidence.

- **Legacy scheduler unwind recovery:** `src/render/scheduler.rs:224` sets is_running before invoking a user callback at :286 and clears it only at :313. A caught callback panic leaves future execute_frame calls returning 0. The public constructor also permits an FPS that becomes division by zero at :79 or a zero frame budget above 1000 FPS; :264 then reschedules tasks without executing. The main App uses different scheduling/FPS machinery. Decide whether caught panics/extreme FPS belong to this legacy API's supported contract before prioritizing; no panic or constructor was executed.
- **Runtime batch unwind recovery:** `src/reactive/runtime.rs:109` increments batch_depth and decrements only after f returns. A caught panic leaves batching active. This matters only for retained/reused legacy runtimes after caught panics; an RAII guard would make the bookkeeping resilient. No unwind experiment ran.
- **Physical aspect correction:** `src/layout/css/containers.rs:127` says cells are half as wide as tall, then multiplies the desired width/height ratio by 0.5. Under that stated physical assumption, a visual square requires columns/rows=2, not 0.5. This is mathematical source reasoning; the intended unit contract and actual terminal cell dimensions were not validated. Separate from the certain aspect-auto reset defect.
- **Standalone VDOM patch addressing/moves:** `src/vdom/diff.rs:154` recurses using sibling-local indices, and patch variants carry limited location information. The keyed algorithm emits original-index moves without an evident sequential index update. A consuming patch applier/contract outside this lane is needed before asserting a concrete application bug. Names/props/handlers in C09 require no such ambiguity.
- **Legacy transition fidelity:** `src/screen/transitions.rs:332` copies source cells only when alpha exceeds 0.1 rather than blending colors, and its element-to-NodeSpec conversion omits native typed visual metadata. The modern ScreenManager composition path differs. A confirmed user-facing claim needs a decision about continued support for this separate TransitionRenderer API; no screen transition was rendered.
- **Legacy LayoutManager incremental propagation:** update_node at `src/layout/manager.rs:263` marks the node/immediate parent; compute_layout at :307 chooses dirty roots. Ancestor flex allocation and subtree-removal ownership deserve a targeted contract review. Main native retained painting uses a separate Taffy cache, so this was not promoted based on generic layout concerns.

## Metrics, duplication and proof quality

- Scoped inventory is 108 Rust files / 43,624 physical lines, including tests. `core-coverage.json` is the exact file ledger; no percentage of tested correctness is inferred from these numbers.
- Static Ripwire size metrics identify `src/layout/colors.rs:41` (314-line named-color function) and `src/layout/css/visual_style.rs:105` (157-line color parser). Large lookup tables alone are not defects. Multiple palette/color parsing paths add maintenance cost; they should share a supported parser/table when semantics match. No drift was asserted without comparing specific outputs.
- C05's nine context-specific helper wrappers are identical delegations; their extra naming obscures absent context rather than implementing it. C14's unused cache facade and C15's tautological guard are concrete abstraction-cost examples, not conclusions based on writing style.
- The framework exports overlapping retained App/component resources, Screen bookkeeping, standalone VDOM diff, public automatic RenderTree instantiation, standalone RenderScheduler, and legacy ReactiveRuntime/Effect/Memo. C01/C08/C09 show why their invariants must be named separately; an apparent implementation elsewhere does not prove that the API a caller uses is connected to it.
- Selectively read tests provide weak proofs for C05 and C07: existence-only variant checks and a private manual recomputation respectively. This review makes no claim that the complete test suite lacks relevant integration tests, and it did not run any suite.
- Positive source evidence: the inspected hook-animation registry snapshots callbacks before invoking them outside its locks; App owns AnimationManager directly, with no added manager mutex reentry deadlock found. Component registry replacement disposal releases the map guard before user cleanup. These points limit overbroad lock/deadlock allegations. Parent's separate Animation state-lock issue is outside this report.

## Completion and verification limits

The source audit and report/coverage outputs are complete for this lane. Structural coverage is comprehensive; focused deep coverage is explicitly narrower. All reproductions are reasoned source traces. Compilation, executed tests, interactive timing, actual terminal rendering, memory behavior and performance remain unverified by explicit user instruction. No source fixes, no gate receipts and no passed checks are claimed.


## Further production pass: confirmed defects

The following findings use normal mounted App component hooks and the native Element painter. They are separate from the ordinary Animation/lock-free animation findings in the parent report. All line anchors still refer to the same frozen snapshot.

## C16 — Explicit grid colors bypass the element's partial opacity

**Severity:** medium. **Confidence:** high. **Type:** painting correctness defect. **Reach:** native Element rendering with public `Element::with_cells`.

**Anchor:** `src/layout/paint_tree/suprtui.rs:1261`; explicit foreground at :1319 and background at :1313.

The painter preserves the element's own opacity in NodePaint (`src/layout/paint_tree.rs:246`), and visual extraction also multiplies its ordinary foreground/background alpha by that opacity (`src/layout/style.rs:1084`). However, paint_cells defines its full-opacity shortcut and explicit-color multiplier using only `node.parent_opacity`. An explicitly colored cell therefore bypasses its own element's partial opacity.

**Observed caller path:** `Element::with_cells` stores the grid at `src/component/element.rs:305`; element_to_paintspec preserves it at `src/component/bridge.rs:46`. The backend Element frame branch calls paint_frame at `src/backend/suprtui.rs:834`, which dispatches this grid to paint_cells at `src/layout/paint_tree/suprtui.rs:585`. This is the actual native Element frame path, not a disconnected helper.

**Trigger:** put a one-cell grid with an opaque red foreground or background on an element sized to contain it, give that element `opacity-50`, and keep ancestor opacity at 1. The `full` branch remains true. Explicit red is written/blended with full alpha, while fallback element text/colors use the attenuated style. This applies independently to explicit foreground and background.

**Why not neutralized:** the guard at :1243 handles opacity zero, so fully hidden cells disappear correctly. Ancestor opacity works because it is the value this helper multiplies. Neither applies the missing own partial opacity. Explicit per-cell foreground overriding the element foreground is documented; it does not require exempting that cell from element opacity.

**Consequence:** grid-backed UI stays opaque during element fades, unlike its surrounding paint. **Recommendation:** apply own-times-parent opacity to explicit cell colors and the full-opacity shortcut. Preserve the separate fallback-color path, whose style alpha already includes own opacity, to avoid applying it twice. **Verification limit:** source trace and alpha arithmetic; no terminal frame or screenshot was produced.

## C17 — Pausing a hook animation past its deadline permanently removes its task

**Severity:** medium. **Confidence:** high. **Type:** animation lifecycle defect. **Reach:** public use_animation/AnimationHandle in a normally running App.

**Anchor:** `src/hooks/animation.rs:154`; paused callback at :292; pause/resume at :353 and :361.

AnimationRuntime computes task progress from `Instant::now() - task.start_time` (:141-151). It invokes the task, then removes it unconditionally once this wall-clock progress reaches 1 (:153-155). AnimationHandle::pause only marks its controller/state paused. Its task callback returns early when that controller is not playing (:292-294), but that return does not stop deadline-based removal. Resume only flips the controller/state back to playing and never registers replacement work (:361-365).

**Observed caller path:** use_animation creates the component-owned owner/controller at :417-454; animate_to invokes owner.start at :348, which registers the task at :311. The ordinary App loop regards registered hook tasks as active at `src/app.rs:255`, schedules rendering, and calls update_hook_animations at `src/app.rs:661`. The latter directly drives RUNTIME at `src/hooks/animation.rs:210`. ScreenRuntime also drives it at `src/screen/runtime.rs:58`.

**Trigger:** start a positive-duration animation and pause before it reaches its target. Keep the App running long enough for at least one animation update after the original wall-clock deadline. That update skips interpolation because the controller is paused, then removes the task. Resume afterward. The hook reports Playing, but its value remains at the paused sample and there is no task to advance it. A shorter pause also consumes animation time and causes a progress jump on resume.

**Why not neutralized:** liveness/generation checks correctly reject stale or unmounted owners; they do not suspend the task clock. The retained current_animation_id still contains the old ID. A new animate_to call can restart work, but is not what the documented resume operation promises. No separate ordinary AnimationManager update reaches this owner task.

**Consequence:** valid pause/resume loses playback and can leave reported state inconsistent with actual work. **Recommendation:** retain logical elapsed animation time across pauses and resume the remaining task, or cancel/re-register remaining work with a coherent start time. Remove tasks when logical playback completes, rather than solely when a paused task's original deadline passes. **Verification limit:** source proof assumes the App keeps delivering frames past that wall-clock deadline; no sleep, timed execution or test ran.

## C18 — SpringHandle discards applied impulses before calculating motion

**Severity:** medium. **Confidence:** high. **Type:** spring state/data-flow defect. **Reach:** public use_spring/SpringHandle in normal component hooks.

**Anchor:** `src/hooks/animation.rs:509`; spring calculation at :541 and overwritten velocity at :547.

apply_impulse adds force to owner.velocity and starts a task. That task never reads owner.velocity. Each frame calculates position/velocity from the current value, target and a cloned SpringConfig, then overwrites owner.velocity with the calculated result (:538-548). The separate config.velocity feeds the analytic calculation (`src/animation/spring.rs:100` and :157); adding an impulse does not update that field either.

**Observed caller path:** use_spring initializes the owned value/target to the initial value and velocity to zero at `src/hooks/animation.rs:561-585`. apply_impulse registers its callback through start_spring_animation (:522-556). That is the same live RUNTIME driven by App render frames at `src/app.rs:661` and kept active by the App loop at :255-258.

**Trigger:** create `use_spring` for 0.0 with SpringConfig::default(), then apply_impulse(10.0). Before delivery, velocity() reports 10.0. On the first task update, current and target are both zero; calculate_position returns the target (`src/animation/spring.rs:68`) and calculate_velocity returns zero (:145-147). The task writes zero velocity and removes itself as settled (`src/hooks/animation.rs:549-552`). There is no motion. With unequal current/target, different impulse strengths still do not affect the inputs used to compute the next sample.

**Why not neutralized:** generation and component-lifetime guards only determine whether a sample is accepted. They do not feed the impulse into motion. Configuration can set an initial velocity, but the public impulse operation explicitly changes a different field. The settling check guarantees the equal-position trigger terminates immediately despite its nonzero applied impulse.

**Consequence:** impulse-driven spring interactions silently discard force. **Recommendation:** integrate the owned current velocity into each spring step and use actual elapsed frame time, or sample a coherent analytic trajectory whose initial velocity includes the impulse. Also ensure equal position does not count as settled while velocity is nonzero. **Verification limit:** data-flow and zero-displacement branch proof; no frame execution. The fixed 0.016-second sample and ten-second wall-clock task deadline are additional timing concerns, not measured performance claims.

## C19 — Hook animation loop settings never reach its frame driver

**Severity:** medium. **Confidence:** high. **Type:** unsupported animation configuration. **Reach:** public use_animation with finite/infinite loop settings.

**Anchor:** `src/hooks/animation.rs:305`; configuration at :399-402; initial controller construction at :426.

use_animation passes loop_count and loop_behavior into an ordinary Animation stored in its controller. Animation::new maps Some(2) to Count(2), Some(0) to Infinite, and PingPong to auto_reverse (`src/animation/mod.rs:257-274`). The separate hook task does not update that Animation. It reads the controller only as a playing flag, uses config.easing for a single interpolation, stops the controller/state at progress 1 (:292-307), and registers only config.duration (:311). AnimationRuntime removes it at that first duration (:154-155).

**Observed caller path:** a normal mounted component creates the handle with use_animation (:417), then animate_to registers owner.start's hook task (:348, :272-313). App reaches this callback through update_hook_animations at `src/app.rs:661`; AnimationController::is_playing returns only its bool (`src/animation/mod.rs:201`). No ordinary Animation update consumes the stored count/auto_reverse in this path.

**Trigger:** create a positive-duration 0.0-to-1.0 hook animation with loop_count Some(2), or with Some(0) for the ordinary constructor's infinite mode, and call animate_to(1.0). Let normal App frames advance past the first duration. The hook reaches 1.0, reports Stopped and loses its task. It never executes a second cycle; PingPong also never performs its configured return cycle.

**Why not neutralized:** the copied configuration contains the requested settings, but its frame callback consults only easing and its registration consults only duration. The controller starts/stops the contained Animation without delivering its update loop. Updating the hook config on rerender (:449) cannot repair this absent driver. The conflicting comment that None means infinite (:399), while Animation::new treats None as no loop, is a separate documentation inconsistency; this finding uses explicit Some counts and does not depend on interpreting that comment.

**Consequence:** valid loop configuration silently behaves as one-shot playback. **Recommendation:** give the hook driver a logical cycle clock that consumes count, repeat direction and completion state, or actually drive its contained Animation and publish those samples. Keep current configuration changes coherent with that authority. **Verification limit:** direct configuration/control-flow trace; assumes a running App delivers a frame at/after the positive wall-clock duration. No animation was executed. This is distinct from the parent report's ordinary Animation callback/autoreverse issues.

## Further-pass inspection notes and limits

- Read the actual production bodies of component element/builtin/layout-info/lifecycle/props/state-flags/change modules, the remaining focus/ref helpers, reactive exports, performance context, animation hooks, notification frames, grid payload/painter tail, inline styles, color cache and UI paint/hit grids. Coverage notes identify production ranges and leave tests structural/selective.
- Rechecked component prop retention through runtime.rs:192 and instance.rs:273. Equal typed props cause an early return before copying into the retained component. Callback omission in Table/Tree/DataTable equality has a supported stale-closure trigger; the widgets reviewer owns that finding (W08), so it is not counted again here.
- Positive boundaries inspected: inline-style parsing rejects unknown properties and invalid/non-finite lengths; this does not supply compile-time property validation to css!. Element metadata/event identity participates in equality. Hook resources use weak owner references, liveness and generation checks. Notification frames pop through RAII. The class cache restricts caching to its supported theme context and invalidates by theme generation. No new defect was promoted in those examined boundaries.
- Geometry inspection covered retained layout measurements, collection, clipping/masks, transforms, paint/hit positioning and grid color blending. It supports C16; no additional geometry issue was promoted from CSS conventions alone. A possible negative-z hierarchy semantics mismatch and legacy ChangeBatch optimization concerns remain unpromoted because the supported contract/current App consumption was not established.
- The ordinary AnimationController and SpringConfig bodies, plus the native backend Element dispatch, were read selectively to establish C16-C19 reach. The scoped ledger remains the 108 owned files; those supporting paths are explicitly named in the findings rather than counted as wholly reviewed files.
- This pass is complete as a static source audit. Four new findings bring this lane to 19. The final scoped ledger has 108 files: 75 focused-deep and 33 structural. The report self-audit checked caller reach, distinct triggers, exact anchors, coverage wording and verification limits. This pass ran no tests, builds, checkers, lints or Sudus commands; runtime rendering/timing remains unexecuted.
