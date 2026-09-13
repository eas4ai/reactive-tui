# Rust API remediation

Status: Agreed 2026-09-08
Prefix: API

The developer requested a commitment to remediate all findings in
`docs/api-audit.md`. This specification turns that audit into requirements.
The audit is diagnostic evidence, not an acceptance suite.

Existing working APIs and inherited contracts remain binding. Removing features,
retiring APIs, narrowing promised semantics, or declaring a finding out of scope
requires an explicit developer decision; documentation alone cannot repair broken
behavior. Routine implementation choices remain with the agent. Judged choices
are recorded before implementation. Platform claims require platform evidence;
unavailable verification is reported and escalated, never fabricated.

## Signal ownership

[API-001]
Every legacy and tagged native signal constructor/getter/setter/destructor MUST use matching allocation types and documented ownership. Preserve existing ABI; reject invalid family use where validation is supported without dereferencing an invalid allocation.
Audit mapping: RAPI-01.
Falsifier: a supported typed lifecycle miscasts or leaks an allocation, or consumer ownership becomes ambiguous.
Mechanism: typed Rust/C lifecycle probes with a suitable memory-safety checker.

## Component expansion and identity

[API-002]
App MUST recursively render registered components before layout and painting, retain one live instance per stable key, deliver prop changes, and unmount removed instances exactly once. Unknown component names MUST have defined observable behavior.
Audit mapping: RAPI-02.
Falsifier: a nested component fails to paint, keyed reorder loses state, props remain stale, or lifecycle callbacks duplicate.
Mechanism: real App/SuprTUI captured-frame and lifecycle tests.

## Hook state and memoization

[API-003]
Generated components MUST reset hook indexing for each render and retain existing slots without growth. Memo values MUST follow documented dependency semantics. Invalid hook ordering MUST have defined behavior.
Audit mapping: RAPI-03.
Falsifier: repeated renders allocate new state slots, state resets, or a changed dependency returns a stale memo.
Mechanism: repeated generated-component renders with state, slot-count and memo assertions.

## Effects context and runtime

[API-004]
App MUST own component effect/timer lifecycle. Effects MUST run and clean up at documented dependency changes and unmount. Removed components MUST leave no runnable timer or effect. Descendants MUST inherit context with scoped overrides and isolation between Apps.
Audit mapping: RAPI-03.
Falsifier: cleanup runs immediately instead of at its lifecycle boundary, work survives removal, parent context is absent, or Apps share component context.
Mechanism: mount/update/unmount tests with controlled scheduler and nested providers.

## Callbacks and event routing

[API-005]
Builder callbacks MUST be retained and invoked through App keyboard/mouse routing exactly once for the intended target. Event bounds and ordering MUST agree with the painted layout after resize and updates.
Audit mapping: RAPI-04.
Falsifier: a supplied handler is discarded, an event hits the wrong node, or resized bounds are stale.
Mechanism: App input-to-callback tests using captured layout and frames.

## Stable focus and traps

[API-006]
Focus identity MUST survive keyed redraw/reorder. Tab and reverse Tab MUST work with and without traps. Nested dialogs MUST autofocus, confine focus and restore it on close or removal.
Audit mapping: RAPI-04.
Falsifier: focus changes solely because of redraw, navigation stalls, or trapping/restoration targets a different node.
Mechanism: keyboard sequences through real rendered trees including nested and removed traps.

## Unicode editor positions

[API-007]
Both plain and syntax editors MUST consistently convert text positions and terminal display columns. Insertion, deletion, movement and selection MUST respect documented grapheme boundaries and multiline behavior for ASCII, CJK, emoji and combining text.
Audit mapping: RAPI-05.
Falsifier: mixed insert_char/insert_text operations corrupt cursor position, backspace leaves the inserted grapheme, or selection/display bounds split a grapheme.
Mechanism: editor operation and rendered-cell tests across Unicode and multiline cases.

## Bounded clipboard operations

[API-008]
Clipboard subprocess operations MUST have bounded deadlines, propagate nonzero exits and unavailable-backend errors, and reap owned children on timeout/cancellation. Claimed desktop backends MUST have explicit verification coverage.
Audit mapping: RAPI-06.
Falsifier: a stalled tool blocks indefinitely, a failed or unavailable copy reports success, or a child survives timeout.
Mechanism: isolated subprocess fixtures plus recorded integration checks for each claimed backend.

## Dynamic styling and text tokens

[API-009]
Focus/hover/disabled variants MUST depend on actual node state. Accepted typography and overflow tokens MUST change rendered output according to their documented terminal semantics.
Audit mapping: RAPI-07.
Falsifier: unfocused styling applies unconditionally, uppercase/truncation is accepted without effect, or a state change fails to repaint.
Mechanism: App state transitions and independent expected cell/text output.

## Gradient and animation painting

[API-010]
Supported gradient and animation properties MUST survive component expansion and layout and affect the supported painter. Terminal approximations MUST be explicit and tested.
Audit mapping: RAPI-07.
Falsifier: properties disappear in the bridge or configured changes never affect frames.
Mechanism: captured intermediate frames with gradients and scheduled property changes.

## Widget behavior and builders

[API-011]
An inventory MUST enumerate public widgets/builders and advertised controls. Existing advertised behavior MUST work through App, including input, checkbox/select, tables/trees, menus, scrolling, selection, callbacks and disabled/empty states. Builders MUST produce functional controls rather than descriptive placeholders.
Audit mapping: RAPI-08.
Falsifier: a catalog entry lacks behavioral coverage, a builder paints a description instead of its control, or interaction relies on fixed/default bounds.
Mechanism: per-widget acceptance matrix and App workflows at multiple viewport sizes.

Screen-reader guarantee (approved 2026-09-09, escalation api-011-api-018): verify
Orca with GNOME Terminal on Linux, including actual delivery of labels, roles,
focus and state changes. Preserve public label APIs. Other terminal/screen-reader
pairs are explicitly unverified; their existing rendering/input guarantees remain.
Retained metadata or painted text alone is not screen-reader acceptance evidence.

## Dialog lifecycle and results

[API-012]
Dialogs MUST paint, accept events, update and deliver completion/cancellation results synchronously or asynchronously as advertised. Stacking MUST honor z-order and limits. Closing MUST emit documented events, release resources and restore focus.
Audit mapping: RAPI-09.
Falsifier: a result is discarded, async mode has no completion, z-order is ignored, or close leaves focus/resources behind.
Mechanism: nested dialog workflows including limits, cancellation, result delivery and cleanup.

## Animation semantics and screens

[API-013]
Typed/untyped keyframes MUST preserve values and interpolate according to documented type/easing semantics. Relative values MUST use actual current properties. Active screens MUST receive input. Transition progress MUST visibly affect output.
Audit mapping: RAPI-10.
Falsifier: a numeric midpoint steps to the previous endpoint, conversion substitutes defaults, relative values start from zero incorrectly, or fade/input is ineffective.
Mechanism: deterministic clock tests plus intermediate rendered screen frames and input delivery.

## Image decoding and terminal output

[API-014]
Advertised file and encoded-memory image paths MUST decode actual image data and share correct placement, clipping and cleanup semantics. Claimed graphics protocols MUST emit valid output. Fallback MUST render the decoded image. URL loading support MUST be explicitly decided before implementation.
Audit mapping: RAPI-11.
Falsifier: a file yields a synthetic pattern, encoded bytes are mistaken for pixels, a claimed protocol only paints fallback cells, or removal leaves stale placement.
Mechanism: known-image pixel comparisons, protocol captures and integration evidence on each claimed host/protocol.

iTerm2 3.7 color/transparency limit (approved 2026-09-10, escalation
api-011-api-014-api-020): inline-image color accuracy and transparency are
unsupported on this host/version. Preserve the public inline-image APIs and
all other host requirements, including placement, update and removal. Retain
its failing color evidence. WezTerm remains the verified inline-protocol host
for color acceptance. This exception does not apply to other hosts or protocols.

Kitty host repair (approved 2026-09-12, escalation api-014-api-020): this
commitment includes the narrow repair that prepares image placements before
choosing Kitty's image-layer paint path. Build and verify an isolated, pinned
host with the existing image checks and an independent sender that exposes
the stale placement-count decision. Preserve the failing stock-host evidence.
The corrected host MUST show new placements without an extra repaint request
or retransmission and retain update and removal behavior. Record the build's
provenance and required host version/patch; do not claim that this updates
users' unmodified Kitty installations. All other API-014 requirements remain.

## Feature configurations

[API-015]
The no-default-features configuration MUST compile and have defined runtime behavior. Optional dependencies MUST be correctly gated or made mandatory by a recorded compatibility decision. The supported feature matrix MUST pass build and applicable behavior checks.
Audit mapping: RAPI-12.
Falsifier: no-default compilation references an absent optional dependency or a claimed feature combination fails.
Mechanism: locked feature-matrix builds and applicable behavior tests.

## Backend and terminal entry points

[API-016]
Public application/backend/terminal entry points MUST have a documented supported route, with functional adapters or an explicitly approved migration. Retained legacy App paths MUST initialize and update prior render state correctly without synthetic patches. Native App construction MUST expose the recovered rendering path.
Audit mapping: RAPI-13.
Falsifier: a retained entry point repeatedly takes first-render behavior, uses fake patches, fails updates/restoration, or has no functional supported route.
Mechanism: Rust and native entry-point workflows with update, error and terminal-restoration assertions.

## Complete native binding access

[API-017]
After native behavior works, editor/layout/dialog APIs and a stateful foreign component/event bridge MUST be exposed through audited C and TypeScript interfaces. Define callback lifetime, reentry, prop/state updates, errors and ownership; preserve existing ABI guarantees.
Audit mapping: RAPI-14.
Falsifier: a required family remains excluded or a consumer cannot paint, interact, observe state/results and clean up safely.
Mechanism: compiled C and TypeScript end-to-end consumers plus independent ABI/layout checks.

## Accurate public API documentation

[API-018]
Publish a complete supported-API matrix linked to behavior evidence and limits. Public Markdown APIs MUST be visible consistently during documentation builds. Documented examples MUST compile. Props validation MUST enforce specified constraints or have an explicitly approved narrower contract.
Audit mapping: RAPI-15.
Falsifier: readiness claims exceed evidence, docs hide public APIs, examples fail, or validation accepts every value despite promised constraints.
Mechanism: documentation/example builds, invalid/valid Props cases and evidence-linked inventory review.

Props contract (approved 2026-09-12, escalation api-018-api-020): keep
`#[derive(Props)]`, fluent builders and caller-invoked `validate() -> bool`.
`#[prop(validate = rule)]` names a predicate receiving a shared reference to its
field. Validation succeeds only if every declared rule succeeds; fields without
rules add no constraints. Construction and App mounting do not call validation.
Reject bare or malformed validation annotations and unsupported prop options.
`#[prop(optional)]` requires an explicitly written `Option<T>` and defaults to
`None`; the derive does not rewrite fields. Preserve documented defaults and
builders, and migrate existing bare validation annotations to named rules.

## Residual audit concerns

[API-019]
The audit coverage table concerns MUST be resolved or verified explicitly: gesture hooks, ui::Updater dispatch, theme propagation and multi-App isolation, Markdown/syntax integration and bounded large-input behavior, editor undo/selection claims, and claimed platform/legacy terminal behavior. Each MUST have an inventory entry with a concrete contract and falsifier before implementation.
Audit mapping: RAPI-03, RAPI-15 and coverage table.
Falsifier: a named concern disappears from the inventory or an advertised operation remains a marker/no-op without approved contract change.
Mechanism: focused behavior checks and final cross-reference review of the complete audit.

Unix async input ownership (approved 2026-09-12, escalation api-019-api-020):
`UnixTty::spawn_input_thread` and `DirectTty::start_async_events` return an owned
`platform::InputReceiver<T>` instead of the standard library's concrete receiver.
Dropping the receiver or final terminal owner MUST stop and join its input work,
including idle and full-queue waits. Queues MUST be bounded and readers MUST NOT
access closed or reused descriptors. Preserve ordinary receive methods and iteration
with standard receive errors. Explicit standard receiver/iterator types require
documented migration. This approval does not weaken other platform/input contracts.

Local reference scopes (approved 2026-09-13, escalation api-004-api-019-api-020):
`hooks::with_local_hooks` MUST own retained arbitrary non-Send local values for
related synchronous renders. `App::run` supplies a scope or reuses the caller's
active scope. Manual renders using local hooks require the wrapper. Hooks remain
Send + Sync and local handles remain non-Send. Local owners MUST reject missing,
different-thread, different-scope or expired-scope use. Same-thread cleanup releases
retained local shares; foreign-thread cleanup defers their destruction until the
creator next uses its local scope or exits it. Scope exit, including unwinding,
MUST release retained shares; escaped LocalRef handles retain ordinary ownership.
App cleanup MUST finish inside its scope and MUST NOT close another owner's scope.

App performance ownership (approved 2026-09-13, escalation
api-004-api-016-api-019-api-020): each App MUST expose its own PerformanceContext
handle and provide it to component hooks with scoped overrides. Mode requests
MUST wake and affect only that App, coalesce with bounded storage, and become
inert after App exit. Snapshots MUST report the selected mode and completed-frame
timing. Keep the public context fields and ordinary hook call forms. Legacy
global context and mode functions serve standalone callers only; Apps MUST NOT
publish into or consume that state. External global callers must migrate to the
App handle or component hooks. Optional context changes MUST preserve hook slots.

## Regression and closure

[API-020]
Every inherited requirement MUST retain current passing evidence. Final review MUST reconcile every audit finding and named subcase with implementation, tests and any explicit developer-approved contract change. Mechanisms MUST demonstrate safe violating and corrected cases. Historical defect-confirming probes MUST not be counted as acceptance passes.
Audit mapping: RAPI-01 through RAPI-15.
Falsifier: an inherited check regresses, an audit item is silently deferred, coverage is weakened, or a defect-confirming assertion is counted as correctness.
Mechanism: all inherited mechanisms and independent final scope/behavior review.

## Approved Updater migration

Approved 2026-09-13, escalation api-019-api-020-2: ui::Updater MUST require
an update method. Existing empty implementations must implement that method.
Each App owns its registered updaters. Request handles wake only their App,
coalesce repeated pending requests, and dispatch in registration order before
rendering on the App thread. Requests during callbacks run on a later turn.
Removed registrations and closed Apps make old handles inert. Callback errors
propagate through App cleanup. Preserve bounded pending storage per registration.

## Approved gesture coordinate units

Approved 2026-09-13, escalation api-019-api-020-3: preserve both Position
variants and hook signatures. Gesture distances and thresholds MUST use cells
for Cell input and pixels for Pixel input. Velocity MUST use the corresponding
units per second. A threshold of 5 means five cells or five pixels. Restart
a gesture when its coordinate units change; never subtract cells from pixels.
