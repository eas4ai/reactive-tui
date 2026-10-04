# Claims versus implementation

“Contradicted” means the inspected frozen source conflicts with the stated behavior. “Unsupported” means the exposed abstraction or description has no corresponding inspected integration/proof. Neither term establishes intent, authorship, a falsified historical test run, or that every false statement was discovered. References point to detailed evidence and recommendations in the subsystem reports; examples were not executed.

| Claim / surface | Observed production implementation | Verdict / finding |
| --- | --- | --- |
| Component async changes through poll_change | Retained runtimes never call its wrapper; no initial future poll | Contradicted managed API behavior, C02 |
| Conditional first/last/odd/even/active/group CSS | Helpers apply base utilities without condition; App preserves these prefixes | Contradicted semantics, C05 |
| CSS-in-Rust Display::None | Builder is returned unchanged; no hidden state recorded | Contradicted behavior, C06 |
| Computed Memo updates with dependencies | Dependencies stored but never subscribed; private recompute used manually in a test | Contradicted public computed behavior, C07 |
| RuntimeContext effects respond to signals | Signal is not connected to runtime; effect body is consumed FnOnce | Contradicted reactive behavior, C08 |
| Compile-time CSS property/type validation | Arbitrary property identifiers share generic value trait; mismatches silently ignored | Unsupported validation guarantee, C13 |
| Compile-time perfect component map | Empty Lazy runtime HashMap with no production population path | Contradicted implementation/optimization description, C14 |
| Additional pin-safety verification | Identical adjacent pointer expressions compared; real pin invariant untested | Ornamental guard, C15; no UB alleged here |
| Lock guard dropped before callbacks | on_update runs before guard drop; reentrant reads deadlock | Contradicted for update callback, N01 |
| Auto-reverse and loop callbacks | Implemented in unused completion helpers; live completion duplicates a different path | Contradicted supported options, N05 |
| Cleanup detects stale/stuck animations | Liveness timestamp never refreshed by updates; active long animations removed | Contradicted cleanup behavior, N06 |
| Optimized batching preserves animated values | Opacity uses progress, transforms jump to destination, some variants return unknown | Contradicted optimization semantics, N09 |
| Interpolation cache validity | Compares enum kinds, not endpoint payloads | Contradicted cache correctness, N08 |
| Visibility/cache/SIMD/automatic debug support | Visibility always true; cache and callback wrappers empty; verbose branch empty | Incomplete advertised surfaces, N16 |
| Performance metrics | Real batch totals exist, but Default retains zero history and global metrics never updated | Partial implementation; avoid claiming all metrics are fabricated, N16 |
| Keep every other cache sample | Sorts then keeps first 50 | Contradicted comment, N16 |
| Complete reactive system for C | Legacy effect constructor boxes callback tuple; run is manual; hook effects storage unused | Contradicted automatic integration, N20; newer owned API excluded |
| PROPER IMPLEMENTATION in FFI widget file | C text input/checkbox are static generic elements; progress is percentage-only | Capability mismatch, N15; button callback limitation disclosed |
| Hit-grid registration/checkHit/dumpBuffers | Regions/IDs discarded; in-bounds returns fabricated ID1; dump only logs | Contradicted operation/docs, N17 |
| Single-operation Surface→Renderer→Terminal integration | Supplied terminal ignored; new renderer restores host on return; raw cell copy drops side data | Contradicted integration/ownership, N22 |
| Incremental syntax highlighting/warm cache reuse | Active-theme route reparses complete document before cache; editor isolated-line cache loses context | Contradicted incremental/context behavior, N12/N13; no timing measured |
| Zero-copy and O(1) gap-buffer operations | Temporary character copies; suffix copy/growth and line-index shifts/scans | Contradicted universal complexity/copy claims, N18 |
| 100 MB char limit | 100M Vec<char> elements need roughly400MB before copies; constructors bypass growth guard | Incorrect unit/memory description, N18 |
| Package/library version | Cargo1.0.0, native rtui_version0.1.0, README0.1.0 candidate | Inconsistent release surfaces, N19; ABI version separate |
| Finite chart values never stall worker | Finite subnormal step underflows, widening loop never progresses; shutdown joins | Contradicted CHT-026, W01 |
| Numeric scatter axis min/max are axis values | Limits filter point indices and do not pin numeric domain | Contradicted public axis semantics, W03 |
| Chart canvas bounds imply bounded drawing resources | Virtual slot counts allocate independently of capped cells | Capacity contract gap, W04; no measured OOM claimed |
| xterm indexed cube conversion | Uses0,51,102,153,204,255 instead of0,95,135,175,215,255 | Contradicted palette conversion, W05 |
| WCAG contrast ratio | Computes weights on encoded sRGB; existing linearizer unused here | Contradicted standards claim, T10; no internal production caller found |
| Diff with statistics | Separate implementation omits ordinary diff's resize clear | Contradicted equivalent output semantics, T16 |
| Lock-free event queue | Mutation needs &mut self; threaded owner locks it | Unsupported description/needless atomics, terminal observations; no safe-use UB claim |
| Cartesian path-check helper presence | Unused pass-through/sink functions suppress dead-code warnings | Nonbehavioral evidence, widgets observations; does not prove exercise of plotting contract |

## Claims deliberately not charged

- The modern bound animation target API has owner/generation checks and current-value capture. Old missing-target/default-value criticisms would be stale for that route.
- Typed keyframes perform actual offset/value validation and preserve typed values; not all animation APIs are placeholders.
- Animation GPU batching is explicitly labeled future work; that disclosure is accurate. The separate optional graphics CPU/GPU renderer is a real implementation.
- FFI raw-pointer trackers and properly unsafe entry points exist in some families; N02 identifies unsound safe exports, not absence of all ownership validation.
- Included AccessKit adapter provenance, disabled private simplified facade, supported screen-reader combinations and platform limits are disclosed. No false claim was established merely from vendoring or limited platform support.
- The crate's supported-API manual states that catalog-wide remediation/acceptance is still open and compilation is not runtime evidence. A public module export alone is not a promise that every alternate path drives App.
- T14 acknowledges process-stderr suppression in its source comment. It is an actionable integration tradeoff, not an undisclosed algorithm or proof of deception.
- Existing receipts, historical benchmark numbers and prior “tests passed” statements were not reproduced. This review does not label them lies without evidence.

The recurring problem is that source/API surface area is used as a substitute for connected behavior: fields, enum options, helper names and checks exist while the actual state transition or caller is absent. The fix is to complete or narrow those contracts and remove misleading descriptions, not to add more implementation-shaped scaffolding.
