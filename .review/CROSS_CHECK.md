# Independent cross-check of selected native/animation/text findings

Reviewed N01, N02, N04, N05, N10, N12, N14 and N15 in `NATIVE_ANIMATION_TEXT.md` against frozen source `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. This was source navigation only with Ripwire maps and Tilth production ranges. No tests, builds, checkers, runtime experiments or Sudus actions ran. Only this report was written; other reports and source were not altered. The earlier CORE report self-audit was already complete.

“Accept” means inspected source supports the stated defect and caller reach. “Qualify” preserves the underlying finding but narrows a claim or identifies a necessary precondition. No selected finding is rejected in full. All reproductions remain reasoned, not executed.

| Finding | Disposition | Essential result |
| --- | --- | --- |
| N01 | Accept | Callback reentry on the same animation state lock is directly reachable through App-managed animations. |
| N02 | Accept | The named safe Rust exports require pointer validity that their signatures cannot impose. |
| N04 | Accept | Parallel timelines mistake a delayed child's lack of a sample for completion. |
| N05 | Accept | The live loop branch does not invoke on_loop or apply auto_reverse. |
| N10 | Qualify | Packed-state lost updates and Count(257) failure are real; this is an exported alternate API, not App's current state engine. |
| N12 | Accept | SyntaxEditor's separate painting cache reparses isolated lines and discards the whole-document results. |
| N14 | Qualify | Wide-text placement is wrong; joined/combining corruption also comes from Surface::write_str itself. |
| N15 | Qualify | Input and checkbox are static representations; avoid claiming every callback-free button/progress representation independently breaches a documented native-control contract. |

## N01 — Accept

`src/animation/mod.rs:419` acquires state.write(). The same guard remains in scope while `callback(self, values)` runs at :489. get_state/get_progress/get_current_values take state.read() at :598/:606/:614. Register an on_update callback that calls any one of them, play, then update a frame: the callback tries to obtain a read lock while its own thread owns the write lock. There is no guard release between those operations. The comment/drop at :522-523 happens afterward and cannot neutralize this path.

For completion, :529-533 holds a read lock across callback(self); writing `animation.state`, which is public at :239, can likewise wait on the callback's own read guard. The lock-release recommendation must cover both callback sites.

App reach is established rather than assumed: `src/app.rs:858` exposes the actual manager mutably, `src/app.rs:260` drives its update in the event loop, and `src/animation/mod.rs:1066-1067` calls each animation's update. App owns the manager directly; no additional manager mutex is needed for the defect. The separate hook-animation callback machinery should not be described as identically affected.

## N02 — Accept

`src/ffi/reactive.rs:538` is `pub extern "C" fn`, not unsafe, and dereferences the supplied signal pointer at :544 after checking only null. `rtui_signal_string_get` at :459 is also safe and both dereferences the signal at :469 and writes through the caller-provided output buffer at :478. The explicitly unsafe neighboring string-set and integer-get APIs at :492/:521 show that this is inconsistent coverage, not a blanket claim that no exports carry safety obligations.

Safe Rust may construct an arbitrary non-null raw pointer with an integer cast and call these safe functions. No external manual or opaque-handle convention establishes the validity of that memory for a safe Rust signature. A successful null check does not establish alignment, allocation, lifetime or aliasing, and catch_panic does not make a dangling reference valid. These named functions are enough to support the high-severity Rust-surface finding even without relying on the broader pointer-tracker concurrency allegation.

**Qualification for broader prose:** retain family-specific evidence for any synchronized-ownership claim. Tracking in other functions should not be treated as nonexistent, and a caller violating a documented unsafe C memory obligation is a different issue from this safe Rust signature defect. No crash/undefined-behavior example was executed.

## N04 — Accept

Timeline::play starts every parallel child at `src/animation/mod.rs:948-950`. Animation::try_play records start_time at :343. Animation::update at :430-433 returns false while the actual monotonic time since start remains below config.delay. Parallel Timeline::update counts only true update returns at :988 and completes when none are true at :993-995. Future timeline calls return before touching children once state is Completed (:965-967).

Use one delayed child, play the parallel timeline, and call update before wall-clock delay expiry. The timeline completes while that child's state is still Playing and its progress has not advanced. This passes through the manager at :1075-1077 as well as the direct public timeline API. There is no `is_completed` check in the parallel branch; the sequential branch does have one at :974.

**Wording adjustment:** the child has already been *started* by play; it will not *advance through the completed timeline*. Delay is tested against elapsed monotonic wall time, not merely the delta_time supplied to update. The reported extra sequential call follows from Animation::update returning true on its final sample (:538), although that secondary observation is lower impact than premature parallel completion.

## N05 — Accept

The live completion branch at `src/animation/mod.rs:493-519` directly resets time/count/direction. Infinite and Count do not inspect config.auto_reverse and none of these branches invokes callbacks.on_loop. PingPong toggles direction at :515 but also omits on_loop.

The only production on_loop invocation sites in this module are the unused completion/restart helpers at :570/:588. auto_reverse's operative check is at :584 inside restart_animation, also marked allow(dead_code). The public builder exposes auto_reverse and on_loop at :799 and :856. No caller found through static navigation connects the helpers to update; direct inspection of update confirms the gap.

Use an unbound builder animation with Count(2) or Infinite, auto_reverse(true), and an on_loop callback. Advance through a full duration repeatedly. The live branch restarts without toggling direction for those modes and without invoking the callback. Target binding does not introduce another loop dispatcher. The duplicated helper path should be consolidated only while preserving callback lock release; invoking it under the current write guard would compound N01.

## N10 — Qualify

The packed-word race is concrete: at `src/animation/lock_free.rs:90` thread A loads old packed state; thread B's increment_loops succeeds through CAS at :121 (or toggles/pauses through :143/:169); thread A stores its edited stale word at :92, erasing B's unrelated bits. Acquire/Release ordering does not turn the two operations into one read-modify-write. The API takes shared `&self` and contains atomics, so safe concurrent calls are possible; no inspected precondition restricts setters to one writer.

The loop-count defect does not require concurrency. increment_loops caps its packed 8-bit value at 255 (:113). For Count(257), update compares `loops < count - 1` at :257, so 255 remains below 256 indefinitely and the completion branch cannot run. Count(0) uses unchecked subtraction, with panic/wrapping behavior depending on overflow checking. These are representable public inputs, not malformed casts.

The updater at :225 tests only is_completed before advancing time; it does not check stopped/paused. get_state at :79-84 has no Reversed mapping, although AnimationState includes Reversed (`src/animation/state.rs:54`). Those supplementary API inconsistencies are also source-supported.

**Reach qualification:** static usages of LockFreeAnimationUpdater show an alternate exported module, not App integration. App uses the ordinary AnimationManager/RwLock state. State that boundary in the final summary. Lost updates require a concurrent interleaving; Count(257), stopped/paused advancement and decoding issues can be reasoned in one thread. No thread race or loop was executed.

## N12 — Accept

SyntaxEditor::with_language calls rehighlight_visible at `src/editor/syntax_editor.rs:74`. That method passes the complete document into highlight_lines at :188 and discards the return value. highlight_lines reparses the full document and stores context-aware lines in SyntaxHighlighter.cached_lines (`src/syntax/highlighter.rs:154-160`).

SyntaxEditor's painting path reads a distinct LineCache at `src/editor/syntax_editor.rs:228`. On a miss it calls rehighlight_line at :231, which creates a new Highlighter and supplies only the single line (`src/syntax/highlighter.rs:225-226`). That function does not first read the complete-document cached line; it overwrites the highlighter's own cached entry at :241 with the isolated result. get_styled_lines then paints the isolated runs.

With a valid Rust language and an active theme, put an otherwise valid Rust statement on a middle line of a multiline block comment, then call get_styled_lines. The syntax context lost from the opener changes how that statement is classified. Parser context cannot be recovered from the separate editor cache's content hash. Whole-document parse work occurring earlier therefore does not neutralize this finding. A plain fallback caused by an unavailable language/theme would hide the visual difference, so retain those reproduction preconditions. No highlighted-color comparison was run.

## N14 — Qualify

Both FFI loops write one char_code at a time at `src/ffi/text.rs:368` and :452, then advance current_x by one at :377/:461. Surface::write_str uses internal width-two advancement at `src/core/surface.rs:1076-1089`. Thus the second call for `界A` is positioned at x+1, inside the cell span reserved for the first wide character, instead of x+2. The same source mismatch exists in both Surface and Renderer FFI routes. The constructor does ignore _width_method at `src/ffi/text.rs:84`.

**Responsibility qualification:** Surface::write_str itself iterates `.chars()` at `src/core/surface.rs:1062` and advances every scalar by at least one. Combining sequences and ZWJ clusters are therefore mishandled by that scalar writer even if FFI were changed to call write_str once with a whole string. Do not imply that merely combining the scalar calls fixes joined Unicode. The native editor's replacement model is correctly grapheme-based: `src/editor/painting.rs:146-158` iterates graphemes, calls set_grapheme and advances grapheme.width().

Keep N14's confirmed Unicode-placement defect and common-grapheme recommendation. Exact terminal appearance remains unexecuted; clipping and selection indices need to preserve the text-buffer contract when consolidating the loops.

## N15 — Qualify

The main structural observation survives a caller check. FFI text input starts with ElementBuilder::new(Layout(Flex)) (`src/ffi/widgets.rs:32`). That constructor sets input=None (`src/builder/core.rs:130`). `.placeholder` only changes native input props if input is already Some (:230-231), otherwise it adds a class marker. Build creates typed TextInput only when input is Some (:310-314). The actual `input()` helper sets this field (:77-81); the FFI constructor does not call it. Neither the initial text nor the class activates an editor/controller. The normal FFI ElementRoot returns its retained Element clone (`src/ffi/app.rs:139-140`), and ComponentRuntime resolves by element/factory metadata, not by a CSS class named input.

Checkbox similarly formats literal `[x]` text at `src/ffi/widgets.rs:81-90`; no state, focus or toggle handler is attached. Progress formats a one-time percentage at :122-137. Button attaches classes/text at :167-173 and no activation callbacks/focus metadata. They are static representations on this path.

**Contract qualification:** the module expressly says “without callbacks” (:10), and the ordinary generic Rust button helper is itself decorated Flex (`src/builder/core.rs:69-73`). A missing custom button callback is therefore not independent evidence of a broken promised callback API. A progress indicator also need not be interactive merely to be a valid control. Keep the stronger input/checkbox edit/toggle gap and the accurate statement that all these FFI functions return static representations, but avoid asserting an undisclosed “all constructors create native retained controllers” promise based only on names or the PROPER IMPLEMENTATION header. A medium capability gap is defensible when tied to documented editable/toggleable widget expectations; otherwise label it a limitation with inaccurate presentation, rather than a fabricated hard contract.

## Corrections to carry forward

No selected finding was fully rejected. Add explicit alternate-API reach to N10; clarify actual-time delay and already-started child wording in N04; attribute scalar Unicode limitations to the Surface writer as well as FFI in N14; narrow N15's contract claim, especially buttons/progress. N01/N02/N05/N12 need no substantive retraction. CORE C13 remains the source-backed analysis of the CSS macro's compile-time-validation claim; this cross-check found the same unsupported assertion at `src/builder/core.rs:147`.
