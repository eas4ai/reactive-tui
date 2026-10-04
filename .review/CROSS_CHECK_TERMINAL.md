# Independent static cross-check — terminal reviewer

Source: `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. References are original-relative paths and frozen-source line numbers. I independently inspected the production source behind W01, W02, W04, C01, C04 and C05 after reading the other reviewers' reports. All six survive this cross-check. No production files, other reports or Sudus state were changed. No tests, builds, benchmarks, checkers, lint gates or production code execution ran; all reproductions below are reasoned source traces.

| Finding | Result | Valid input and reachable caller | Neutralization checked |
| --- | --- | --- | --- |
| W01 | Confirmed; high confidence | A single finite positive minimum-subnormal value with the default unpinned five-tick axis | Finite validation, include-zero domain normalization, tick-step progression and worker shutdown |
| W02 | Confirmed; high confidence | Public theme variables aliasing a to b and b to a; resolving a | Literal-first resolution, fallback, setter validation and direct-self guard |
| W04 | Confirmed; high confidence | Tiny populated chart with a large public band/point slot count | Canvas cell limit, validation, visible-label truncation and worker ownership |
| C01 | Confirmed; high confidence; retain public/Screen scope | Replacing an automatic render tree or Screen content with the same registered component key | Component clone identity, tracked cleanup, registry replacement and App's distinct resolved conversion |
| C04 | Confirmed; high confidence | Two ordinary mouse Move events, first inside a retained component and then outside it | App target resolution, processor routing, explicit Leave handler and unregister cleanup |
| C05 | Confirmed; high confidence | App classes containing group-hover or first while those conditions are false | App state filtering, optimizer fallback and variant helper context |

## W01 — Finite subnormal chart data can prevent drawing and shutdown

`src/widgets/display/charts/plot/tick.rs:71` divides the positive span by the interval count without guarding the result against underflow. For `f64::from_bits(1)` and four intervals, `raw` becomes zero. `log10(0)` yields negative infinity and the power-of-ten magnitude becomes zero, so tick_step returns zero. In nice_domain's unpinned loop (`:110`), the zero lower end divided by the zero step produces NaN; its end comparison never succeeds. next_step at `:81` also returns zero, so the loop cannot make progress.

This is valid production input: chart point validation at `src/widgets/display/charts/live/canvas.rs:348` accepts finite values; `plot/scale.rs:26` includes zero while retaining the finite positive upper end; the unequal domain does not enter the equal-domain padding branch. `plot/domain.rs:100` invokes nice_domain, and `src/widgets/display/charts.rs:886` defaults to five ticks. `live/canvas/cartesian.rs:278` requests this automatic value domain for ordinary unpinned charts. There is no existing positive-span padding that neutralizes the trigger.

The worker draws outside its cancellation check (`live/worker.rs:154`), while its Drop joins the drawing thread (`:130`). Closing or replacing a chart already executing the loop can therefore wait indefinitely. The finite-data correctness and shutdown consequence in W01 are supported. This is mathematical reasoning from IEEE floating-point operations, not an executed hang demonstration; no claim is made about scheduler timing before the job starts.

## W02 — Theme color alias cycles have no terminating ownership or resolution rule

`src/theme/variables.rs:30` accepts arbitrary string values. A theme containing `--color-a = b` and `--color-b = a` is constructible through that public setter. `Theme::resolve_color("a")` enters resolve_variable at `src/theme/mod.rs:141`, then defined at `:128`. Non-literal `b` recursively resolves at `:132`, which then finds non-literal `a` and repeats. The comparison `value != token` blocks only direct self-reference, not a two-node cycle. Literal parsing (`src/layout/colors.rs:365`) contains no alias-cycle logic; fallback runs only after defined returns and cannot terminate the recursion.

Direct public resolution is already sufficient reach. Chart color resolution at `src/widgets/display/charts/live/canvas.rs:280` and themed CSS application at `src/theme/mod.rs:160` additionally expose it in rendering. The report's stack-exhaustion mechanism is supported. Exact stack size and process-level failure behavior depend on the runtime/platform and were not exercised. Recommendation remains cycle detection with a visited set or explicit bounded resolution depth and a failed-resolution result.

## W04 — A bounded canvas does not bound slot-count allocations

`src/widgets/display/charts/live/canvas/cartesian.rs:246` derives band count from the maximum of requested slots and actual data. The public builder stores band_count without a maximum (`src/widgets/display/charts.rs:630`), and production validation at `live/canvas.rs:325` does not reject excessive slot counts. A tiny finite dataset in an ordinary 80×24 chart passes validation and has axes (`plot/layout.rs:97`).

The derived count allocates the full label vector (`live/canvas/cartesian.rs:542`) and full band-tick collection (`:555`) before visible-label truncation (`:561`). Stack position/negative vectors at `:885` also scale with the count before looping over actual data. The canvas's one-million-cell height bound (`live/canvas.rs:472`) does not limit these allocations. Grid-column positions at `live/canvas/cartesian.rs:579` similarly iterate the requested column count, with unchecked multiplication at `:584`.

A count such as ten million causes allocation and formatting work unrelated to the tiny viewport; a maximum usize can reach capacity/overflow failure before completing the frame. These public values have no documented constructor precondition or validation exclusion in the inspected paths. The worker owns the operation but does not catch draw panics (`live/worker.rs:154`); ownership therefore provides no recovery guarantee. W04 is supported as a serious resource-bound defect. Exact out-of-memory behavior is environment-dependent; no allocation, panic or memory benchmark was executed.

## C01 — Old automatic tree cleanup can remove a newer registry owner

The public converter at `src/render/tree.rs:436` explicitly enables instantiation and local keys. Its component path registers a clone at `:539` and retains the original instance in the node. AnyComponentInstance::clone at `src/component/instance.rs:242` calls the factory to create a fresh instance; it does not share the node's component ownership. Registry insertion at `src/component/registry.rs:269` replaces the entry by key. ElementNode::drop at `src/render/tree.rs:228` then unregisters solely by the node's key, without comparing a generation or owner.

For a registered component factory, Screen::set_content at `src/screen/mod.rs:112` first creates the replacement automatic tree, then assigns it at `:129`. Dropping the previous same-key tree removes the just-inserted replacement registry entry. TrackedComponentInstance cleanup exists (`src/component/tracked_instance.rs:53`); it calls Unmount when the instance is ultimately disposed. It prevents a simplistic permanent-registration-leak claim, but it does not repair owner identity. With no externally retained tracked reference, the removal also immediately disposes the replacement instance. External tracked references can delay destruction, but cannot keep the registry entry present.

App's resolved converter at `src/render/tree.rs:431` is an actual neutralizing path and C01 correctly excludes it. ScreenRuntime separately resolves its own actual component (`src/screen/runtime.rs:59`), so this finding should retain the report's precise scope: public automatic conversion and Screen bookkeeping produce extra lifecycle work and incorrect global registry ownership; it is not proof that App's normal retained component is destroyed. No lifecycle-counter execution was performed.

## C04 — A retained mouse-position signal stays inside after a raw Move changes owners

Normal App dispatch resolves the component under the pointer and feeds the raw event to ComponentRuntime before EventRouter (`src/app.rs:341`). ComponentRuntime routes no-hit motion with owner None (`src/component/runtime.rs:381`) and routed hits with the current component owner (`:397`). MouseEventProcessor routes Move directly to handle_mouse_move (`src/hooks/processor.rs:310`). That handler sets the current owner's position is_inside flag true (`:415`) but the previous-owner transition clears only hover state (`:433`). Leaving every hit target likewise does not clear the old position signal.

Explicit Leave events do clear position at `src/hooks/processor.rs:690`; normal router-synthesized Enter/Leave callbacks are not sent back into this processor. Unregister at `:238` removes the position state, which is irrelevant to the report's trigger because the old component stays mounted. No shared position ownership state clears the prior signal during the inspected raw Move transition. C04's two-Move reproduction and impact are supported without requiring a hover hook. No mouse replay was executed.

## C05 — Several CSS condition names have no condition evaluation

App's filter at `src/app/event_tree.rs:106` understands focus, focus-within, hover, disabled and viewport prefixes. Its default match arm at `:120` breaks without discarding the remaining prefixed token. The CSS optimizer delegates such tokens to variants (`src/layout/css/optimizer.rs:246`). Group variants strip the group prefix (`src/layout/css/variants.rs:86`) and eventually invoke an unconditional base utility (`:197`). The active/visited/first/last/odd/even/group helpers at `:167` onward all lack state, ancestor or sibling arguments and simply invoke that same utility path.

Thus `bg-black group-hover:bg-red-500` applies the later red style even without an active group hover, and `first:p-8` on each child applies padding to each parsed child. Existing App filtering does neutralize supported hover/focus/disabled prefixes and should not be implicated; it does not neutralize the prefixes named by C05. The helper existence assertions do not prove conditions were evaluated. C05 is supported as incorrect runtime style resolution and an unsupported conditional-feature claim. No rendered screenshot or assertion execution occurred.

## Additional root-owned spring sanity checks

The parent requested source-only independent validation of two late animation candidates. Both are supported; the parent owns their final findings and severity.

**Initial velocity sign:** `src/animation/spring.rs:100` uses positive configured velocity in the underdamped remaining-displacement response, then calculate_position subtracts that response at `:86`. Differentiating at zero from the right gives negative configured velocity. The critical (`:116`) and overdamped (`:133`) coefficients have the same mismatch. Public calculate_velocity returns positive configured velocity for zero time (`:142`), contradicting the right-hand derivative as well as the documented initial-velocity property (`:19`). Nonzero finite velocity with displacement larger than precision is a valid public with_velocity input (`:50`); no wrapper flips its sign in the inspected spring-to-easing path.

**Normalized time versus physical spring time and completion:** `SpringConfig::new(1, 1, 2)` is accepted at `src/animation/spring.rs:39` and has exact critical damping. The public easing function advertises normalized time (`src/animation/easing.rs:176`), but its Spring arm passes that value directly to calculate_position (`:196`). Animation::spring forwards arbitrary duration and the easing unchanged (`src/animation/mod.rs:292`). Update samples normalized progress at `:459` and stores the eased interpolation at `:477`; at raw progress one, the no-loop branch marks Completed (`:494`) without snapping the value to its endpoint. For a forward 0→1 interpolation with this configuration, the stored completion fraction is `1 - 2/e`, approximately 0.2642, including when the configured duration is ten seconds. Estimate-duration and settled APIs exist, but this update path does not consult them. No source comment in the inspected path gives these discrepancies an intentional endpoint or initial-velocity contract. These are algebraic source checks, not executed animation samples.

## Inspection limits

This cross-check focused on the six named findings and two requested spring candidates. It is not expanded structural coverage of their reviewers' entire scopes. It checked valid-input construction, relevant production callers, cleanup and state transitions; selective test assertions were considered only as source. No timing, allocation size, terminal rendering, stack-exhaustion or shutdown scenario was executed. No corrections to the six cross-checked findings were necessary; the scope limits for C01 and environment-dependent consequences for W02/W04 remain material.

## Follow-up cross-check — C16–C19 and W07–W08

The parent requested six additional production-source checks. All six are supported with high confidence in the mechanisms below. Severity remains owned by the original reviewers. This follow-up is also exclusively static: no production execution, test, build, checker, benchmark or Sudus action occurred. The report update is append-only.

| Finding | Result | Boundary or masking case |
| --- | --- | --- |
| C16 | Confirmed: explicit CellGrid colors omit the current node's opacity | Zero opacity is correctly skipped; ancestor opacity is applied; fractional current-node opacity is the defect |
| C17 | Confirmed: paused task expires and resume cannot restore it | Requires a runtime delivery after the original deadline while still paused; earlier resume instead consumes pause time |
| C18 | Confirmed: spring impulse velocity is not an input to position evolution | Getter briefly exposes the stored impulse, which does not make the later frame use it |
| C19 | Confirmed: hook animation loop configuration does not repeat the scheduled interpolation | Ordinary non-hook Animation has separate loop logic and is outside this finding |
| W07 | Confirmed: mixed integer/float comparisons lose exact integer order | Pure integer/integer pairs retain the correct exact comparison |
| W08 | Confirmed: callback-only props updates retain old closures | A coincident compared-prop change or remount can refresh them and mask the defect |

### C16 — Explicit CellGrid colors bypass the node's own fractional opacity

`src/layout/paint_tree/suprtui.rs:775` stores the ancestor placement's opacity in PaintNode.parent_opacity. It is not the product including the current node: that product is passed only to children at `:828`. The explicit packed-color fast path in paint_cells decides full opacity using parent_opacity alone (`:1261`); its color helper applies only that same ancestor value (`:1266`). Explicit backgrounds use the helper or direct opaque assignment (`:1313`), and explicit foregrounds use the helper (`:1319`).

A valid 1×1 CellGrid with an opaque explicit red foreground or background, attached to a node with opacity 0.5 and fully opaque ancestors, therefore emits that explicit color at full opacity. CellGrid's public setter packs the supplied colors directly (`src/layout/paint_tree/cells.rs:119`, `:137`); no current-node style exists in the grid to pre-attenuate them. The early gate at suprtui.rs:1243 correctly suppresses opacity zero but does not fix fractional values. Current opacity can reach the next node as an ancestor value, so moving the opacity to a wrapper could mask the bug; that does not repair styling the grid node itself. This supports C16 specifically for explicit grid colors, without claiming an equivalent failure in all text/background painting paths. No pixels or terminal cells were rendered.

### C17 — Paused hook animations can lose their task before resume

The task's start_time is captured once when inserted (`src/hooks/animation.rs:183`). Runtime delivery derives progress from wall-clock elapsed time regardless of the controller's pause state (`:149`). The callback exits early when the controller is not playing (`:292`), but deliver still removes the task when its externally computed progress reaches one (`:154`). Public pause sets the controller paused (`:353`); resume sets it playing (`:361`) and neither adjusts the task clock nor inserts a replacement task.

Valid trigger: start a one-second hook animation, pause before completion, allow the normal App runtime delivery to occur after the original one-second deadline while paused, then resume. `src/app.rs:661` feeds this runtime during frame preparation. The callback never advances the value during the paused delivery, but the registry removes its only task. Resume then advertises Playing with no remaining callback to change the value. Component liveness/generation checks protect teardown; they do not recreate the removed task for a live owner. Even resuming before removal consumes paused wall time and can jump progress. The stronger permanent-stall trigger requires the paused deadline delivery explicitly; no timing scenario was executed.

### C18 — Spring impulses update a reported velocity that the solver ignores

Public apply_impulse adds the finite force to SpringOwner.velocity and starts a new task (`src/hooks/animation.rs:509`). The frame callback reads current value, target and SpringConfig, then calculates position and velocity from that config (`:538`). It never reads SpringOwner.velocity as a solver input, and overwrites it with the calculated velocity (`:547`). The config's separate velocity field is unchanged by apply_impulse.

For a live spring moving from zero toward one with default configured velocity, applying an impulse of 100 produces the same next-frame position calculation as applying an impulse of one: both invoke calculate_position(0.016, current, target) with identical config. With an initial target equal to the current value and zero config velocity, the public impulse cannot initiate displacement either. The getter at `:504` can briefly return the incremented field, which does not neutralize the absent evolution input. Cancellation and fresh generation prevent obsolete callbacks; they do not copy the impulse into the solver. This supports the no-op-motion defect, separately from the parent's spring sign/time findings. No frame sampling was executed.

### C19 — Hook loop configuration stops after its first deadline

Public AnimationConfig includes loop_count and loop_behavior (`src/hooks/animation.rs:410`), and use_animation forwards them into the stored Animation controller (`:426`). The independently scheduled hook task contains one start_time and duration (`:183`), not loop state. Its update closure calls the easing interpolation directly, checks controller.is_playing, and on first terminal progress stops the controller (`:305`). Runtime delivery removes the task on the same progress (`:155`). Neither path calls the controller Animation's normal update loop machinery to advance loops or schedule another interpolation.

Using a live hook with LoopMode::Infinite, or an ordinary positive repeat count and its count loop mode, therefore finishes this hook's scheduled interpolation once despite the supplied configuration. Calling animate_to again is a new user request, not automatic looping. The non-hook Animation update path has separate looping branches and should remain outside C19's scope. Component-owned cleanup is valid but does not provide repetition. No loop playback was executed.

### W07 — Mixed numeric sorting does not preserve exact integer order

`src/widgets/display/table.rs:716` parses integer spelling into i128 and decimal spelling into a finite f64. The comparator preserves integer/integer comparison at `:736`, but every mixed pair converts the integer to f64 (`:729`, `:737`). Valid numeric strings `9007199254740993` and `9007199254740992.0` thus compare Equal: the first rounds down to the second's exactly representable value. The public table validation accepts these cell strings, and numeric_column accepts both (`:748`). sort_rows invokes this comparator through its stable sort (`:385`, `:392`); the mounted data-table query also uses it (`src/widgets/display/data_table/live.rs:164`). An ascending sort of the larger integer followed by the smaller decimal can retain that incorrect order.

Adding integer spelling `9007199254740992` also demonstrates inconsistent comparator equality: each integer compares Equal to the decimal, while the two integers compare unequal to one another. Existing exact integer parsing therefore neutralizes pure integer cases, not mixed columns. No caller supplies a precision-preserving secondary comparison before the table decision. The report correctly avoids claiming an executed sort panic. This cross-check is exact-value floating-point reasoning, not a sorting run.

### W08 — Equality rejects callback-only prop changes before storing them

TableProps declares its public selection/action callbacks at `src/widgets/display/table.rs:93` but omits them from equality at `:194`. TreeProps callbacks at `src/widgets/display/tree.rs:510` are omitted from equality at `:570`. DataTableProps callbacks at `src/widgets/display/data_table.rs:176` are omitted from equality at `:197`, including callback omissions inherited from table_props.

The normal retained component path offers replacement props (`src/component/runtime.rs:192`). AnyComponentInstance::update_shared delegates a new props Arc to update_any (`src/component/instance.rs:191`), but ComponentInstanceWrapper::update_any returns before copying equal props (`:273`). Updating the supplied Arc cache does not update the wrapped component's stored callback. Table.render then clones those old props into its live child (`src/widgets/display/table.rs:618`), whose own LiveProps equality also omits callbacks (`table/live.rs:28`). The actual table selection code invokes props.on_select (`table.rs:521`), tree selection invokes it (`tree/live.rs:391`), and the data-table export toolbar clones props.on_export into the click closure (`data_table/live.rs:262`). These are reachable callback consumers, not merely unused callback fields.

Replacing callback A with callback B or None at the same retained key and otherwise equal props therefore keeps A. External state stored in a shared object and read by A might hide stale captures, but the public API also accepts closures with different captured destinations. Remounting or changing a compared option can force copying; the finding explicitly requires callback-only replacement. No widget interaction or closure execution occurred. W08 is supported for all three public prop types and has a second equality barrier in live table props.
