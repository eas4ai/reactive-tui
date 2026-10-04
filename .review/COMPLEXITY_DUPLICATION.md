# Complexity, duplication and ornamental implementation

These are static maintenance observations. Complexity estimates are not a defect by themselves; clone similarity is not proof two functions should be merged. No benchmark, compiler, lint or test ran.

## Concentrated complexity

Ripwire's tree-sitter estimates for production function bodies, including branches that may be feature-gated or interspersed with inline tests:

| Function | File | Span LOC | Estimated cyclomatic | Estimated cognitive |
| --- | --- | ---: | ---: | ---: |
| cartesian | [src/widgets/display/charts/live/canvas/cartesian.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/charts/live/canvas/cartesian.rs) | 1197 | 218 | 535 |
| Runtime::render | [src/widgets/display/modal/live/render.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/modal/live/render.rs) | 356 | 56 | 102 |
| Popover::render_live | [src/widgets/display/popover/live/render.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/popover/live/render.rs) | 297 | 57 | 100 |
| DiffWriter::try_diff | [src/core/surface.rs](/home/shawn/workspace2/reactive-tui/src/core/surface.rs) | 212 | 40 | 88 |
| Window::print | [src/core/window.rs](/home/shawn/workspace2/reactive-tui/src/core/window.rs) | 160 | 28 | 87 |
| run_worker | [src/backend/suprtui.rs](/home/shawn/workspace2/reactive-tui/src/backend/suprtui.rs) | 272 | 43 | 79 |
| render | [src/widgets/terminal/paint.rs](/home/shawn/workspace2/reactive-tui/src/widgets/terminal/paint.rs) | 213 | 37 | 78 |
| paint_node | [src/layout/paint_tree/suprtui.rs](/home/shawn/workspace2/reactive-tui/src/layout/paint_tree/suprtui.rs) | 166 | 32 | 77 |
| LiveAutocomplete::render | [src/widgets/dialog/autocomplete/live.rs](/home/shawn/workspace2/reactive-tui/src/widgets/dialog/autocomplete/live.rs) | 237 | 34 | 74 |
| LiveTree::handle_event | [src/widgets/display/tree/live.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/tree/live.rs) | 189 | 47 | 73 |
| LiveTable::handle_event | [src/widgets/display/table/live.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/table/live.rs) | 231 | 57 | 70 |
| LiveProgress::render | [src/widgets/display/progress_bar/live.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/progress_bar/live.rs) | 178 | 28 | 69 |
| Explorer::paint | [src/widgets/display/file_explorer/live/paint.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/file_explorer/live/paint.rs) | 342 | 42 | 64 |
| AstWalker::walk_node | [src/markdown/ast_walker.rs](/home/shawn/workspace2/reactive-tui/src/markdown/ast_walker.rs) | 256 | 50 | 64 |
| Tree::handle_key_navigation | [src/widgets/display/tree.rs](/home/shawn/workspace2/reactive-tui/src/widgets/display/tree.rs) | 167 | 39 | 64 |

The chart Cartesian renderer combines domain choice, numeric/category limits, tick labels, virtual slots, plot geometry, grids and many plot families. W01/W03/W04 establish actual contract failures at those joins. Split domain/viewport selection and bounded tick generation from type-specific drawing first. The modal/popover/control render functions deserve focused extraction around layout and focus/interaction contracts; merely moving branches into helpers would not reduce integration risk. The renderer/event-router branches represent real work and need contract-preserving boundaries, not a score-driven rewrite.

Raw metrics include `tested` flags from static source heuristics. They are not executed coverage or passing test evidence. Call edges are heuristic: the full captured mixed-language graph has 3,155 ambiguous and 3,491 unresolved edges; by-name no-match cannot prove a path is unused.

## Duplicated behavior that already diverges

| Duplication | Evidence and failure | Useful consolidation |
| --- | --- | --- |
| Animation live completion plus unused helper completion | N05: the live branch omits callbacks/auto-reverse implemented in dead helpers | One transition path, callbacks invoked outside locks |
| FFI text Surface and Renderer painters | N14: two copied scalar-coordinate loops have the same width defect | Shared grapheme-aware native painter, selection indices kept separately |
| Plain span diff versus diff with stats | T16: the statistics variant drops resize clearing | Instrument one diff path, preserving output semantics |
| Full-document syntax cache and editor line cache | N12/N13: first result discarded, second parser loses context | One context-aware cache/parser authority |
| Several mouse/event/runtime paths | C03/C04: Screen omits processor dispatch; raw Move transition omits position leave | Shared component-target dispatch and transition semantics |
| Reactive runtime/owned Hooks/C manual callback facade | C08/N20: exported implementations have incompatible reactivity | State which API owns effects and dependencies; adapters use that mechanism |
| Multiple color/luminance formulas | T10: core helper skips linearization while theme/backend versions do it | One explicit color-space contract and shared formula where types permit |
| Several public parsers/session owners/render primitives | T02–T08/T13/N22/T17: related features diverge by route | Narrow supported authorities before adding another layer |

The clone detector found matching normalized attribute-delta bodies at `core/span_diff.rs:106` and `core/surface.rs:2247`; image display-size helpers at `widgets/display/image/protocol_renderer.rs:199` and `sixel_renderer.rs:98`; resize helpers at `:232` and `:132`. These are concrete maintenance candidates, without a confirmed bug solely from similarity. Use the already shared image/protocol/geometry model when contracts agree.

Axis-symmetric CSS width/height and margin/padding helpers, theme presets differing mainly by data, typed FFI field accessors, and Unix/Windows PTY wrappers also appear as clones. Some of that is legitimate declarative or platform repetition. A normalized-token score alone is not a reason to unify unrelated contracts.

## Source patterns that create an impression of implementation

- C13: a generic conversion macro is described as compile-time property/type validation, yet invalid pairs silently return the builder.
- C14: an empty runtime HashMap is described as a compile-time perfect component map.
- C15: two adjacent identical pointer expressions are compared as purported pin-stability verification. This is decorative validation, not demonstrated UB.
- N16: visibility always true, caching/callback wrapping empty, discarded measurements and an empty verbose branch sit behind optimization/debug configuration APIs. Real batch totals/averages still exist.
- N17: a hit-grid registration is discarded, while bounds checks fabricate hit ID 1; dumpBuffers logs without dumping.
- N20: the C reactive effect is a callback tuple with manual run, despite complete reactive integration language.
- WIDGETS observations: unused Cartesian `fit_label`/`_sink` functions are retained for path checks and suppress dead-code warnings. They add no rendering behavior.
- TERMINAL_GRAPHICS observations: EventQueue is called lock-free while mutation requires &mut self and its threaded owner locks it. Atomics do not establish the advertised shared-queue contract.

These patterns are what this review treats as “slop”: concrete excess surface, disconnected state and misleading validation/performance language. They do not establish that AI authored the code or that someone intended deception. Blanket dead-code allowances can conceal incomplete public alternatives; scoped removal/deprecation is more useful than another facade.

## Detector limits and tool choice

`duplicate-bodies.xml` records a mixed-language snapshot scan with `--ignore-tests`: 13,671 duplicate body lines out of 144,944 considered body lines (9.4%), 374 clone clusters, and 500 of 1,370 type-2/type-3 groups shown. The displayed window is capped. Examples, vendored/generated code and possibly inline-test structures contribute; this is **not** 9.4% debt in the owned Rust production code. The Rust file inventory and symbol index are exhaustive for captured tracked/nonignored files; the clone window is not.

Tilth was useful for exact definitions, ranges and caller navigation. Ripwire added the global AST/symbol map, estimated complexity and normalized-body clones without running a language server. Together they were sufficient here; literal scoped rg searches helped confirm integration references. Their call resolution is weaker than compiler analysis, so important conclusions were traced in concrete callers rather than inferred solely from a missing edge. No rust-analyzer was started and no new tools were installed. Additional heavy indexing was unnecessary for this review.
