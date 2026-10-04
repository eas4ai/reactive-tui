# Native API, animation and text review

Frozen-source static audit; no tests, builds, Clippy, benchmarks or rust-analyzer ran. Every trace below is reasoned from source, not an executed reproduction. Confidence is high in the stated source behavior; public alternate paths are labeled where they differ from App. No deceptive intent is inferred.

## Findings index

| ID | Severity | Kind | Finding |
| --- | --- | --- | --- |
| N01 | high | confirmed defect | Animation callbacks deadlock on ordinary state reads |
| N02 | high | confirmed Rust API unsoundness | Safe Rust FFI exports dereference caller-controlled pointers |
| N03 | medium | confirmed defect | Reversing a playing animation stops its updates |
| N04 | medium | confirmed defect | A parallel timeline completes while its animations are waiting for delay |
| N05 | medium | confirmed defect / contradicted claim | Loop callbacks and auto-reverse are stranded in unused completion helpers |
| N06 | medium | confirmed defect | Stale-animation cleanup deletes animations that are actively progressing |
| N07 | medium | confirmed defect | Automatically generated animation IDs collide within a millisecond |
| N08 | medium | confirmed defect | Interpolation cache treats different endpoint values as the same animation |
| N09 | medium | confirmed defect | Optimized batching changes animation values instead of preserving semantics |
| N10 | medium | confirmed defect | Lock-free animation state loses updates and cannot finish larger loop counts |
| N11 | medium | confirmed defect | Grid stagger distance overflows for ordinary terminal-sized grids |
| N12 | medium | confirmed defect / contradicted claim | SyntaxEditor discards document context when painting highlighted lines |
| N13 | medium | confirmed performance issue / design risk | Incremental highlighting reparses the entire document and has inconsistent size guards |
| N14 | medium | confirmed defect | C text-buffer rendering corrupts wide and joined Unicode text |
| N15 | medium | contradicted capability / implementation gap | FFI widget constructors return decorated static elements, not native controls |
| N16 | medium | contradicted claim / incomplete implementation | Animation performance features and metrics are partly decorative |
| N17 | medium | contradicted claim / incomplete implementation | FFI hit grid and debug API expose successful no-ops and fabricated hit IDs |
| N18 | low | contradicted documentation / maintenance smell | Gap-buffer zero-copy and constant-time claims omit its actual work |
| N19 | low | contradicted documentation / version inconsistency | Package, README and native version disagree |

## N01 — Animation callbacks deadlock on ordinary state reads

**Severity:** high. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/mod.rs:419](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:419).

Animation::update retains its state.write() guard through on_update(self, values) at line 489. get_progress/get_state/get_current_values take state.read() at 598, 606 and 614. A callback such as on_update(|animation, _| { let _ = animation.get_progress(); }) therefore waits for a lock held by its own calling thread. App drives this update path. The comment about dropping the guard before callbacks at 522 only applies after on_update has already run. on_complete also runs with a read guard at 529–533; a callback writing the public state lock can hang.

**Recommendation:** Copy the sample and state needed by callbacks, release every guard, then invoke callbacks. Confirm callback reentry without holding either animation or manager locks.

## N02 — Safe Rust FFI exports dereference caller-controlled pointers

**Severity:** high. **Confidence:** high. **Type:** confirmed Rust API unsoundness.

**Anchor:** [src/ffi/reactive.rs:538](/home/shawn/workspace2/reactive-tui/src/ffi/reactive.rs:538).

rtui_signal_int_set is a public safe extern C function. It casts and dereferences the supplied handle at 544 after only a null check. Safe Rust can pass 0x1000usize as *mut RTuiSignal without an unsafe block. rtui_signal_string_get is also safe and writes through a caller-controlled output buffer at 478. Animation-manager update/destroy and many text/stats functions have the same problem. catch_unwind handles panics, not invalid memory access or Rust undefined behavior. The C manual correctly requires live handles; that caller obligation does not make a safe Rust signature sound. Typed trackers in some families also release the lookup lock before access and do not establish synchronized ownership.

**Recommendation:** Make functions with caller-side memory obligations unsafe on the Rust surface and document those obligations, or introduce safe owned/validated handles with liveness and synchronization covering the complete access. Do not present pointer plausibility as memory safety.

## N03 — Reversing a playing animation stops its updates

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/mod.rs:391](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:391).

reverse flips is_reversed and changes Playing to Reversed at 395. update rejects every state except Playing at 425. is_playing explicitly returns true for Reversed at 621–625. Sequence: play, update a partial frame, reverse, repeatedly update. It remains reported as playing while progress stops. This affects the exported Animation API regardless of whether the target is bound to App.

**Recommendation:** Keep playback state and direction orthogonal, or accept Reversed in update and apply its direction consistently.

## N04 — A parallel timeline completes while its animations are waiting for delay

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/mod.rs:985](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:985).

Animation::update returns false during configured delay at 429–434. The parallel timeline interprets any false update as not playing; if every child is delayed, any_playing remains false and the timeline changes to Completed at 993–995. Sequence: parallel timeline with one delayed child, play, update before its delay expires. The child has already been put into Playing by play, but the completed timeline never advances it after its delay. This trigger requires the first update to occur before the real monotonic delay expires. The method also returns true on a final sample although its documentation says false when complete; sequential advancement consequently takes an extra call.

**Recommendation:** Represent pending, playing, paused and completed explicitly; use completion state rather than the presence of a new sample to finish a timeline.

## N05 — Loop callbacks and auto-reverse are stranded in unused completion helpers

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect / contradicted claim.

**Anchor:** [src/animation/mod.rs:493](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:493).

The live update completion branch at 493–519 advances loops directly. on_loop and config.auto_reverse are used only by handle_animation_complete/restart_animation at 543–590, both marked allow(dead_code), which the live update never calls. AnimationBuilder exposes on_loop at 856 and auto_reverse at 799. A finite or infinite loop can complete iterations without calling its registered loop callback; setting auto_reverse does not alternate those loops. PingPong separately flips direction and does not fix the other loop modes.

**Recommendation:** Use one completion/loop transition implementation and invoke loop callbacks after releasing locks. Remove the dead duplicate transition path.

## N06 — Stale-animation cleanup deletes animations that are actively progressing

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/mod.rs:1116](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:1116).

cleanup_all_stale tests start_time and runtime.last_frame_time at 1125–1131. last_frame_time is assigned in try_play at 342 and cleared in stop at 379; update never refreshes it. After the threshold has elapsed, a continuously updated long or infinite animation meets the stale predicate and is removed. cleanup_completed also claims to remove failed/stuck animations while only testing completion and poisoned timeline locks.

**Recommendation:** Record a successful update timestamp or use a real liveness/progress marker. Distinguish completed, paused and stalled states in both behavior and documentation.

## N07 — Automatically generated animation IDs collide within a millisecond

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/mod.rs:264](/home/shawn/workspace2/reactive-tui/src/animation/mod.rs:264).

Animation::new assigns anim_<elapsed milliseconds>. AnimationManager::add_animation uses HashMap::insert at 1039. Two new animations created in the same millisecond get the same ID; adding the second silently replaces the first, including its callbacks/target. This is a concrete conditional collision trace, not a measured frequency claim. Explicit unique builder IDs and the modern animate route are separate routes.

**Recommendation:** Use a per-process atomic sequence or another unique ID source and decide explicitly whether manager insertion may replace an existing ID.

## N08 — Interpolation cache treats different endpoint values as the same animation

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/performance.rs:304](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:304).

is_cache_valid compares only enum discriminants for from/to, easing and age. It does not compare payload values. Query key k with Opacity(0)→Opacity(1), linear, progress .5; then reuse k with Opacity(.6)→Opacity(1), same easing/progress. The cached .5 is accepted although the correct new value is .8. store_in_cache also retains old endpoint metadata when an existing key is reused, rather than replacing an invalid entry. This public cache is presently disconnected from AnimationBatch because apply_cached_interpolation is a placeholder.

**Recommendation:** Use a key or validity comparison that includes complete endpoints/easing; replace invalid entries rather than accumulating samples under old metadata.

## N09 — Optimized batching changes animation values instead of preserving semantics

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/performance.rs:114](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:114).

Basic opacity batching returns normalized progress at 125 rather than the interpolated opacity. An opacity animation .4→.8 at progress .5 yields .5 instead of .6. Transform batching returns the destination immediately (TranslateX etc. at 143–146), and valid variants including ScaleX/ScaleY fall into unknown with 0 at 147. None mode returns get_current_values, so changing only optimization level changes the meaning/result of the animation. These are public alternate APIs; App does not automatically use this manager.

**Recommendation:** Batch the same sampled AnimatedValue semantics as the unoptimized route. Handle every advertised transform variant or return an explicit unsupported result.

## N10 — Lock-free animation state loses updates and cannot finish larger loop counts

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/lock_free.rs:89](/home/shawn/workspace2/reactive-tui/src/animation/lock_free.rs:89).

These are exported alternate APIs, with no App integration established. The loop-count defect is single-threaded; lost packed-word updates require concurrent callers. set_state reads the packed word, edits its low bits, and performs a plain store. A concurrent increment_loops, toggle_reversed or set_paused can CAS a newer word between that load/store and have its successful update overwritten. In addition, increment_loops saturates the 8-bit count at 255 (113) while update compares against a u32 Count at 257: Count(257) can never reach the completion branch. Count(0) subtracts 1 unchecked. update does not inspect paused/stopped flags and time advancement is a load-plus-store, so this atomically stored state is not a consistent multi-writer animation machine. Reversed is not decoded by get_state.

**Recommendation:** Define the concurrency contract and use atomic read-modify-write for packed transitions. Widen the loop count or reject unsupported counts; honor playback state. The type being lock-free is not proof its state transitions are correct.

## N11 — Grid stagger distance overflows for ordinary terminal-sized grids

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/animation/stagger.rs:163](/home/shawn/workspace2/reactive-tui/src/animation/stagger.rs:163).

With grid Some((200,1)), origin Position(0,0), and calculate_grid_delays(200,1), x=182 squares 182 as i16. 33,124 exceeds i16::MAX: debug builds panic, while wrapping arithmetic yields a negative value whose sqrt becomes NaN and Duration::from_secs_f32 panics. The public inputs are representable and no grid validator rejects this case. The ordinary non-grid Position-origin path repeats the i16 square at stagger.rs:286; a position (182,0) suffices there too. The same module directly converts overshooting/negative easing samples and unchecked range results to Duration.

**Recommendation:** Widen coordinates before subtraction/squaring, use hypot in a suitable float domain, and validate/clamp duration results. Share the duplicated direction/easing/range post-processing for linear and grid delays.

## N12 — SyntaxEditor discards document context when painting highlighted lines

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect / contradicted claim.

**Anchor:** [src/editor/syntax_editor.rs:182](/home/shawn/workspace2/reactive-tui/src/editor/syntax_editor.rs:182).

rehighlight_visible runs highlighter.highlight_lines on the complete buffer and ignores its returned styled lines. get_styled_lines later populates a separate cache via highlighter.rehighlight_line at 231; that method creates a new Highlighter and parses only the individual line at highlighter.rs:225–226. A line inside a Rust multiline block comment or multiline string is therefore painted using isolated-line syntax, despite whole-document parsing already having classified it. The two caches do not share context.

**Recommendation:** Paint from the whole-document cached output or maintain a persistent parser and context-aware incremental cache; use one authority for highlighted lines.

## N13 — Incremental highlighting reparses the entire document and has inconsistent size guards

**Severity:** medium. **Confidence:** high. **Type:** confirmed performance issue / design risk.

**Anchor:** [src/syntax/highlighter.rs:121](/home/shawn/workspace2/reactive-tui/src/syntax/highlighter.rs:121).

When syntax highlighting has an active theme, highlight_lines constructs a new Highlighter and calls highlight(text) at 154–156 before looking at cached output. Plain-text/no-theme and failed-lock branches are exceptions. Its warm cache does not skip the parse, contrary to the comment at 152–153 and the incremental API name/manual wording. SyntaxEditor edits call rehighlight_visible and then its painting cache parses visible lines again. MAX_SYNTAX_BYTES is checked only in try_highlight_text: highlight_lines and rehighlight_line bypass it, including the SyntaxEditor path. Markdown render_with_sourcepos similarly bypasses MAX_MARKDOWN_BYTES. The headers describe limits for checked methods, so the alternate-method quota gap is a design risk, not proof that every API promised that limit. No benchmark or memory measurement ran.

**Recommendation:** Either implement persistent incremental parsing or document full reparsing accurately; avoid the unused full-document pass. Apply explicit configurable limits consistently at public processing boundaries.

## N14 — C text-buffer rendering corrupts wide and joined Unicode text

**Severity:** medium. **Confidence:** high. **Type:** confirmed defect.

**Anchor:** [src/ffi/text.rs:306](/home/shawn/workspace2/reactive-tui/src/ffi/text.rs:306).

renderTextBufferToSurface and renderTextBufferToRenderer iterate Unicode scalars and call Surface::write_str on each scalar, then increment x by exactly 1 at 377 and 461. Surface::write_str advances internally by terminal width (surface.rs:1058–1092), but the next separate call resets x from the scalar counter. For 界A the A is written into the wide glyph trailing cell; combining characters and ZWJ emoji are split into separate calls. The constructor ignores width_method at 84. Surface::write_str itself is scalar-based, so combining/joined-text breakage is inherited as well as caused by splitting each scalar into a separate call; simply making one full write_str call is not a complete repair. The native editor already has a shared complete-grapheme painting path; this is a duplicated, weaker implementation.

**Recommendation:** Render complete graphemes with a common width/continuation policy, preserving scalar selection indices separately. Repair both copied rendering loops or consolidate them.

## N15 — C text-input and checkbox constructors create static representations

**Severity:** medium. **Confidence:** high. **Type:** capability mismatch / missing native control integration.

**Anchor:** [src/ffi/widgets.rs:22](/home/shawn/workspace2/reactive-tui/src/ffi/widgets.rs:22).

The file labels itself PROPER IMPLEMENTATION. `rtui_text_input_create` builds a generic Flex element with placeholder/text, and `rtui_checkbox_create` formats `[x]` text. Neither selects a native input/checkbox controller. `ElementBuilder::new` leaves input=None (`builder/core.rs:130`); only `.input()` activates native TextInput (`:77–81`), `.placeholder()` does not (`:229`), and build converts only Some(input) (`:310`). A C text input created this way does not accept editing through App's native input runtime; the checkbox does not maintain native toggle state. The no-callback disclaimer is narrower than the input's missing built-in editing behavior.

The progress-bar constructor similarly emits a percentage string, without a native bar. A progress indicator need not be interactive, so no retained-controller contract is inferred for it. The button constructor's decorated Flex matches the ordinary Rust button builder and expressly disclaims callbacks; that detail is not counted as an independent button defect.

**Recommendation:** Connect the input/checkbox constructors to native supported controls, or name/document them explicitly as static representations. Describe the percentage-only indicator accurately and retain the button callback limitation.

## N16 — Animation performance features and metrics are partly decorative

**Severity:** medium. **Confidence:** high. **Type:** contradicted claim / incomplete implementation.

**Anchor:** [src/animation/performance.rs:198](/home/shawn/workspace2/reactive-tui/src/animation/performance.rs:198).

The module advertises batching, interpolation caching, visibility optimization and SIMD. is_update_visible always returns true (198–205); apply_cached_interpolation does nothing (209–211); GPU delegates to basic batching (99–102, correctly marked future). OptimizedAnimationManager never records global_metrics yet exposes it at 615. PerformanceMetrics derives Default, leaving max_history=0; record_batch_update immediately removes every history item, so recent_avg_performance always returns None. The comment Keep every other sample at 392 actually sorts and truncates to the first 50. DebugAnimationManager adds the same pattern: callback wrapping is an empty method (`debug.rs:901`), measured frame time is unused at 884, and the verbose snapshot branch at 890–897 is empty. Explicit manual debugger logging remains available; automatic wrapping/profiling is absent. These are exact source contradictions; no speedup/slowness magnitude was measured.

**Recommendation:** Remove unsupported options/claims or implement them. Initialize useful history capacity, update global metrics, and keep comments tied to behavior.

## N17 — FFI hit grid and debug API expose successful no-ops and fabricated hit IDs

**Severity:** medium. **Confidence:** high. **Type:** contradicted claim / incomplete implementation.

**Anchor:** [src/ffi/stats.rs:120](/home/shawn/workspace2/reactive-tui/src/ffi/stats.rs:120).

addToHitGrid discards every region and ID parameter (120–139). checkHit only checks renderer bounds and returns 1 at 157–159, so registering ID 42 over a small region cannot produce ID 42 and points outside that region still report 1. setRenderOffset only logs, updateStats ignores supplied timing values, updateMemoryStats ignores all supplied memory values, and dumpBuffers logs metrics instead of writing the file its doc comment promises. Some body comments acknowledge limitations, but the exported names/docs remain behavior-shaped facades.

**Recommendation:** Implement the advertised operations or explicitly return/report unsupported behavior; avoid false hits and distinguish logging from buffer dumps.

## N18 — Gap-buffer zero-copy and constant-time claims omit its actual work

**Severity:** low. **Confidence:** high. **Type:** contradicted documentation / maintenance smell.

**Anchor:** [src/editor/gap_buffer.rs:1](/home/shawn/workspace2/reactive-tui/src/editor/gap_buffer.rs:1).

The header calls this zero-copy and describes cursor insert/delete as O(1). move_gap_to allocates temporary Vec<char> copies at 131/155; ensure_gap_capacity copies the suffix at 194; every insert shifts subsequent line-break offsets at 223–233 and delete scans/re-writes them at 260–264. Cursor word movement and painting also allocate text views. A gap buffer may have amortized advantages, but these operations are not universally O(1) and this implementation is not zero-copy. MAX_BUFFER_SIZE is 100,000,000 chars, described as a 100 MB char limit, although Vec<char> needs roughly 400 MB of element storage before temporary copies; constructors do not enforce that growth-only guard.

**Recommendation:** State amortized/local-edit behavior accurately, use copy_within where appropriate, and define memory limits in bytes with fallible growth. This is a source-based complexity/memory accounting finding, not a benchmark.

## N19 — Package, README and native version disagree

**Severity:** low. **Confidence:** high. **Type:** contradicted documentation / version inconsistency.

**Anchor:** [src/ffi/mod.rs:183](/home/shawn/workspace2/reactive-tui/src/ffi/mod.rs:183).

Cargo.toml declares package version 1.0.0. README.md:35 calls the checkout a pre-release 0.1.0 candidate. rtui_version hardcodes 0.1.0 while abi_version remains 1. ABI version is a separate concept, so this does not establish an ABI layout break; it does mean a consumer cannot rely on all three surfaces reporting the same library release. README also still names Syntect although Cargo/source use Lumis.

**Recommendation:** Derive library version fields from Cargo package version, maintain a separate explicit ABI version, and update release/dependency statements together.

## N20 — The legacy C effect and hooks facade is not an automatic reactive system

**Severity:** medium. **Confidence:** high. **Type:** contradicted capability / missing integration. **Reach:** legacy `rtui_effect_*` and `RTuiHooks` APIs; the separate retained foreign-component API is not implicated.

**Anchor:** [src/ffi/reactive.rs:732](/home/shawn/workspace2/reactive-tui/src/ffi/reactive.rs:732).

The module header claims a complete reactive system with effects and hooks. `rtui_effect_create` only boxes `(callback, cleanup, user_data)` at 743–747, with an explicit comment that real hook integration would be needed. Nothing subscribes this object to signals or runs it initially. Only `rtui_effect_run` invokes the body at 787; destroy invokes cleanup. `FFIHooks` separately caches signals by string key at 1036–1066, while its effects vector is initialized empty and never populated. Imports and opaque memo/effect types do not supply dependency tracking. Changing a signal after creating an effect does not run that effect. The manual run function itself can work; the false claim is automatic reactive integration. Native Hooks and the newer owned foreign API have distinct real integration paths.

**Recommendation:** Connect these APIs to retained native hooks with documented dependencies and cleanup ownership, or name/document them as manually driven callback objects and keyed signal storage. Avoid calling the legacy facade a complete reactive system.

## N21 — Markdown table separators are generated from a permanently empty column list

**Severity:** low. **Confidence:** high. **Type:** confirmed rendering defect.

**Anchor:** [src/markdown/ast_walker.rs:184](/home/shawn/workspace2/reactive-tui/src/markdown/ast_walker.rs:184).

`table_alignment` is initialized to an empty vector at 33 and never assigned. The Table branch discards its AST metadata at 184. After a header row the renderer adds `├` at 202, then iterates this empty vector to draw separator segments at 203 and skips its closing-edge branch. A two-column GFM table therefore gets a standalone `├` line instead of a table separator, and its declared alignment is never represented. Source search found only declaration, initialization and these two reads. This is a small supported rendering defect, not a claim that all Markdown is broken.

**Recommendation:** Read the AST's columns/alignment, calculate column widths and draw matching separators; keep one table layout authority. If table layout is unsupported, represent that explicitly instead of partial box drawing.

## N22 — C surface-to-terminal rendering ignores the supplied terminal and restores the host after each call

**Severity:** medium. **Confidence:** high. **Type:** confirmed integration/lifecycle defect.

**Anchor:** [src/ffi/lib.rs:628](/home/shawn/workspace2/reactive-tui/src/ffi/lib.rs:628).

`renderSurfaceToTerminal` dereferences the supplied terminal into an unused `_terminal_ref` at 645. It then constructs a new `Renderer` at 651. That constructor creates another `Terminal`, runs its capability gate, and enters modern mode (`core/renderer.rs:68–70`). The temporary renderer is dropped on return; Drop calls `restore_terminal` (`:45`, `:331–342`), which exits modern mode and disables raw mode through `core/terminal.rs:116–131`. A caller that already used `setupTerminal` thus has its host session restored by a drawing helper while its original handle still exists. This legacy terminal route has no shared ownership counter to neutralize that cleanup. `renderWithStats` repeats the unused-handle/new-renderer pattern at `ffi/lib.rs:739–745`.

The raw-cell copy at 655–660 also discards Surface grapheme/continuation and image side data; a complete Surface copy already exists in core rendering. Pointer checks do not repair either integration problem.

**Recommendation:** Render through the caller's actual terminal/session and reuse its frame/diff state. Keep terminal-mode ownership outside one-shot drawing operations and copy complete Surface metadata. No host-terminal experiment was run.

## N23 — Spring position starts with the opposite of its configured velocity

**Severity:** medium. **Confidence:** high. **Type:** confirmed numerical/API defect.

**Anchor:** [src/animation/spring.rs:100](/home/shawn/workspace2/reactive-tui/src/animation/spring.rs:100).

The response formulas use positive `self.velocity` for the derivative of the remaining displacement, then calculate_position returns `to - response`. Its position derivative at time zero from the positive side is therefore **negative** configured velocity. Independently, calculate_velocity at zero returns **positive** configured velocity at 142. For `SpringConfig::new(1,100,10).with_velocity(1)`, from=0,to=1, the analytic response coefficient gives a position derivative of -1 immediately after zero, while calculate_velocity(0,0,1) reports +1. Critical and overdamped branches use the same inconsistent sign convention. Constructors accept these ordinary parameters; no unusual/nonfinite input is required.

**Recommendation:** Define velocity as the derivative of position consistently and correct each response's initial-condition coefficient. Verify position and velocity agree at zero and immediately afterward for all damping regimes. This is an algebraic source trace, not executed numerical sampling.

## N24 — Completed spring animations can stop far short of their target

**Severity:** medium. **Confidence:** high. **Type:** confirmed playback defect.

**Anchor:** [src/animation/easing.rs:196](/home/shawn/workspace2/reactive-tui/src/animation/easing.rs:196).

Spring easing passes normalized progress directly to calculate_position as physical time. Animation::update limits progress to one at mod.rs:459–473 and samples that easing, then marks the animation Completed at 494–499 without replacing the final value. `SpringConfig::new(1,1,2)` is a valid exactly critically damped configuration. At final progress 1, its 0→1 position is `1 - (1+1)*exp(-1) = 1 - 2/e`, about 0.2642. A ten-second opacity animation with that configuration therefore completes at about 26.4% opacity; a longer duration still samples one physical second. The public Animation::spring builder at mod.rs:292 and modern spring_animate both select this easing. The separate apply_with_values helper is not called by playback and cannot repair it.

**Recommendation:** Map animation duration/progress to a defined spring settling time or run the spring in actual elapsed time, then define final-value/settling behavior explicitly. Do not report completion while leaving an unintended partial destination. No animation or test was executed.

## Qualified and rejected interpretations

- The new foreign-component API guards callback recursion and checks thread ownership; it should not be described as identical to the legacy unsafe callbacks.
- Typed pointer trackers do exist for some renderer/surface/terminal/buffer families. The claim is inconsistent coverage and unsound safe Rust entry points, not absence of all tracking.
- GPU is explicitly marked future in animation/performance.rs; that individual label is candid. The optional graphics canvas elsewhere has real GPU/software implementations and is reviewed separately.
- AccessKit integration deliberately limits verified screen-reader combinations and records upstream provenance. Structural mapping of its inherited interface facades does not establish a defect.
- Typed keyframes validate offsets and lossless conversions; the old/default-value criticism would be stale for that path.
- No runtime performance rate, crash reproduction, test pass, or author intent is asserted.
