# Terminal, platform, graphics and vendored-crate audit

Reviewed frozen source at `/home/shawn/workspace2/scratchpads/tmp/reactive-tui-review-20261004-i_nse8c5`. All locations below are original-repository-relative and refer to that snapshot, not to edits made concurrently in the live repository. This is a static review. No tests, builds, Clippy, rust-analyzer, lint gates, benchmarks or Sudus commands ran.

Audit checklist: source/provenance mapping complete; production-path review complete for the focused areas listed in coverage; findings and coverage cross-check complete. “Deep” means focused source-level behavior review, with partial-file limits recorded; it does not mean every branch was verified. “Structural” means AST/source inventory and symbol mapping, sometimes selected ranges, without a full behavioral audit. The coverage file deliberately records structural coverage for substantial upstream and generated code.

Scope: `src/backend/`, `src/core/`, `src/platform/`, `src/terminal/`, `src/embedded/`, `src/graphics/`, `src/escape/`, `src/display/`, and the four vendored terminal crates. Navigation used Tilth maps, definitions and ranges, plus Ripwire static tree-sitter trees and focused symbol/signature maps. The upstream provenance documents were read. No source was edited.

The default SuprTUI path uses its own renderer, ownership guard and patched Crossterm input path. Several findings below affect the separately exported direct/core/escape APIs. They should not be presented as defects in every backend. The FFI scalar-at-a-time Surface writer and widget indexed palette findings belong to other review lanes.

## Findings index

| ID | Severity | Confidence | Classification | Affected path |
|---|---|---|---|---|
| T01 | P1 | High | Confirmed defect | Windows DirectTtyBackend input |
| T02 | P2 | High | Confirmed defect | Unix direct/threaded input Escape |
| T03 | P2 | High | Confirmed defect | Direct bracketed paste |
| T04 | P2 | High | Confirmed defect | Public TerminalWriter styled output |
| T05 | P2 | High | Confirmed defect | Public escape::parser UTF-8 |
| T06 | P2 | High | Confirmed defect | Public escape::parser string termination |
| T07 | P2 | High | Confirmed defect | DirectTty startup input |
| T08 | P2 | High | Confirmed defect | TokioEventLoop stdin flags |
| T09 | P2 | High | Confirmed defect | Adaptive FPS bounds |
| T10 | P2 | High | Confirmed defect / claim contradiction | Public Rgba contrast ratio |
| T11 | P2 | High | Confirmed defect | TokioEventLoop restart |
| T12 | P2 | Medium-high | Design risk, concrete race | Unix resize dispatcher teardown |
| T13 | P2 | High mechanism, medium reach | Design risk | Multiple independently initialized native TTY owners |
| T14 | P2 | High mechanism | Design risk | Graphics initialization and process stderr |
| T15 | P2 | Medium | Design risk | Crossterm startup parser mode after timeout |
| T16 | P2 | High | Confirmed defect | Public SpanDiffWriter resize with statistics |
| T17 | P2 | High | Confirmed defect | Public GraphemeSurface wide-cell edits |

P1 means a major user workflow fails on the affected supported path. P2 means a real correctness defect or an actionable reliability risk with narrower reach. The risks are not runtime-confirmed defects.

## T01 — Windows direct backend never reads input in the App polling loop

**Severity:** P1. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/windows.rs:274`.

`WindowsTty::read_input_events` compares elapsed time with the timeout before its first `read_input_record`. For a zero timeout it always breaks and returns an empty vector, including when console records already wait. `DirectTty::poll_events` delegates to it at `src/platform/mod.rs:583`; `DirectTtyBackend::poll_event` passes the requested timeout through at `src/backend/direct_tty.rs:313`. The backend does not override `Backend::poll_event_with_wake`, whose default loop calls `poll_event(Some(0))` at `src/backend/mod.rs:172`. `App::run_loop` calls that method at `src/app.rs:288`.

**Reasoned reproduction:** run an App with DirectTtyBackend on Windows and type a key, click, or change focus. Every application input poll returns no console event. Resize may still appear through the independent size comparison.

**Impact:** keyboard and mouse operation fails for this backend. The default SuprTUI/Crossterm path has a separate implementation.

**Recommendation:** perform at least one nonblocking console-read/readiness attempt before considering a zero deadline expired. Verify through the actual Backend/App polling contract, not only a nonzero native timeout.

**Static limit:** no Windows console execution or application test ran.

## T02 — A lone Escape stays pending forever on Unix input paths

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/mod.rs:578`.

`EscapeSequenceParser::parse` deliberately retains a lone ESC at `src/platform/parser.rs:84`. `flush_pending` at line 105 resolves it after an input timeout. DirectTty's no-data branch returns an empty vector instead of calling it. The async direct path at `src/platform/mod.rs:593` only invokes `parse`; the threaded Unix event reader at `src/platform/loop.rs:293` also parses newly read bytes without a timed flush.

**Reasoned reproduction:** send only `0x1b`, then poll repeatedly after input is quiet. No Escape event appears. A later `a` is merged into the pending bytes and becomes Alt+a through `src/platform/parser.rs:144`, so even the next key's identity changes.

**Callers/proof:** source search for `flush_pending` found the parser method and a parser test, not a production input caller. The test at `src/platform/parser.rs:1166` manually calls the missing integration step.

**Impact/recommendation:** cancel/dismiss shortcuts fail. Add a bounded Escape ambiguity timeout to synchronous and worker input paths and preserve genuine split escape sequences.

**Static limit:** the caller search is by-name/static; the named concrete paths establish the defect without relying solely on no matches.

## T03 — Direct bracketed paste becomes ordinary key actions

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/parser.rs:685`.

DirectTtyBackend enables bracketed paste at `src/backend/direct_tty.rs:68`. The parser emits only PasteStart and PasteEnd for delimiters and has no paste collection state (`EscapeSequenceParser` contains only `pending`, line 56). Payload bytes continue through normal key parsing. `DirectTtyBackend::map_terminal_event` recognizes `TerminalEvent::Paste(String)` at `src/backend/direct_tty.rs:242`, but discards the delimiter variants.

**Reasoned reproduction:** feed `ESC[200~hello\rworldESC[201~`. The backend delivers characters and Enter, rather than one Paste event. Paste a multiline value into a form or shell-like widget and the carriage return can invoke its Enter action.

**Impact:** pasted content can execute submit/shortcut behavior instead of being inserted as content. This is not merely missing decoration on an event.

**Recommendation:** assemble bounded paste payloads across read boundaries and emit one Paste event; consume the delimiters in that state. Check malformed/truncated paste behavior and cancellation.

**Static limit:** event ordering and mapping were traced in source; no terminal paste was exercised.

## T04 — TerminalWriter resets foreground and background when changing attributes

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/core/writer.rs:182`.

`apply_attributes` emits SGR 0, then only the requested bold/italic/etc. SGR 0 also clears foreground/background. `RenderOpsBuilder::print_styled` emits foreground, background, then attributes at `src/core/render_ops.rs:191`; `styled_text_to_render_ops` uses the same order in `src/core/styled_text.rs:113`. Their cached color state is not invalidated or reapplied by this reset.

**Reasoned reproduction:** build a red, bold styled run and convert it through `render_ops_to_ansi`. Red is selected, then SGR 0 removes it before the run prints. Follow it with a run of the same color but another attribute and the cache skips the supposedly unchanged color.

**Impact:** public RenderOps/styled converters produce the wrong visible color. The default SuprTUI renderer has a different attribute implementation.

**Recommendation:** use selective attribute reset codes or reapply/invalidate color state whenever SGR 0 is emitted. Check interpreted terminal style at each printed run.

**Proof gap:** `src/core/writer.rs:292` checks color code substrings and line 305 checks reset/bold/italic separately. Those assertions do not prove the combined sequence leaves the intended color active.

**Static limit:** no emitted ANSI was executed in a terminal.

## T05 — Public escape parser treats UTF-8 bytes as characters and controls

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/escape/parser/mod.rs:175`.

`Parser::feed` accepts bytes, but `ground` casts each byte to `char`. Its comment says “UTF-8 or extended ASCII”; it has no UTF-8 decoder state. The global C1 dispatch at line 100 also interprets UTF-8 continuation bytes in 0x80..0x9f as terminal controls.

**Reasoned reproduction:** feed UTF-8 `é` (C3 A9): actions print `Ã©`. Feed `界` (E7 95 8C): E7 prints `ç`; the other bytes are lost/handled as C1, rather than completing the character. Some other valid Unicode text contains continuation bytes equal to DCS/CSI/OSC introducers and changes parser state.

**Callers/contract:** this parser is publicly exposed through `src/escape/mod.rs` and `src/lib.rs`. Internal searches did not identify it as the main embedded or legacy Terminal parser; those use different implementations. Its byte-feed/Print(char) API and UTF-8 comment expose the broken contract to downstream consumers.

**Recommendation:** introduce a streaming UTF-8 decoder and explicitly define C1 interpretation; avoid interpreting continuation bytes as independent control introducers.

**Static limit:** no downstream use frequency was established; no parser execution ran.

## T06 — Public escape parser mishandles both BEL and ESC-ST string endings

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/escape/parser/mod.rs:73`.

On BEL, `process_byte` calls `osc_end`, but `osc_end` at line 614 never leaves OSCString. Following ordinary text remains OSC payload. Conversely, ESC is intercepted globally at line 90 and changes state to Escape before `osc_string` or `dcs_passthrough` can collect it. Their “previous byte ESC, then backslash” termination branches at lines 493 and 529 are therefore unreachable for the ordinary 7-bit ST sequence.

**Reasoned reproductions:** `ESC]2;title BEL X` emits the title but no Print(X). `ESC]2;title ESC\\` emits no OSC title at all. A DCS payload ended by `ESC\\` likewise does not dispatch its DCS action.

**Impact/contract:** downstream users of the public escape parser lose text following common title updates and lose common OSC/DCS commands. The separate `src/terminal/parser.rs` uses dedicated string-escape states and is not implicated.

**Recommendation:** make termination transitions explicit, finalize/reset OSC on BEL, and handle ESC-ST within string states before global Escape transition.

**Proof gap:** `test_osc_title` at line 686 checks a BEL title only; it never feeds following text or ST. This explains why the assertion cannot establish stream recovery.

**Static limit:** all reproductions are source traces, not executed cases.

## T07 — Direct capability probing consumes and discards keys typed at startup

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/mod.rs:1233`.

`DirectTty::init` runs `detect_capabilities`. Its probe sends queries with sleeps, then `read_capability_responses` reads every incoming byte into a buffer at line 1299. The entire buffer is passed only to `parse_capability_responses`; unrelated user input is not retained in the input parser or an event queue. Any nonempty buffer also skips environment fallback at line 1232 even if it contains no capability reply.

**Reasoned reproduction:** type `x` while the direct backend starts (during its query sleeps and response-read window). The startup reader consumes `x`; later normal polling cannot recover it. On a nonresponding terminal, that one key also suppresses environment capability detection.

**Impact:** the first commands/text can disappear and capability selection depends on typing timing. The patched Crossterm startup exchange explicitly retains unrelated events and provides an existing pattern.

**Recommendation:** parse startup replies through a shared event source and retain unrelated bytes/events; merge environment evidence with actual recognized replies rather than treating any byte as proof.

**Static limit:** no timing reproduction ran.

## T08 — TokioEventLoop permanently changes the caller's stdin to nonblocking

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/loop.rs:608`.

`start_cancellable_unix_input` duplicates STDIN with F_DUPFD_CLOEXEC at line 594, reads flags, then sets O_NONBLOCK on the duplicate. Duplicated descriptors share the same open-file description and its status flags. Closing the owned duplicate does not restore those flags, and no restore guard exists in this path.

**Reasoned reproduction:** start and stop a TokioEventLoop with initially blocking stdin, then use ordinary blocking stdin input in the same process. With no queued bytes, it can now return WouldBlock. Startup errors after F_SETFL also leave this change behind.

**Cross-check:** `src/platform/unix.rs:293` explicitly opens a separate description for native async input and explains that a duplicate would share flags with synchronous IO. The Tokio path violates the same constraint.

**Impact/recommendation:** a reusable library changes later host input semantics. Use an independently opened terminal descriptor where appropriate, or a lifecycle-coordinated flag ownership mechanism that restores the exact original state on every exit. Avoid treating dup as flag isolation.

**Static limit:** no fcntl/read calls were executed by this review.

## T09 — Adaptive FPS creation and Auto mode ignore configured bounds

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/display/adaptive.rs:104`.

`with_config` normalizes min/max and clamps the recommendation, then replaces it with `mode.target_fps()` without clamping. `set_performance_mode(Auto)` similarly stores an unclamped recommendation at line 283. Other setters use the configured interval, and the public fields describe a minimum to maintain and maximum to attempt.

**Reasoned reproduction:** use min=max=30 with Balanced mode: construction selects 60, exceeding the maximum. Use min=max=100 and Balanced: it selects 60, below the minimum. Switching to Auto can again move outside the interval. If Auto adaptation then reduces a below-minimum target, line 258 calls `clamp(min_fps, target_fps)` with reversed bounds and panics.

**Callers/impact:** `src/app.rs` owns this manager, and the public/async manager APIs delegate to the same methods. Custom performance policy can be ignored or eventually terminate an application.

**Recommendation:** normalize and clamp every target-producing path, and clamp reductions against the validated config interval. Inspect custom intervals for all mode transitions.

**Proof gap:** current mode tests at lines 439-446 use default bounds only.

**Static limit:** the reach of the eventual panic depends on metrics and mode changes; the creation bound violation is unconditional for the examples.

## T10 — Rgba's advertised WCAG contrast result can approve inadequate contrast

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect and claim contradiction. **Primary location:** `src/core/surface.rs:451`.

`contrast_ratio` is documented as WCAG contrast (line 455), but `luminance` multiplies encoded RGB channels directly. The same type already has `to_linear` at line 415 using the sRGB transfer function and `to_srgb` at line 433; its ordinary colors are not required to be linear before calling contrast_ratio. WCAG requires linearization before the weighted luminance sum. [W3C WCAG 2.2 relative luminance](https://www.w3.org/TR/WCAG22/#dfn-relative-luminance).

**Reasoned reproduction:** `Rgba::new(0.4, 0.4, 0.4, 1.0).contrast_ratio(Rgba::black())` computes 9:1. For opaque sRGB gray 102/255, the specified formula gives approximately 3.66:1. A consumer checking the usual 4.5:1 threshold receives a false pass.

**Callers/impact:** this is a public helper; static search found no internal production caller for contrast_ratio. `theme/roles.rs:40` and `backend/suprtui.rs:1080` implement linearization separately. This finding does not claim those paths produce this wrong result.

**Recommendation:** use the existing sRGB conversion before luminance, state the color-space contract explicitly, and consolidate duplicate formula implementations where types allow. Define treatment of transparent colors separately.

**Static limit:** arithmetic is reasoned from the formula and source; no production test or UI measurement ran.

## T11 — A stopped TokioEventLoop silently stops again when restarted

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/platform/loop.rs:665`.

`begin_shutdown` sends true through the persistent watch channel. `stop_async` takes and joins the input task. A later `start_async` can create a new task because the task slot is empty, but `start_input_task` clones the existing receiver without resetting the stop value or replacing its channel. The receiver retained by the object never consumed that changed version. The new Unix supervisor's `shutdown_rx.changed().await` at line 657 is immediately satisfied by the prior true, so it cancels the freshly started reader.

**Reasoned reproduction:** create loop; start_async; stop_async; start_async; await/read the next key. Start returns success, but the new reader is canceled. The non-Unix path also checks the retained true on changed().

**Contract/impact:** the public EventLoop exposes start/stop, start_input_task handles an empty task slot as startable, and no one-shot precondition is documented. Reusing the loop fails without an actionable startup error.

**Recommendation:** reset the lifecycle channel before a new run, or explicitly make the object single-use and reject restart. Clear stale parser/queue state according to that contract.

**Static limit:** no Tokio runtime executed; this follows watch version/state ownership.

## T12 — Resize dispatcher teardown can clear a new owner's callbacks

**Severity:** P2. **Confidence:** medium-high. **Type:** design risk with a concrete concurrent interleaving. **Primary location:** `src/platform/unix.rs:428`.

Last-owner release sets owners to zero and removes dispatcher resources under SIGNAL_LIFECYCLE, then drops that lock before calling stop_signal_handler (`393-409`). Teardown joins the old worker and then clears the process-global SIGNAL_HANDLERS vector. Meanwhile a different thread can initialize a new UnixTty (`install_signal_handlers`, line 350) and register its callback (`register_winch_handler`, line 270).

**Reasoned schedule:** A releases the last old owner and blocks joining its worker. B installs a fresh dispatcher and appends its callback. A finishes join and clears the shared callback vector, removing B's live registration. New SIGWINCH notifications run an empty handler list.

**Impact/recommendation:** concurrent native session replacement can lose resize callbacks permanently. Keep callback membership scoped to the lifecycle generation/owner, or detach the old generation's callback list atomically before permitting new installation.

**Reach/static limit:** no internal production caller of register_winch_handler was established; this is a publicly exposed lifecycle risk, not a demonstrated default App failure. The existing signal test is serial. No scheduling test ran.

## T13 — Independently created native TTY sessions restore each other's terminal state

**Severity:** P2. **Confidence:** high mechanism, medium application reach. **Type:** design risk. **Primary location:** `src/platform/unix.rs:171`.

Each UnixTty::init saves the terminal's current termios, sets raw mode, and owns an independent TtyState. Clones share one state's ownership, but separately initialized instances do not. Each final state drop restores its own saved termios at line 91. WindowsTty similarly captures/restores console modes per instance at `src/platform/windows.rs:496`.

**Reasoned schedule:** start with cooked mode; initialize A (saves cooked), then B (saves raw). Drop A while B is live: the terminal becomes cooked underneath B. Drop B last: it restores raw, leaving the host terminal raw after all sessions end.

**Impact/recommendation:** reusable native sessions lack a process-wide exclusivity or ownership contract. Share terminal-mode ownership, or reject overlapping independent sessions on the same terminal. Coordinate legacy core Terminal and SuprTUI ownership if simultaneous use is supported.

**Cross-check/static limit:** SuprTUI has its own RAW_MODE_OWNERS guard at `src/backend/suprtui.rs:1098`; that does not protect independent native termios owners. Concurrent multi-session support is not clearly promised; therefore this is a design risk, not a universal startup/shutdown defect. No terminal state was changed by the audit.

## T14 — Graphics initialization discards unrelated process stderr

**Severity:** P2. **Confidence:** high mechanism. **Type:** design risk / deliberate tradeoff. **Primary location:** `src/graphics/worker.rs:400`.

QuietTerminalStderr dup2s /dev/null over process descriptor 2 while HybridRenderer initializes. Its mutex serializes only other instances of this guard, not unrelated threads writing stderr. The source openly acknowledges at line 364 that all such writes are lost. Graphics adapter/device initialization can wait for deadlines in `src/graphics/gpu.rs:488`, so the interval is not limited to one driver print.

**Reasoned reproduction:** while the first canvas initializes a hardware adapter, another worker writes an important diagnostic to a terminal stderr. The diagnostic disappears. A concurrent host stderr redirection can also be overwritten by the guard's later restore.

**Impact/recommendation:** automatic rendering startup can silently suppress application/operator diagnostics. Prefer targeted driver logging controls or isolate adapter probing in a process with its own stderr. If the tradeoff must remain, make it an explicit host choice and document the diagnostic-loss interval publicly.

**Static limit:** suppression duration and driver logging were not measured. This is acknowledged behavior, not evidence of misleading intent or a claim that file/pipe stderr is suppressed.

## T15 — Startup parsing remains in special mode indefinitely if no DA1 arrives

**Severity:** P2. **Confidence:** medium. **Type:** design risk requiring a timeout/late-reply contract decision. **Primary location:** `crates/reactive-tui-crossterm/src/terminal/sys/unix.rs:398`.

The project-owned query_startup sets STARTUP_REPLIES_PENDING at line 378. Successful DA1 parsing clears it in `src/event/sys/unix/parse.rs:408`; write failure clears it too. Timeout or input-read error does not clear it. While true, parse_event interprets a standalone ESC] or ESC_ as an incomplete startup reply instead of Alt+] / Alt+_ (`parse.rs:161,173`), regardless of input_available.

**Reasoned scenario:** query a terminal/multiplexer that never replies to DA1, let the 200 ms backend timeout expire, then type Alt+]. With no following byte, the key stays held indefinitely. A late DA1 would clear the mode, which is why the intended late-response behavior matters.

**Impact/recommendation:** a missing startup response changes later key parsing for the remainder of the process. Define a bounded period for late replies and a way to release ambiguous held key bytes on expiry. Consider an exchange-local state rather than an unowned global boolean.

**Static limit:** no terminal lacking DA1 was exercised. Keeping the flag for delayed replies may be intentional; the unresolved part is how long and how standalone keys are released. This is qualified separately from confirmed findings.

## T16 — Adding diff statistics changes resize rendering semantics

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/core/span_diff.rs:224`.

`SpanDiffWriter::diff` checks old/new dimensions and clears/redraws on a size change (lines 167-182). The separately implemented `diff_with_stats` skips that branch entirely and visits only rows/spans in the new surface. The method is documented as generating a diff with statistics, not as changing resize behavior.

**Reasoned reproduction:** show an old five-column surface containing ABCDE, then diff_with_stats against a three-column surface containing ABC on a host area still at least five columns wide. It writes ABC without clearing the old DE. Shrinking height likewise leaves removed old rows. Plain diff clears the screen in the same case.

**Impact/callers:** downstream consumers of this public API get stale content when they enable statistics. Static search found the implementation and its test, not an internal production caller.

**Recommendation:** share the actual diff-generation path and collect statistics within it, preserving its resize contract. Check interpreted output after dimensions shrink, not only byte/stat counters.

**Static limit:** no ANSI stream or test executed; host geometry staying larger than the resized logical surface is an explicit condition.

## T17 — GraphemeSurface overwrites leave invalid wide-cell occupancy

**Severity:** P2. **Confidence:** high. **Type:** confirmed defect. **Primary location:** `src/core/grapheme_cell.rs:154`.

`set_cell` replaces one cell without clearing a previous wide glyph or its Void continuation. `write_str` calls it directly and creates new continuation cells but never repairs overwritten occupancy (lines 187-204). `to_row_spans` blindly skips Void cells at line 266 and extends spans based only on style at line 230.

**Reasoned reproduction:** write `界B` at column zero with one style, then write `X` at column zero. Cells become X, Void, B. Row spans extend the same-style text as `XB`, so B prints at column one even though the surface still records it at column two. Writing into the trailing half of a wide glyph leaves its leading two-column glyph intact and gives similarly inconsistent output.

**Impact/callers:** the public grapheme-aware Surface/SpanDiffWriter path shifts or retains characters after ordinary edits. This is a different type/path from the scalar-at-a-time FFI Surface finding owned by the native lane.

**Recommendation:** enforce wide-glyph occupancy on replacement, clearing both cells of any overlapped glyph and rejecting/clipping a new wide glyph that cannot fit. Keep span continuity tied to actual column occupancy.

**Proof gap/static limit:** the mixed-width test at line 345 verifies initial placement only; it does not overwrite either half. No rendering test ran. This public legacy path was not established as the default SuprTUI implementation.

## Additional observations and review limits

- Multiple publicly exposed terminal/parser/render implementations duplicate UTF-8, style, mode ownership and capability decisions. Findings T02-T08 show concrete divergence; consolidating authority is more useful than adding another abstraction.
- Direct DA1 parsing at `src/platform/mod.rs:1352` ignores every parsed code and assumes modern color/hyperlink/paste/focus support. It does not read DA1 Sixel attribute 4, whereas the patched Crossterm parser does. Environment estimates and explicit terminal answers should be distinguished in the capability contract; this was not raised as a separate defect without a specific supported-terminal reproduction.
- The public EventQueue is labeled “lock-free” at `src/platform/loop.rs:44`, but all mutation requires &mut self and its actual threaded consumer wraps it in a mutex/condvars. Atomics and unsafe slot writes do not supply a shared concurrent queue API. This is an unsupported performance description and unnecessary complexity, not a memory-safety defect in safe use. There is no concurrency/performance proof from this review.
- GPU/software source review traced shared compilation, raster coverage, blending, clips, glyph placement/atlas failures, adapter/device/readback failure paths, and picture limits. No visual-equivalence or hardware-performance claim was verified. Picture size limits do not establish aggregate scene/glyph/clip-memory bounds; production stress measurements remain absent because execution was forbidden.
- The SuprTUI output path separates logical acceptance and checked output failures and explicitly retries full frames after flush errors. The worker and native PTY paths have bounded queues and owned cleanup mechanisms. These are source observations, not passing checks.
- `crates/libghostty-vt/UPSTREAM.md` identifies a vendored newer upstream API and dependency adjustments; that crate's retained upstream Rust behavior was mapped, not deeply re-audited. `libghostty-vt-sys/src/bindings.rs` is generated and was inspected structurally. Its native archive selection/build plumbing was mapped, not built or cryptographically reverified here.
- Crossterm upstream Windows/style/command code and examples were mapped. Focused deep review covered the owned Unix readiness/startup/associated-text changes. The no-Mio tty implementation has inherited behaviors that were not automatically attributed to this project's patch. SuprTUI Unicode tables are generated; large upstream Unicode helpers/tests were structural coverage, while rendering/wide-cell paths received focused review.
- Tests were source evidence only. Existing tests generally verify individual parser/output pieces; the reported proof gaps concern their ability to establish integration behavior. No test failure/pass is claimed.
- Cross-lane questions: confirm intended support for public legacy direct/core/escape APIs and overlapping independent terminal sessions; confirm the permitted late-startup-reply period for T15. Other lanes own scalar-at-a-time FFI text placement and widget indexed-color conversion. None of these questions blocked writing this report.
