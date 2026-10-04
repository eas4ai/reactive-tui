# Reactive TUI production code review

The review found **64 evidenced findings and 4 separately qualified risks**. Six findings are high priority. The recurring problem is disconnected or competing public implementations: API fields, helpers and validation-looking code exist while the behavior or integration they describe is missing. There are also concrete defects in the retained App, chart, control and hook paths.

This was a read-only source review. Only review artifacts were written inside the repository. No project Rust code was compiled or executed; no tests, Clippy, builds, benchmarks or rust-analyzer ran. Sudus's existing request to run GFX-005 was not advanced because the user explicitly prohibited execution/state-changing work.

“Evidenced” means supported by inspected code/callers and reasoned triggering sequences. It does not mean runtime reproduction. Contradicted claims are recorded below without inferring author intent, AI authorship, or asserting every possible defect was discovered.

## Highest priorities

| ID | Finding | Affected route / concrete trigger |
| --- | --- | --- |
| [N02](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:39) | Safe Rust FFI exports dereference caller-controlled pointers | Optional FFI: public safe Rust functions dereference arbitrary caller pointers; caller memory obligations are not reflected in signatures. |
| [N01](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:29) | Animation callbacks deadlock on ordinary state reads | Exported Animation/manager: on_update calls a normal animation getter while update holds the write lock, producing deadlock. |
| [W01](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:9) | Finite subnormal chart data can hang tick generation and shutdown | Chart domain/ticks: finite minimum subnormal data produces a zero step and an infinite loop; worker shutdown joins it. Tick-format builders can also hang synchronously. |
| [W02](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:25) | Theme aliases can recurse until the process exhausts its stack | Theme aliases: two short mutually referring variables recurse until stack exhaustion. |
| [W04](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:53) | Virtual chart slot counts amplify tiny input into unbounded allocation | Charts: one datum plus a large virtual slot count allocates millions of labels/ticks/accumulators despite the cell cap; capacity failure/conditional OOM. |
| [T01](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:35) | Windows direct backend never reads input in the App polling loop | Windows DirectTtyBackend: App polls with timeout0; the backend returns before reading queued input. Default SuprTUI is a different path. |

Other supported-path defects include unconditional CSS variants, Screen mouse-hook/lifecycle divergence, explicit grid colors ignoring their own opacity, pause/resume and loop failures in animation hooks, ignored spring impulses, spring completion at a partial target, inaccurate mixed numeric sorting and stale callbacks after retained widget prop replacement. The full index below distinguishes those from auxiliary public API defects.

## Read the review

- [Architecture, workspace/module responsibilities and competing ownership paths](/home/shawn/workspace2/reactive-tui/.review/ARCHITECTURE.md)
- [Claims audit: contradicted behavior, unsupported claims and accusations deliberately rejected](/home/shawn/workspace2/reactive-tui/.review/CLAIMS.md)
- [Complexity hotspots, meaningful duplicates, ornamental checks and tool limits](/home/shawn/workspace2/reactive-tui/.review/COMPLEXITY_DUPLICATION.md)
- [Core App/components/hooks/reactivity/layout/events/VDOM/screens: 19 findings](/home/shawn/workspace2/reactive-tui/.review/CORE.md)
- [Animation, native C ABI, editor/syntax/Markdown and release metadata: 24 findings](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md)
- [Terminal/platform/backend/graphics and vendored crates: 13 findings plus 4 risks](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md)
- [Widgets/builders/theme: 8 findings](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md)
- [Every captured Rust file: 769 files, 310,890 physical lines](/home/shawn/workspace2/reactive-tui/.review/RUST_MAP.md)
- [Rust declaration/symbol inventory: 19,980 parser symbols across 767 indexed files](/home/shawn/workspace2/reactive-tui/.review/RUST_SYMBOLS.md)
- [Per-file review depth and limits for all 769 files](/home/shawn/workspace2/reactive-tui/.review/COVERAGE.md)
- [Concurrent source and contract changes during review](/home/shawn/workspace2/reactive-tui/.review/DRIFT.md)

The three delegated reviewers were GPT-6.1-Sol at High effort. Independent cross-checks are in [CROSS_CHECK.md](/home/shawn/workspace2/reactive-tui/.review/CROSS_CHECK.md), [CROSS_CHECK_WIDGETS.md](/home/shawn/workspace2/reactive-tui/.review/CROSS_CHECK_WIDGETS.md) and [CROSS_CHECK_TERMINAL.md](/home/shawn/workspace2/reactive-tui/.review/CROSS_CHECK_TERMINAL.md). Several initial hypotheses were narrowed or rejected: the unkeyed registry leak had cleanup, ordinary decorated buttons are a legitimate builder pattern, some optimized metrics are real, and new owned APIs must not inherit old facade accusations.

## Complete findings index

High/medium/low are normalized review priorities; the subsystem reports preserve their original P1/P2 terminology and confidence. The four T12–T15 design risks are listed separately and are not counted as confirmed behavioral defects. Some evidenced findings concern documentation or unused public abstractions rather than broken runtime behavior.

| ID | Priority | Finding | Primary source |
| --- | --- | --- | --- |
| [N01](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:29) | high | Animation callbacks deadlock on ordinary state reads | [src/animation/mod.rs:419](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:419) |
| [N02](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:39) | high | Safe Rust FFI exports dereference caller-controlled pointers | [src/ffi/reactive.rs:538](/home/shawn/workspace2/reactive-tui/src/ffi/reactive.rs:538) |
| [T01](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:35) | high | Windows direct backend never reads input in the App polling loop | [src/platform/windows.rs:274](/home/shawn/workspace2/reactive-tui/src/platform/windows.rs:274) |
| [W01](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:9) | high | Finite subnormal chart data can hang tick generation and shutdown | [src/widgets/display/charts/plot/tick.rs:71](/home/shawn/workspace2/reactive-tui/src/widgets/display/charts/plot/tick.rs:71) |
| [W02](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:25) | high | Theme aliases can recurse until the process exhausts its stack | [src/theme/mod.rs:130](/home/shawn/workspace2/reactive-tui/src/theme/mod.rs:130) |
| [W04](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:53) | high | Virtual chart slot counts amplify tiny input into unbounded allocation | [src/widgets/display/charts/live/canvas/cartesian.rs:542](/home/shawn/workspace2/reactive-tui/src/widgets/display/charts/live/canvas/cartesian.rs:542) |
| [C01](/home/shawn/workspace2/reactive-tui/.review/CORE.md:11) | medium | A dropped old render tree can unregister the replacement component | [src/render/tree.rs:228](/home/shawn/workspace2/reactive-tui/src/render/tree.rs:228) |
| [C02](/home/shawn/workspace2/reactive-tui/.review/CORE.md:25) | medium | Component::poll_change is never polled by the managed runtimes | [src/component/mod.rs:87](/home/shawn/workspace2/reactive-tui/src/component/mod.rs:87) |
| [C03](/home/shawn/workspace2/reactive-tui/.review/CORE.md:39) | medium | ScreenRuntime dispatch never feeds component mouse hooks | [src/screen/runtime.rs:153](/home/shawn/workspace2/reactive-tui/src/screen/runtime.rs:153) |
| [C04](/home/shawn/workspace2/reactive-tui/.review/CORE.md:53) | medium | Mouse-position hooks keep is_inside=true after the pointer leaves | [src/hooks/processor.rs:415](/home/shawn/workspace2/reactive-tui/src/hooks/processor.rs:415) |
| [C05](/home/shawn/workspace2/reactive-tui/.review/CORE.md:67) | medium | Several conditional CSS variants apply their styles unconditionally | [src/layout/css/variants.rs:167](/home/shawn/workspace2/reactive-tui/src/layout/css/variants.rs:167) |
| [C06](/home/shawn/workspace2/reactive-tui/.review/CORE.md:81) | medium | CSS-in-Rust display:none does not hide anything | [src/layout/css/css_in_rust.rs:113](/home/shawn/workspace2/reactive-tui/src/layout/css/css_in_rust.rs:113) |
| [C07](/home/shawn/workspace2/reactive-tui/.review/CORE.md:93) | medium | Public Memo values never recompute after construction | [src/reactive/signal.rs:308](/home/shawn/workspace2/reactive-tui/src/reactive/signal.rs:308) |
| [C08](/home/shawn/workspace2/reactive-tui/.review/CORE.md:107) | medium | RuntimeContext effects are disconnected from signals and consumed after one run | [src/reactive/runtime.rs:315](/home/shawn/workspace2/reactive-tui/src/reactive/runtime.rs:315) |
| [C09](/home/shawn/workspace2/reactive-tui/.review/CORE.md:121) | medium | Public VDOM diff ignores node names, element props and event handlers | [src/vdom/diff.rs:223](/home/shawn/workspace2/reactive-tui/src/vdom/diff.rs:223) |
| [C10](/home/shawn/workspace2/reactive-tui/.review/CORE.md:135) | medium | HandlerLookup aliases distinct node IDs modulo 256 | [src/event/cache.rs:185](/home/shawn/workspace2/reactive-tui/src/event/cache.rs:185) |
| [C11](/home/shawn/workspace2/reactive-tui/.review/CORE.md:149) | medium | HandlerChain reports handled and captured input as ignored | [src/event/cache.rs:76](/home/shawn/workspace2/reactive-tui/src/event/cache.rs:76) |
| [C16](/home/shawn/workspace2/reactive-tui/.review/CORE.md:238) | medium | Explicit grid colors bypass the element's partial opacity | [src/layout/paint_tree/suprtui.rs:1261](/home/shawn/workspace2/reactive-tui/src/layout/paint_tree/suprtui.rs:1261) |
| [C17](/home/shawn/workspace2/reactive-tui/.review/CORE.md:254) | medium | Pausing a hook animation past its deadline permanently removes its task | [src/hooks/animation.rs:154](/home/shawn/workspace2/reactive-tui/src/hooks/animation.rs:154) |
| [C18](/home/shawn/workspace2/reactive-tui/.review/CORE.md:270) | medium | SpringHandle discards applied impulses before calculating motion | [src/hooks/animation.rs:509](/home/shawn/workspace2/reactive-tui/src/hooks/animation.rs:509) |
| [C19](/home/shawn/workspace2/reactive-tui/.review/CORE.md:286) | medium | Hook animation loop settings never reach its frame driver | [src/hooks/animation.rs:305](/home/shawn/workspace2/reactive-tui/src/hooks/animation.rs:305) |
| [N03](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:49) | medium | Reversing a playing animation stops its updates | [src/animation/mod.rs:391](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:391) |
| [N04](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:59) | medium | A parallel timeline completes while its animations are waiting for delay | [src/animation/mod.rs:985](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:985) |
| [N05](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:69) | medium | Loop callbacks and auto-reverse are stranded in unused completion helpers | [src/animation/mod.rs:493](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:493) |
| [N06](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:79) | medium | Stale-animation cleanup deletes animations that are actively progressing | [src/animation/mod.rs:1116](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:1116) |
| [N07](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:89) | medium | Automatically generated animation IDs collide within a millisecond | [src/animation/mod.rs:264](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:264) |
| [N08](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:99) | medium | Interpolation cache treats different endpoint values as the same animation | [src/animation/performance.rs:304](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:304) |
| [N09](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:109) | medium | Optimized batching changes animation values instead of preserving semantics | [src/animation/performance.rs:114](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:114) |
| [N10](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:119) | medium | Lock-free animation state loses updates and cannot finish larger loop counts | [src/animation/lock_free.rs:89](/home/shawn/workspace2/reactive-tui/src/animation/lock_free.rs:89) |
| [N11](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:129) | medium | Grid stagger distance overflows for ordinary terminal-sized grids | [src/animation/stagger.rs:163](/home/shawn/workspace2/reactive-tui/src/animation/stagger.rs:163) |
| [N12](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:139) | medium | SyntaxEditor discards document context when painting highlighted lines | [src/editor/syntax_editor.rs:182](/home/shawn/workspace2/reactive-tui/src/editor/syntax_editor.rs:182) |
| [N13](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:149) | medium | Incremental highlighting reparses the entire document and has inconsistent size guards | [src/syntax/highlighter.rs:121](/home/shawn/workspace2/reactive-tui/src/syntax/highlighter.rs:121) |
| [N14](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:159) | medium | C text-buffer rendering corrupts wide and joined Unicode text | [src/ffi/text.rs:306](/home/shawn/workspace2/reactive-tui/src/ffi/text.rs:306) |
| [N15](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:169) | medium | C text-input and checkbox constructors create static representations | [src/ffi/widgets.rs:22](/home/shawn/workspace2/reactive-tui/src/ffi/widgets.rs:22) |
| [N16](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:181) | medium | Animation performance features and metrics are partly decorative | [src/animation/performance.rs:198](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:198) |
| [N17](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:191) | medium | FFI hit grid and debug API expose successful no-ops and fabricated hit IDs | [src/ffi/stats.rs:120](/home/shawn/workspace2/reactive-tui/src/ffi/stats.rs:120) |
| [N20](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:221) | medium | The legacy C effect and hooks facade is not an automatic reactive system | [src/ffi/reactive.rs:732](/home/shawn/workspace2/reactive-tui/src/ffi/reactive.rs:732) |
| [N22](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:241) | medium | C surface-to-terminal rendering ignores the supplied terminal and restores the host after each call | [src/ffi/lib.rs:628](/home/shawn/workspace2/reactive-tui/src/ffi/lib.rs:628) |
| [N23](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:253) | medium | Spring position starts with the opposite of its configured velocity | [src/animation/spring.rs:100](/home/shawn/workspace2/reactive-tui/src/animation/spring.rs:100) |
| [N24](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:263) | medium | Completed spring animations can stop far short of their target | [src/animation/easing.rs:196](/home/shawn/workspace2/reactive-tui/src/animation/easing.rs:196) |
| [T02](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:49) | medium | A lone Escape stays pending forever on Unix input paths | [src/platform/mod.rs:578](/home/shawn/workspace2/reactive-tui/src/platform/mod.rs:578) |
| [T03](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:63) | medium | Direct bracketed paste becomes ordinary key actions | [src/platform/parser.rs:685](/home/shawn/workspace2/reactive-tui/src/platform/parser.rs:685) |
| [T04](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:77) | medium | TerminalWriter resets foreground and background when changing attributes | [src/core/writer.rs:182](/home/shawn/workspace2/reactive-tui/src/core/writer.rs:182) |
| [T05](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:93) | medium | Public escape parser treats UTF-8 bytes as characters and controls | [src/escape/parser/mod.rs:175](/home/shawn/workspace2/reactive-tui/src/escape/parser/mod.rs:175) |
| [T06](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:107) | medium | Public escape parser mishandles both BEL and ESC-ST string endings | [src/escape/parser/mod.rs:73](/home/shawn/workspace2/reactive-tui/src/escape/parser/mod.rs:73) |
| [T07](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:123) | medium | Direct capability probing consumes and discards keys typed at startup | [src/platform/mod.rs:1233](/home/shawn/workspace2/reactive-tui/src/platform/mod.rs:1233) |
| [T08](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:137) | medium | TokioEventLoop permanently changes the caller's stdin to nonblocking | [src/platform/loop.rs:608](/home/shawn/workspace2/reactive-tui/src/platform/loop.rs:608) |
| [T09](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:151) | medium | Adaptive FPS creation and Auto mode ignore configured bounds | [src/display/adaptive.rs:104](/home/shawn/workspace2/reactive-tui/src/display/adaptive.rs:104) |
| [T10](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:167) | medium | Rgba's advertised WCAG contrast result can approve inadequate contrast | [src/core/surface.rs:451](/home/shawn/workspace2/reactive-tui/src/core/surface.rs:451) |
| [T11](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:181) | medium | A stopped TokioEventLoop silently stops again when restarted | [src/platform/loop.rs:665](/home/shawn/workspace2/reactive-tui/src/platform/loop.rs:665) |
| [T16](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:243) | medium | Adding diff statistics changes resize rendering semantics | [src/core/span_diff.rs:224](/home/shawn/workspace2/reactive-tui/src/core/span_diff.rs:224) |
| [T17](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:257) | medium | GraphemeSurface overwrites leave invalid wide-cell occupancy | [src/core/grapheme_cell.rs:154](/home/shawn/workspace2/reactive-tui/src/core/grapheme_cell.rs:154) |
| [W03](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:39) | medium | Numeric scatter axis limits filter indices instead of x values | [src/widgets/display/charts/live/canvas/cartesian.rs:286](/home/shawn/workspace2/reactive-tui/src/widgets/display/charts/live/canvas/cartesian.rs:286) |
| [W05](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:71) | medium | Indexed terminal and captured-image colors use the wrong cube levels | [src/theme/ansi.rs:146](/home/shawn/workspace2/reactive-tui/src/theme/ansi.rs:146) |
| [W06](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:87) | medium | One scrollbar can cause overflow without the other scrollbar appearing | [src/widgets/layout/scroll_view.rs:219](/home/shawn/workspace2/reactive-tui/src/widgets/layout/scroll_view.rs:219) |
| [W07](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:101) | medium | Mixed integer and decimal table cells lose exact numeric ordering | [src/widgets/display/table.rs:727](/home/shawn/workspace2/reactive-tui/src/widgets/display/table.rs:727) |
| [W08](/home/shawn/workspace2/reactive-tui/.review/WIDGETS.md:117) | medium | Replacing a table or tree callback alone keeps calling the old callback | [src/widgets/display/table.rs:194](/home/shawn/workspace2/reactive-tui/src/widgets/display/table.rs:194) |
| [C12](/home/shawn/workspace2/reactive-tui/.review/CORE.md:163) | low | aspect-auto does not remove an existing aspect constraint | [src/layout/css/containers.rs:106](/home/shawn/workspace2/reactive-tui/src/layout/css/containers.rs:106) |
| [C13](/home/shawn/workspace2/reactive-tui/.review/CORE.md:179) | low | The CSS macro does not validate property names or property-specific types at compile time | [src/layout/css/css_in_rust.rs:1](/home/shawn/workspace2/reactive-tui/src/layout/css/css_in_rust.rs:1) |
| [C14](/home/shawn/workspace2/reactive-tui/.review/CORE.md:189) | low | The advertised compile-time perfect component lookup is an empty runtime HashMap | [src/component/cache.rs:14](/home/shawn/workspace2/reactive-tui/src/component/cache.rs:14) |
| [C15](/home/shawn/workspace2/reactive-tui/.review/CORE.md:199) | low | The poll wrapper's pointer checks do not verify pin stability | [src/component/instance.rs:304](/home/shawn/workspace2/reactive-tui/src/component/instance.rs:304) |
| [N18](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:201) | low | Gap-buffer zero-copy and constant-time claims omit its actual work | [src/editor/gap_buffer.rs:1](/home/shawn/workspace2/reactive-tui/src/editor/gap_buffer.rs:1) |
| [N19](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:211) | low | Package, README and native version disagree | [src/ffi/mod.rs:183](/home/shawn/workspace2/reactive-tui/src/ffi/mod.rs:183) |
| [N21](/home/shawn/workspace2/reactive-tui/.review/NATIVE_ANIMATION_TEXT.md:231) | low | Markdown table separators are generated from a permanently empty column list | [src/markdown/ast_walker.rs:184](/home/shawn/workspace2/reactive-tui/src/markdown/ast_walker.rs:184) |

## Qualified risks

| ID | Priority | Risk | Limit |
| --- | --- | --- | --- |
| [T12](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:195) | medium | Resize dispatcher teardown can clear a new owner's callbacks | Concrete teardown/new-owner interleaving; no internal default-App callback consumer established. |
| [T13](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:207) | medium | Independently created native TTY sessions restore each other's terminal state | Independent session ownership mechanism is unsafe to overlap; simultaneous independent sessions are not clearly promised. |
| [T14](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:219) | medium | Graphics initialization discards unrelated process stderr | Process stderr suppression is explicit in source; affects unrelated tty diagnostics during optional graphics initialization; no duration measured. |
| [T15](/home/shawn/workspace2/reactive-tui/.review/TERMINAL_GRAPHICS.md:231) | medium | Startup parsing remains in special mode indefinitely if no DA1 arrives | Startup parser flag survives timeout; intended late-reply/ambiguous Alt-key contract is unresolved. |

## Coverage and practical limits

The entire captured tracked/nonignored Rust codebase was mapped. **548 files** are categorized as production/module/build source; **265** received focused body/caller review and **283** structural inspection. The other **221 files** are dedicated tests, examples, verification fixtures, benches or a generator tool. Across all files there are 265 deep, 312 structural and 192 inventory-only records. Inline tests remain part of production-file physical line counts. Focused deep does not mean every statement or branch was audited. Large generated bindings and retained upstream code mostly received structural coverage.

Source inspection covered supported App/frame/input lifecycles, retained controls and prop updates, chart worker/domain/ticks/budgets, styles and geometry, reactive/effect ownership, terminal session/input parsing, optional graphics failure boundaries, FFI handle/callback APIs, text/grapheme painting, syntax caches and owned accessibility transport. Boundary areas without a confirmed issue are recorded in the subsystem reports; that is not proof they are defect-free.

All Windows/platform/feature behavior is inferred from source; none was built or run. No visual parity, runtime timing, memory pressure, race schedule, external client, reader integration or compiler result was validated. Historical receipts/benchmark claims were not reproduced. Source manifests and hashes make the input reproducible; they do not convert static analysis into passing checks.

## Recommended order of attention

1. Address the six high-priority termination, memory-safety and unusable-input findings.
2. Repair retained-path state/interaction behavior: C01–C06/C16–C19, W03/W06–W08 and spring completion/velocity. Preserve clear ownership through callbacks, prop replacement and shutdown.
3. Decide which auxiliary APIs are actually supported. Connect or deprecate legacy Memo/RuntimeContext/VDOM/cache/C control/effect/optimized-animation facades; route adapters through the existing working authority.
4. Remove ornamental checks/dead path markers and synchronize claims with behavior. Consolidate duplicates that already diverge before refactoring merely to lower a metric.

These are review recommendations only; no fixes or migrations were performed.

## Navigation tools

Tilth plus Ripwire worked well for this review. Ripwire complemented Tilth with a global AST/symbol map, static complexity and normalized-body clone reports without starting a language server. Scoped rg searches confirmed literal integration references. Call graphs remain heuristic and clone results include vendor/example/generated noise; concrete callers were inspected for material findings. No additional tools were installed.

## Work and verification log

- [done] Freeze inputs, inventory all Rust files and assign coverage.
- [done] Review production subsystems; map tests and inspect them only as supporting source.
- [done] Validate callers, independently cross-check major/late findings, reject false positives and deduplicate.
- [done] Publish the index/claims/coverage/drift artifacts and verify report consistency.

[Artifact consistency audit](/home/shawn/workspace2/reactive-tui/.review/ARTIFACT_AUDIT.md) completed successfully for JSON/counts, unique finding IDs, source/line bounds, coverage completeness, local link targets and frozen input hashes. Live-source drift is recorded separately. It is not a project test/lint/quality-gate result. Review findings remain unfixed by design.
