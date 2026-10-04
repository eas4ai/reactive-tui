# Static cross-check: widgets and selected native/text findings

Input: `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. Source traces were reconsidered against production bodies and public callers. No tests, builds, Clippy, benchmarks, rust-analyzer, Sudus actions, executable reproductions, or analysis/checker gates ran. This cross-check does not establish measured crash frequency or timing.

The requested widget metric correction was applied only to the relevant WIDGETS.md paragraph: modal renderer estimated cyclomatic **56** / cognitive **102**, popover renderer estimated cyclomatic **57** / cognitive **100**. Findings and other source/reports were left unchanged.

## W01–W06

| ID | Verdict | Severity/confidence | Caller reach and qualification |
| --- | --- | --- | --- |
| W01 | Retain | P1 / high | Finite minimum subnormal survives the positive-span domain validation; division by four produces zero step. NaN comparison prevents exit and next_step(0) cannot progress. Ordinary chart worker and tick-format builder both reach the path. |
| W02 | Retain | P1 / high | Public ThemeVariables accepts the two-alias cycle; parse_color_literal fails and recursive resolve_variable alternates indefinitely. Chart validation and utility styling can trigger it after theme activation. Requires a cyclic configured alias, rather than a default preset. |
| W03 | Retain | P2 / high | Numeric scatter builds x_domain from data, ignoring ChartAxis min/max, while samples and original anchors filter their indices using those limits. Public x_axis forwards the finite configuration unchanged. The 10–20 x-range / indices 0–1 example is sound. |
| W04 | Retain, resource qualification | P1 / high | One actual datum plus a virtual count causes count-sized labels, owned tick strings and stack arrays. Canvas cell limits do not cap this work. A capacity panic terminates the chart worker; process termination depends on allocation/OOM behavior. Very large caller-supplied virtual count is required. |
| W05 | Retain | P2 / high | Terminal and captured-image painting actually convert Indexed via ansi256_to_rgb. The cube is wrong relative to xterm's default palette; the child PTY advertises TERM=xterm-256color by default. Base colors 0–15 and custom terminal palettes are distinct from this cube defect. |
| W06 | Retain | P2 / high | Both bars are decided against full_size, then visible_size loses their row/column. Fixed 10×20 content in a 10×5 viewport retains a horizontal scroll limit of one but no horizontal track/thumb after the vertical bar appears. Keyboard/wheel access is not necessarily lost. |

W01's route is `src/widgets/display/charts/live/worker.rs:154` → `live/canvas/cartesian.rs:278` → `plot/domain.rs:100` → `plot/tick.rs:106` and `:110`. The initial reveal's sampled values do not rescue it: the non-transition domain uses props' own data. Shutdown joins the stalled worker at `live/worker.rs:130`. The synchronous builder path is `charts.rs:773` → `plot/domain.rs:131`. This supports the process-level stall severity.

W02's exact cycle is `"--color-a"="b"`, `"--color-b"="a"`: `src/theme/mod.rs:129` finds a value, `:132` recurses, and `:141` starts the next lookup. A normalized self-alias alone does **not** establish the same infinite loop: the identical-spelling guard eventually stops that single-node case. The final WIDGETS.md report correctly uses the two-alias case.

W04 should continue to distinguish process OOM from the smaller failure of a capacity panic in the isolated worker. The one-datum / 10,000,000 virtual-band case is allocation amplification, not merely a huge caller-owned Vec: `cartesian.rs:542` creates slot labels; `:555` calls count-wide tick generation at `plot/tick.rs:200`; only afterward does `cartesian.rs:561` truncate ticks to data_count. `:885`–`:886` create count-sized numeric arrays even without stacking. `live/canvas.rs:472` bounds canvas cells, not those allocations. P1 is justified for the potentially process-wide resource failure, but no assertion that every such count aborts every host is warranted.

W05's parent contract is stronger than a generic approximation helper: `src/terminal/pty/unix.rs:70` and `src/terminal/pty/windows/native.rs:394` set the default child TERM to xterm-256color. Application env can override that TERM, so this is default behavior rather than an unconditional environment promise. The known cube levels follow xterm's [palette generator](https://raw.githubusercontent.com/ThomasDickey/xterm-snapshots/master/256colres.pl), lines 60–69. The report appropriately confines the claim to the default 216-color cube.

## N07, N11, N13, N16, N18 and N19

| ID | Verdict | Recommended qualification/correction |
| --- | --- | --- |
| N07 | Retain, medium/high | It is a conditional same-millisecond collision, not a measured rate. Legacy/new construction plus manager insertion is public and can be used through App's ordinary AnimationManager. Modern auto-generated IDs are unique within their own atomic sequence, but share the same namespace as old IDs. |
| N11 | Retain, medium/high | Public grid helper definitely overflows at (182,0); the ordinary position-based calculate_delays path has the same arithmetic. No automatic App grid invocation needs to be claimed. |
| N13 | Retain redundant parsing; guards remain a design risk | “Always reparses” needs the condition that an active theme exists and the syntax resource lock succeeds. This is confirmed repeated full-document work/contradicted incremental comment, not a measured slowdown. Checked-method headers expressly limit their quota promises. |
| N16 | Retain, scope narrowly | Recent-history and global metrics are broken; batch total/average/peak metrics are real. Cache/SIMD claims overstate Aggressive behavior, but visibility/GPU placeholders are candidly identified in bodies/docs. The optimized manager is an alternate public API, not App's manager. |
| N18 | Retain, low/high | Correct left temporary-copy reference to gap_buffer.rs:131, not :134. The zero-copy and universal O(1) claims are contradicted, but no universal slow-edit claim follows. The 100 MB statement is an incorrect char-storage unit, not an allocation measurement. |
| N19 | Retain, low/high | README's pre-release statement begins at :35, not :36. Package/FFI release versions differ; ABI version is independent. No binary ABI incompatibility or released version's existence follows. |

### N07 — IDs and actual manager reach

`src/animation/mod.rs:264` formats elapsed milliseconds from a process-wide monotonic timer (`:124`–`:128`), not a uniqueness counter. `AnimationManager::add_animation` inserts into a HashMap with that ID and does not reject replacement (`:1037`–`:1040`). Same-millisecond construction therefore replaces the earlier animation when both are added.

`src/app.rs:119` owns an ordinary AnimationManager; `:858` exposes it to callers. The App uses that manager in its loop and initializes it at `:1073`. Thus this is a public supported registration failure, with no requirement that App itself manufacture IDs.

The modern API's `src/animation/api.rs:264`–`:267` uses a per-process atomic counter, but its string also has the form `anim_<number>`. Consequently modern and legacy IDs can still coincide if those construction routes are mixed in the same manager. Do not phrase the modern route as providing global cross-route uniqueness.

### N11 — Arithmetic and additional caller path

`src/animation/stagger.rs:163` computes both squares and their sum in i16 before casting to f32. At x=182 and origin (0,0), 182²=33,124 exceeds i16::MAX. With checked overflow it panics there. With wrapping arithmetic, the signed result is negative and its square root is NaN; `Duration::from_secs_f32` rejects that value at `:164`–`:165`. Configured dimensions 200×1 and the origin are representable; the function has no validator stopping this example.

The scope can be strengthened without relying only on the grid helper: `calculate_delays` dispatches a Position origin with nonempty positions to `calculate_position_delays` at `:79`–`:83`. That helper repeats the same i16 distance arithmetic at `:286`. One position `[(182,0)]`, origin Position(0,0), and default positive delay reaches the same panic. This is still public API reach, not proof App automatically computes staggers.

### N13 — What is proven

With an active syntax theme, `src/syntax/highlighter.rs:154` creates a new Highlighter and `:155` calls highlight(text) before the cached-range lookup at `:168`. A warm cache cannot skip the parsing operation. The no-theme and poisoned-resource-lock branches return a fallback at `:143`–`:149` and are exceptions to “always.”

`SyntaxEditor::rehighlight_visible` materializes full text and discards the returned styled lines (`src/editor/syntax_editor.rs:183`–`:188`). After edits clear its separate painting cache (`:136`, `:144`), `get_styled_lines` calls `rehighlight_line` at `:231`. Hence the redundant full parse followed by per-line painting parse is real. No measured performance threshold is established.

Quota distinction in the root report is appropriate: `MAX_SYNTAX_BYTES` is expressly for the checked highlighter (`highlighter.rs:16`), and enforced at `:83`; the alternate highlighting APIs lack that check. Likewise `MAX_MARKDOWN_BYTES` is documented for checked rendering (`src/markdown/renderer.rs:5`), checked at `:75`, while `render_with_sourcepos` parses without it at `:97`. This is inconsistent resource protection at alternate public boundaries, not a violation of an express “all APIs” limit.

### N16 — Separate real accounting from disconnected features

`src/animation/performance.rs:105`–`:108` actually records every batch's elapsed update time and animation count. `:469`–`:472` update totals and peak, and those totals feed `avg_time_per_animation` at `:482`. Calling all metrics fabricated would be a false positive.

However derived Default leaves max_history=0 (`:448`, `:459`, `:465`), and each history insertion removes its only item (`:475`–`:477`), so recent_avg_performance consistently returns None. `OptimizedAnimationManager::update_all` updates batches and individual animations but never records global_metrics (`:589`–`:611`); the getter exposes the untouched metrics at `:615`.

Aggressive processing delegates to basic processing (`:180`), retains every update through an unconditional true helper (`:198`–`:205`) and calls an empty cache method (`:209`). The public Aggressive enum doc promises caching/SIMD (`:24`), whereas the visibility helper explicitly labels its limitation and GPU is explicitly future (`:26`, `:100`). Preserve those qualifications. App's manager is AnimationManager (`src/app.rs:119`), not OptimizedAnimationManager.

### N18 — Complexity and memory units

The header's zero-copy/O(1) wording is unqualified (`src/editor/gap_buffer.rs:1`–`:4`). Gap moves allocate temporary Vec<char> slices at `:131` and `:155`; growth copies a suffix at `:194`. Even when the gap already sits at the edit, inserting near the beginning of a many-line buffer updates subsequent line-break entries at `:224`–`:225`; deletion scans the line-break vector at `:260` and `:261`. That directly disproves universal constant-time edits. Local edits in suitable cases can still be fast/amortized.

`:176` defines 100,000,000 elements and labels it a 100 MB char limit; storage uses Vec<char> (`:31`), so element storage alone is approximately 400 MB in decimal units. `with_capacity` and `from_string` do not apply that growth-only bound (`:47`, `:60`). Source accounting proves the unit mismatch; no memory measurement ran.

### N19 — Version meaning

`Cargo.toml:3` says 1.0.0; `README.md:35` says a pre-release 0.1.0 candidate; `src/ffi/mod.rs:185`–`:187` report release tuple 0.1.0. The function is documented as the library version at `:181`, rather than a C-wrapper release number. Its ABI field is separately documented at `:160` and remains 1 at `:188`. This supports a version inconsistency and consumer-reporting defect, not an ABI-layout-break claim.

## Result

No W01–W06 finding needs withdrawal. N07/N11 remain concrete defects; N13/N16 need their source-work and alternate-API qualifications kept visible; N18/N19 are appropriately low-severity documentation/accounting issues. Minor exact-line corrections and the renderer-metric correction are identified above. No production edits or runtime validation were performed.
