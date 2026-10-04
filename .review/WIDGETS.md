# Widgets, builders, and theme static review

Input: frozen snapshot `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. Paths and line numbers below refer to that snapshot's original-relative paths, not the concurrently changing working tree.

This was a source review. No tests, builds, Clippy, benchmarks, rust-analyzer, Sudus commands, or production edits ran. Trigger sequences below are reasoned reproductions, not executed reproductions. All 163 owned production Rust files received a structural map/outline pass; the accompanying JSON distinguishes focused body tracing from structural inspection. Inline tests were read selectively for lifecycle guarantees, without execution.

Confirmed findings: W01–W08. P1 means a process-level failure or indefinite stall; P2 means incorrect supported behavior. Confidence is confidence in the source-level conclusion, not runtime verification.

## W01 — Finite subnormal chart data can hang tick generation and shutdown

**Severity:** P1. **Confidence:** high. **Type:** confirmed defect; contradicted contract.

**Location:** `src/widgets/display/charts/plot/tick.rs:71`, `:110`, `:122`.

**Trigger:** Render a populated ordinary bar/line chart with one value `f64::from_bits(1)`, default unpinned axes and the default five ticks. Alternatively call `ChartsBuilder::build()` with that data and a value tick-format closure.

**Proof and callers:** `ScaleLinear::domain_including_zero` retains the distinct finite range `(0, minimum_positive_subnormal)` (`plot/scale.rs:37`, `:43`); `value_domain_with` accepts its finite positive span and calls `nice_domain` (`plot/domain.rs:95`, `:100`). At `tick.rs:71`, dividing that span by four underflows to zero. The logarithm/power calculation produces magnitude zero and hence step zero. At `:111`, `0 / 0` produces NaN; the exit comparison at `:113` is forever false. `next_step(0)` also returns zero, so the unconditional loop makes no progress. Default tick count is five (`charts.rs:886`).

The production renderer calls this domain path at `live/canvas/cartesian.rs:278`, through the worker's `canvas::draw` at `live/worker.rs:154`. Worker shutdown joins the thread at `:130` and cancellation is only checked outside the draw operation at `:145`. Thus an affected worker consumes CPU indefinitely and unmount/shutdown cannot finish. The tick-format builder also calls the same path synchronously at `charts.rs:773` through `plot/domain.rs:131`, so that form can hang the application before mounting.

**Contract check:** `docs/spec/charts.md:160` (CHT-026) expressly says finite values must never panic or stall the worker; existing finite-value validation at `live/canvas.rs:348` accepts this input.

**Recommendation:** Require a finite strictly positive representable step, bound the widening loop and detect non-progress; use a representable fallback domain/step when division or decimal magnitude underflows. Ensure long draw work can observe cancellation.

## W02 — Theme aliases can recurse until the process exhausts its stack

**Severity:** P1. **Confidence:** high. **Type:** confirmed defect.

**Location:** `src/theme/mod.rs:130`–`:133`.

**Trigger:** Construct `ThemeVariables::new().set("--color-a", "b").set("--color-b", "a")`, attach them to a theme, and call `resolve_color("a")`. Activating such a theme and using either alias in a chart or utility class reaches the same failure.

**Proof and callers:** The public variable setter accepts arbitrary strings (`theme/variables.rs:30`). `defined` recursively calls `resolve_variable` for any nonliteral value differing from the *current token's spelling*. Only direct identical spelling is guarded. The two aliases alternate indefinitely; fallback never runs because recursive resolution never returns (`theme/mod.rs:128`, `:141`, `:151`). There is no visited set or depth bound. Chart color resolution calls the public resolver at `widgets/display/charts/live/canvas.rs:280`; theme utility application is public at `theme/mod.rs:160`.

**Consequence:** A small erroneous theme configuration can produce stack overflow and terminate the process during styling or chart validation.

**Recommendation:** Resolve alias chains iteratively with a set of normalized variable names and a bounded depth, returning an explicit cycle error or a controlled unresolved result. Check cycles across parent-theme lookups as well.

## W03 — Numeric scatter axis limits filter indices instead of x values

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect; contradicted public API semantics.

**Location:** `src/widgets/display/charts/live/canvas/cartesian.rs:286`, `:308`, `:807`, `:1223`.

**Trigger:** Give a scatter chart points `(x=10,y=1)` and `(x=20,y=2)`, and set `x_axis.min=Some(10)`, `x_axis.max=Some(20)`. Both points lie in that numeric range, but neither is drawn. Conversely limits 0–1 admit their indices despite both x values being outside the requested range.

**Proof and callers:** The renderer correctly detects numeric x and maps points by their x accessor (`:282`, `:1190`). However the x domain is computed only from data, then widened with `pinned=(false,false)` (`:294`–`:308`); the configured limits are ignored. The shared `index_visible` closure compares limits with `i as f64` (`:807`), and the numeric-scatter sampling path filters with it at `:1223`. Indices zero and one therefore fail a minimum of ten. Original-point anchors use the same index filter later in the function (`:1348`).

**Contract check:** Public `ChartAxis` documents min/max as the axis's minimum/maximum *value* (`charts.rs:850`, `:852`), and `ChartsBuilder::x_axis` forwards the axis unchanged (`:226`). CHT-033 describes a numeric linear x scale (`docs/spec/charts.md:178`). Validation accepts these finite limits (`live/canvas.rs:326`); there is no rejection or documented conversion to indices for numeric scatter.

**Recommendation:** Apply configured min/max to the numeric x domain and clip using each point's numeric x accessor. Keep index-based clipping restricted to genuinely categorical axes.

## W04 — Virtual chart slot counts amplify tiny input into unbounded allocation

**Severity:** P1. **Confidence:** high. **Type:** confirmed defect; allocation amplification.

**Location:** `src/widgets/display/charts/live/canvas/cartesian.rs:542`, `:555`, `:885`.

**Trigger:** Render an 80×24 bar chart with one finite datum and `.band_count(10_000_000)`. This tiny caller-owned data vector asks only for virtual empty slots, but the renderer creates millions of owned tick objects and strings. `.band_count(usize::MAX)` reaches allocation-capacity failure even with one series. Line charts expose the equivalent `point_count` path.

**Proof and callers:** The public setter stores an unrestricted `usize` (`charts.rs:630`). `cartesian` sets `count = max(requested_slots, actual_data_count)` (`:241`–`:246`), then allocates a `Vec<Option<String>>` for every slot at `:542`. `band_ticks` creates one owned tick and index string per slot (`plot/tick.rs:199`–`:210`) **before** the caller truncates those ticks to the single actual datum (`cartesian.rs:561`). Bars also allocate two `f64` vectors of length `count` at `:885` even when not stacked. This is not a caller supplying a huge data vector: a single integer multiplies memory and work by the requested empty-slot count.

Canvas drawing caps its text/mask height to one million cells at `live/canvas.rs:472`, but this does not bound these auxiliary allocations. Production `validate` checks values, axes, colors and radial/Sankey input, and imposes no slot-count budget (`:325`–`:423`). An 80×24 chart takes the axes path (`plot/layout.rs:97`), so the problematic allocations are reached. Worker execution is at `live/worker.rs:154`. Very large allocations can abort the process; a capacity panic kills the worker without publishing a response, leaving the current picture pending.

The same unchecked-budget family also exists for grid columns: `cartesian.rs:579` collects one position per requested column, and `:584` multiplies integer `k * plot_w` without a checked product.

**Contract check:** CHT-034 explicitly supports extra empty slots and chosen grid-column counts (`docs/spec/charts.md:184`). Their semantics do not require an allocation for every invisible empty slot; neither the public setters nor validation document a supported maximum.

**Recommendation:** Keep virtual geometry counts separate from actual allocations. Generate labels only for populated/visible data, size stack accumulators to actual data, and lazily or boundedly sample grid/tick positions at the output resolution. Reject unsupported allocation budgets explicitly and check count arithmetic.

## W05 — Indexed terminal and captured-image colors use the wrong cube levels

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect.

**Location:** `src/theme/ansi.rs:146`–`:148`.

**Trigger:** A child process in `TerminalWidget` writes `ESC[38;5;17mX`. Indexed color 17 is converted to RGB (0,0,51), whereas xterm's default 256-color cube gives (0,0,95). Captured chafa/viu output using that index follows the same wrong conversion.

**Proof and callers:** `ansi256_to_rgb` uses cube component index multiplied by 51. xterm's own [palette generator](https://raw.githubusercontent.com/ThomasDickey/xterm-snapshots/master/256colres.pl), lines 60–69, instead maps index zero to zero and nonzero indices to `index * 40 + 55`, yielding levels 0,95,135,175,215,255. This claim is about xterm's default cube, not configurable base ANSI colors 0–15.

The mounted terminal paint path calls this converter for `TerminalColor::Indexed` (`widgets/terminal/paint.rs:22`), then applies it to cell foreground/background (`:28`–`:29`). Captured-image cell colors call it too (`widgets/display/image/live/cells.rs:35`). The terminal lane confirmed the parser preserves indexed colors and leaves conversion to this widget path. The companion RGB conversion also mishandles the lower component threshold (`ansi.rs:27`–`:30`): for instance (60,0,0) becomes cube black rather than the nearer (95,0,0).

**Consequence:** Child terminal applications and captured image output visibly lose intended shades; this is a systematic palette error, not terminal negotiation or theme choice. The helper's “approximation” comment does not explain the unnecessary replacement of the known cube with a different one.

**Recommendation:** Use xterm's six canonical cube levels for index decoding and nearest-level thresholds for RGB encoding, keeping configurable base-color handling separate.

## W06 — One scrollbar can cause overflow without the other scrollbar appearing

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect.

**Location:** `src/widgets/layout/scroll_view.rs:219`–`:234`.

**Trigger:** With both directions enabled and scrollbars shown, put fixed 10-column by 20-row content in a 10×5 viewport. Vertical overflow enables the vertical bar. Its column shrinks the visible viewport to nine columns, so the tenth content column is clipped and horizontal scrolling is now needed. No horizontal bar appears. The symmetric case occurs when a horizontal bar induces vertical overflow.

**Proof and callers:** `bars` compares both content dimensions against the full box (`:224`–`:225`), independently of space consumed by the other bar. `visible_size` then subtracts that bar's column/row (`:233`–`:234`). `limits` consequently exposes a horizontal limit of one (`:243`), while rendering still gates the horizontal bar on the original false result (`:468`, `:478`). Content is unconstrained in the enabled horizontal direction (`:428`), so measuring it does not remove this mismatch.

**Contract check:** NAV-002/NAV-003 in `docs/spec/layout-widgets.md:54` and `:60` tie scrollbar space and pointer scrolling to overflow. The source comment explicitly chooses full-box comparison; the resulting smaller visible box can overflow in a direction for which no pointer track/thumb is provided.

**Recommendation:** Determine the two scrollbar requirements together against the resulting visible dimensions, using a bounded fixed-point calculation, and use that same result for viewport sizing, limits, painting, and pointer hit areas.

## W07 — Mixed integer and decimal table cells lose exact numeric ordering

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect; contradicted numeric-sort contract.

**Location:** `src/widgets/display/table.rs:727`–`:740`.

**Trigger:** Supply a sortable column containing, in this order, `"9007199254740993"` and `"9007199254740992.0"`, then sort it ascending. The second value is numerically smaller, but the comparator returns Equal and the stable sort preserves the descending input order. This is valid string-backed numeric data, with ordinary row IDs and column configuration; no unusually large vector or invalid geometry is needed.

**Proof and callers:** `Number::parse` retains the first value as an exact i128 and parses the decimal spelling as finite f64 (`:716`–`:724`). The integer/integer branch compares exactly (`:736`), but every mixed pair passes through `float`, which rounds the larger integer down to 9007199254740992 (`:729`, `:737`–`:740`). `numeric_column` accepts both strings (`:748`–`:760`); `validation_error` validates keys, widths and sort indices without forbidding mixed numeric cells (`:14`–`:49`). The mounted table builds its visible row order with `Table.sort_rows` (`table/live.rs:125`–`:126`), whose comparator calls `compare_cells` (`table.rs:385`–`:392`). The mounted data table uses the same comparator for each active sort (`data_table/live.rs:146`–`:173`, especially `:164`–`:166`). Neither caller supplies a precision-preserving secondary comparison.

Adding exact integer `"9007199254740992"` also shows inconsistent equality: the decimal compares Equal to both integers, while the integers compare unequal to each other. This violates the equivalence relation expected of a sorting comparator. No claim of a sort panic is made.

**Contract check and consequence:** DAT-003 requires columns of numbers to sort by their numbers (`docs/spec/data-widgets.md:69`), and `table.rs:707` explicitly promises exact integers past 2^53. Mixed decimal/integer columns can display the wrong order and data-table sort priorities can proceed to a later column even though the primary numeric values differ.

**Recommendation:** Compare mixed integer/float pairs without first rounding the integer to f64. Handle finite float sign, integer part, fractional part and i128 bounds explicitly, preserving a consistent total numeric order. Keep exact integer/integer comparison.

## W08 — Replacing a table or tree callback alone keeps calling the old callback

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect; ordinary prop-update behavior.

**Location:** `src/widgets/display/table.rs:194`–`:218`; equivalent omissions at `src/widgets/display/tree.rs:570`–`:598` and `src/widgets/display/data_table.rs:197`–`:211`.

**Trigger:** Mount a table with callback A in `on_select`. An application state change rerenders the same component identity/key with identical rows, columns, selection and display options, but supplies callback B capturing the new application state. Selecting a different row still calls A. Changing the callback to None similarly fails to disconnect it. Tree selection and data-table export callbacks have the same supported trigger.

**Proof and callers:** The public prop equality implementations omit callbacks, so callback-only updates compare equal. `ComponentRuntime::render_mounted` offers new props to the mounted instance (`src/component/runtime.rs:192`), but `ComponentInstanceWrapper::update_any` returns before cloning or storing them when `supplied_props == typed_props` (`src/component/instance.rs:273`–`:276`). `ComponentInstance::update_props` also stores props only on inequality (`:37`–`:49`). This is a retention decision, not merely suppression of extra paint work.

`Table.render` clones those retained props into `LiveTable` (`table.rs:616`–`:625`), and its child equality also omits callback identity (`table/live.rs:28`–`:38`). Selecting a row invokes the retained `props.on_select` (`table.rs:521`–`:523`). The tree's mounted selection invokes `props.on_select` (`tree/live.rs:386`–`:392`), and the data table's export toolbar clones `props.on_export` into its click closure (`data_table/live.rs:262`–`:266`) after `DataTable.render` forwards the retained config (`data_table.rs:261`–`:265`). No widget-specific bridge refreshes these callbacks. The core review lane independently traced the equal-prop retention behavior; the widget callback and caller paths were checked here.

**Consequence and qualification:** Interactions keep invoking stale captured application state or an action the application removed until a compared prop also changes or the component remounts. This can send a selection or export action to the wrong application destination. The finding requires callback-only replacement at a retained component identity; a coincident row/options change can mask it.

**Recommendation:** Include optional callback identity in equality, using the existing `display::overlay::same_callback` helper, as input and progress dialog options already do (`dialog/input.rs:73`–`:76`, `dialog/progress.rs:42`). Audit the live child prop wrappers too, including their internal callback fields. Alternatively refresh supplied callbacks independently of render-equivalence checks. The comments that callbacks “can't be compared” are misleading: closure behavior cannot be compared, but Arc identity can.

## Design risks and metrics — separate from confirmed defects

- **Full-text work on the App thread (static risk, no timing claim):** `src/widgets/input/text_input/paint.rs:98` scans the complete text and allocates a String per grapheme to build all rows. It runs from cursor scrolling (`:168`), mouse targeting (`:217`), wheel scrolling (`:49`) and rendering (`:244`). Normal key editing can perform multiple complete scans before a frame. This makes large input fields sensitive to input length despite a tiny viewport. Cache row/grapheme layout by text revision and width, and derive visible rows/cursor positions from that cache. No benchmark ran, and the threshold at which this becomes user-visible remains unknown.
- **Concentrated renderer complexity (measurement, not correctness proof):** The parent lane's static estimate places `cartesian` at 1,197 lines, estimated cyclomatic 218/cognitive 535; modal live rendering at 356 lines/estimated cyclomatic 56 and cognitive 102, and popover live rendering at 297 lines/estimated cyclomatic 57 and cognitive 100. Source tracing shows numeric/category domain, tick generation, slot allocation, grid, bars/candles/areas/scatter and hover share the cartesian function. Splitting along actual scale/chrome/type-rendering contracts would make the confirmed defects easier to constrain; the metric alone is not a finding.
- **Nonbehavioral source padding (observation, no intent inference):** `cartesian.rs:1399` is an unused pass-through `fit_label` wrapper and `:1406` an unused `_sink` function, explicitly retained for “path checks.” Both suppress dead-code diagnostics. They add no rendering behavior and are poor evidence that a plotting contract is exercised. Remove them or make contract checks follow actual calls. No claim is made about author intent.
- **Toast offset arithmetic (unverified reachability, not counted as a defect):** `dialog/engine/live.rs:120` adds saturated per-toast heights to an `i16` accumulator with ordinary `+=`. Enough retained toast height could overflow despite bounded dialog count. This deserves a focused review for supported large viewports/configurations; an end-to-end trigger was not established here.

## Boundaries checked and limits

Input editing/clipboard and grapheme paths, dialog HTTP cancellation/response handling, file explorer staged operations and platform-specific rename/symlink handling, menu event routing, table/tree reconciliation, modal/popover layout and timers, image decode/worker integration, terminal widget update/event/monitor/blink/paint, and chart worker/domain/tick/renderer paths were traced in focused bodies. Structural-only files were inspected for declarations, ownership, integration points and relevant callees rather than every statement. No claim of complete runtime validation is made.

Rejected hypotheses include terminal title-prop priority (the emulator is not initialized from config.title and OSC titles may legitimately take priority), raw key-release handling (the mounted runtime filters releases), and table drag indexing after column replacement (update cancels stale drag). No additional confirmed filesystem traversal or dialog HTTP lifecycle vulnerability was established. This is not proof that those boundaries are defect-free.

## Further production interaction pass

The additional pass deepened previously structural checkbox and named-radio updates/events, input popup placement, popup and dialog menus, their invocation/state helpers, wizard and confirmation dialogs, progress dialogs, input/autocomplete public dialog models, data-table filtering and numeric helpers, tree row painting, and relevant public table/tree props/builders. Coverage notes identify focused body areas; “deep” does not mean every legacy helper in a large file was reread. This pass added W07 and W08 and did not rerun previously reviewed chart/terminal/image paths.

No additional finding was established in checkbox focus/disabled handling, menu selection repair after item changes, popup authored-visibility reconciliation after local dismissal, wizard step-ID/validation/visited-state reconciliation, confirmation enabled-button and callback-veto handling, progress cancellation/prop replacement, filter parse-error retention and callback execution outside model locks, or tree paint target rebuilding and width/scroll clipping. The named-radio checked-prop hypothesis was rejected because its public builder expressly calls it the initial checked state (`src/builder/specialized.rs:274`). Input dialog option callbacks compare Arc identity, unlike the data widgets in W08. Legacy autocomplete HTTP placeholder methods were not promoted as missing mounted HTTP support: its `render` delegates to the separately reviewed live implementation (`dialog/autocomplete.rs:451`–`:458`). These are scoped negative results, not guarantees that all interactions are correct.
