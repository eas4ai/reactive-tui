# Platform layer and public terminal types

Prefix: PLT

The platform layer (src/platform) is what an application reaches when it
chooses the direct TTY backend (`DirectTtyBackend`, src/backend/direct_tty.rs)
instead of the default SuprTUI backend, or uses the types directly: `DirectTty`
over `UnixTty` and `WindowsTty`, the `EscapeSequenceParser` that turns the
terminal's bytes into events, and the threaded and Tokio event loops
(src/platform/loop.rs). Beside it the crate exposes a public escape parser
(src/escape), the legacy output types of src/core (`TerminalWriter` with
`RenderOps`, `GraphemeSurface` with `SpanDiffWriter`, `Rgba`), the adaptive
frame-rate manager (src/display/adaptive.rs) that the App owns, the startup
exchange of the vendored crossterm fork (crates/reactive-tui-crossterm) that
the default backend runs before its first frame, and the graphics worker's
guard over the process's standard error while a hybrid renderer starts
(src/graphics/worker.rs). Two of these paths are on every application's way:
the startup exchange (PLT-004) on Unix, and the graphics guard (PLT-016) when
the `wgpu-graphics` feature paints a canvas or a pixel look.

Read on 2026-10-04 from the developer's production code review of 65e618ec
(its terminal and graphics report, findings T02 to T17), each confirmed on
34b7958b on 2026-10-05 by reading: a lone Escape stays pending on every Unix input path of the
platform layer, so the next key becomes Alt+key; the direct backend turns a
bracketed paste into ordinary key presses, a pasted carriage return into
Enter; the startup capability probe swallows keys typed while it waits and
takes any byte as proof that the terminal answered; the fork's startup
exchange leaves its pending flag set when no device-attributes reply ever
comes, so a standalone `ESC ]` or `ESC _` is held instead of being Alt+] or
Alt+_; the public escape parser casts UTF-8 bytes to characters and treats
continuation bytes as C1 controls, never leaves an OSC string on BEL and
cannot see the `ESC \` ending of an OSC or DCS string; `TerminalWriter`
emits SGR 0 before an attribute and so drops the colors set just before,
while the converters' color caches believe them set; `diff_with_stats` skips
the size-change redraw that `diff` has; a `GraphemeSurface` overwrite leaves
half a wide glyph behind and its spans shift the next character; `Rgba`'s
WCAG contrast skips the sRGB linearization; the Tokio loop leaves the
caller's stdin non-blocking; a stopped Tokio loop restarts into an immediate
shutdown; the resize dispatcher's teardown can clear a newer owner's
callback; two independently started native sessions restore each other's
terminal state; the adaptive frame-rate manager stores targets outside its
configured bounds and can panic on a reversed clamp; graphics startup
replaces the process's stderr with /dev/null for the whole initialization.

[PLT-001] On the Unix input paths of the platform layer (the direct backend's synchronous poll, its async events, the threaded event loop's reader and the Tokio event loop's reader) a lone Escape byte MUST be delivered as the Escape key once no further byte has arrived within 50 ms of it, and a byte that arrives after that deadline MUST be its own key, not an Alt-modified one; bytes that complete an escape sequence within the deadline MUST still form that sequence.
Falsifier: An `EscapeSequenceParser` given `ESC`, then nothing, then `a` after 50 ms, driven through `DirectTty::poll_events` over a socket pair (or the parser's deadline path those loops share) yields anything other than an Escape press followed by a plain `a`; or `ESC [ A` given within the deadline yields anything other than one Up key.
Mechanism: review-terminal
Rationale: The parser kept a lone ESC on purpose and had a `flush_pending` for the quiet case, but no production path called it, so Escape never arrived and the next key merged into Alt+key (T02).
Status: Agreed 2026-10-05

[PLT-002] The direct backend MUST deliver a bracketed paste as one `Paste` event carrying the pasted text with its line endings as pasted, assembled across read boundaries; the payload MUST be bounded at 1 MiB, the rest of a longer paste discarded; the paste delimiters MUST produce no key events; and a paste whose end never arrives MUST be released as a `Paste` of what was collected when the deadline of PLT-001 passes.
Falsifier: Feeding `ESC [ 200 ~ a CR ESC [ 201 ~` to the parser, or the same bytes split into three reads, yields anything other than one `TerminalEvent::Paste("a\r")`; or the backend's mapping of that event is not one application paste event; or an Enter key reaches the application from the pasted carriage return.
Mechanism: review-terminal
Rationale: The backend enabled bracketed paste but the parser emitted only PasteStart and PasteEnd and let the payload through as keys, so a pasted carriage return invoked Enter (T03).
Status: Agreed 2026-10-05

[PLT-003] The direct backend's startup capability probe MUST hand every byte that is not a recognized capability reply on to the input parser as ordinary input, MUST use the environment's evidence (`COLORTERM`, `TERM`, `TERM_PROGRAM`) whenever no reply was recognized, and MUST merge recognized replies with that evidence rather than take any byte as proof that the terminal answered.
Falsifier: A `DirectTty` over a socket pair whose peer answers the probe with the single byte `x` and nothing else, in a process whose `COLORTERM` is `truecolor`, reports `true_color` false or fails to deliver `x` as a key from the first poll after the probe; or a peer that answers with a valid DA1 reply followed by `x` loses the `x`.
Mechanism: review-terminal
Rationale: `read_capability_responses` read every byte in its window into a buffer that only the capability parser saw, so keys typed at startup vanished and one key disabled the environment fallback (T07).
Status: Agreed 2026-10-05

[PLT-004] The default backend's startup exchange on Unix MUST stop treating a standalone `ESC ]` or `ESC _` as the start of a late reply once the exchange has ended, whether by the device-attributes reply, by its timeout or by an error: the pending flag MUST clear when the exchange ends, bytes held as a possible reply MUST then be parsed again and delivered as the keys they are (Alt+] and Alt+_), and a device-attributes reply that arrives later MUST still be recognized and consumed, never delivered as keys.
Falsifier: With the exchange ended by its timeout, `parse_event(b"\x1b]", false)` or `parse_event(b"\x1b_", false)` returns no event; or an event source holding `ESC ]` at the moment the exchange ends does not deliver Alt+] without further input; or a DA1 reply fed after the timeout reaches the application as key events.
Mechanism: review-terminal
Rationale: `query_startup` set STARTUP_REPLIES_PENDING and only a DA1 reply or a write failure cleared it, so on a terminal that never answers DA1 a standalone `ESC ]` stayed held for the rest of the process (T15).
Status: Agreed 2026-10-05

[PLT-005] The public escape parser (`reactive_tui::escape::Parser`) MUST decode its input as UTF-8: a multi-byte character, whole or split across `feed` calls, MUST print as that one character, and a continuation byte MUST never act as a C1 control introducer; an invalid byte MUST print as U+FFFD and leave the parser in its ground state.
Falsifier: Feeding `é` (`C3 A9`) prints anything other than the single character `é`; feeding `界` (`E7 95 8C`), whole or one byte per call, prints anything other than `界`; feeding `ÛX` (`C3 9B 58`) enters the CSI state or prints anything other than `Û` then `X`.
Mechanism: review-terminal
Rationale: `ground` cast each byte to a char and the global C1 dispatch took continuation bytes 0x80..0x9f as DCS, CSI or OSC introducers, so `é` printed as two characters and `界` lost two of its bytes (T05).
Status: Agreed 2026-10-05

[PLT-006] The public escape parser MUST end an OSC string on BEL and on `ESC \`, and a DCS string on `ESC \`, dispatching the string's action and returning to its ground state, so that the text that follows prints; the two bytes of `ESC \` MUST be recognized across `feed` calls.
Falsifier: Feeding `ESC ] 2 ; t BEL X` yields anything other than the set-title action followed by a print of `X`; feeding `ESC ] 2 ; t ESC \ X`, whole or split between the ESC and the backslash, yields anything other than the same; feeding `ESC P q x ESC \ X` yields no DCS action with payload `x` before the print of `X`.
Mechanism: review-terminal
Rationale: `osc_end` dispatched the action but never left the OSC state, and the global ESC transition made the `ESC \` branches of the string states unreachable, so text after a title update was swallowed and every `ESC \`-ended string was lost (T06).
Status: Agreed 2026-10-05

[PLT-007] `TerminalWriter` MUST change text attributes without resetting the foreground and background in force: a styled run converted through `RenderOps` or the styled-text converters and written out MUST leave its colors and its attributes active on the terminal, with an attribute set on an earlier run not carried into a later one that lacks it.
Falsifier: The ANSI output of `render_ops_to_ansi` for a red-on-blue bold `A` followed by a red-on-blue italic `B`, interpreted by a VT100 emulator, shows `A` other than red on blue and bold, or `B` other than red on blue, italic and not bold.
Mechanism: review-terminal
Rationale: `apply_attributes` emitted SGR 0 before the attribute, which also clears the colors the converters had just set, and their color caches then skipped the unchanged color on the next run (T04).
Status: Agreed 2026-10-05

[PLT-008] `SpanDiffWriter::diff_with_stats` MUST produce the same terminal output as `diff` for the same surfaces, statistics added: when the surface's size changed it MUST clear the screen and redraw, so no cell outside the new size keeps old content.
Falsifier: Showing a five-by-one surface `ABCDE` and then applying `diff_with_stats` against a three-by-one surface `ABC`, interpreted on a five-column emulator, leaves `D` or `E` on screen; or a one-by-two surface `A` over `B` diffed against a one-by-one `A` leaves `B` on the second row.
Mechanism: review-terminal
Rationale: `diff_with_stats` was a second implementation that skipped the dimension-change branch, so enabling statistics changed what the screen showed after a resize (T16).
Status: Agreed 2026-10-05

[PLT-009] A `GraphemeSurface` write MUST keep every wide glyph whole: writing into either column of a wide glyph MUST clear both of its cells before the new content is placed, a wide glyph that does not fit before the right edge MUST not be placed, and `to_row_spans` MUST give each span the columns its text occupies, so a span never prints a character at a column other than the cell that holds it.
Falsifier: On a three-by-one surface, writing `界B` at column 0 and then `X` at column 0 yields cells other than `X`, blank, `B`, or spans that print `B` at column 1; writing `界B` and then `X` at column 1 leaves any part of `界`; or writing `界` at column 2 places a glyph.
Mechanism: review-terminal
Rationale: `set_cell` replaced one cell without touching the overlapped glyph or its continuation and `to_row_spans` skipped Void cells and joined spans by style alone, so after an overwrite `B` printed one column left of where the surface held it (T17).
Status: Agreed 2026-10-05

[PLT-010] `Rgba::contrast_ratio` MUST be the WCAG 2 contrast ratio: each color's relative luminance computed from its channels linearized by the sRGB transfer function (the 0.04045 breakpoint), opaque colors assumed, and its doc comment MUST say so.
Falsifier: `Rgba::new(0.4, 0.4, 0.4, 1.0).contrast_ratio(Rgba::black())` differs from 3.657 by more than 0.01, the same gray against white differs from 5.742 by more than 0.01, or `Rgba::new(0.02, 0.02, 0.02, 1.0)` against black differs from 1.031 by more than 0.01.
Mechanism: review-terminal
Rationale: `luminance` weighted the encoded channels without the linearization the type's own `to_linear` provides, so gray 0.4 on black scored 9:1 and a caller checking the usual 4.5:1 threshold got a false pass (T10).
Status: Agreed 2026-10-05

[PLT-011] The Tokio event loop MUST leave the caller's standard input as it found it: its reader MUST not change the status flags of the open file description behind descriptor 0, reading instead from its own description of the terminal when standard input is one, and restoring every flag it set on stop or on a failed start otherwise.
Falsifier: With descriptor 0 replaced by the blocking read end of a pipe whose writer stays open, starting and then stopping a `TokioEventLoop` leaves `O_NONBLOCK` set on that descriptor, or a start that fails after the loop touched the descriptor leaves it set.
Mechanism: review-terminal
Rationale: `start_cancellable_unix_input` duplicated stdin and set `O_NONBLOCK` on the duplicate, which shares the description's flags, and nothing restored them, so the host's later blocking reads returned WouldBlock (T08).
Status: Agreed 2026-10-05

[PLT-012] A `TokioEventLoop` that was stopped MUST start again on the next `start` or `start_async` and deliver input until it is stopped again: each run MUST have its own shutdown signal, not the one the previous run's stop already fired.
Falsifier: A `TokioEventLoop` reading from a pipe that is started, stopped, started again and then given a key delivers no event for that key, or its second start reports success while its reader has already ended.
Mechanism: review-terminal
Rationale: `begin_shutdown` sent true on a watch channel the loop kept for life and `start_async` cloned the same receiver, so the new reader's `changed()` fired at once and start reported success for a loop that read nothing (T11).
Status: Agreed 2026-10-05

[PLT-013] A resize-dispatcher callback registered by a new owner MUST survive the teardown of the previous generation: the callbacks of the old generation MUST be detached before its worker is joined, so clearing them cannot remove a registration made in between.
Falsifier: With the old generation's worker held inside a callback, releasing its last owner on one thread while another thread installs a new dispatcher and registers a counting callback, then letting the old worker finish, leaves the new callback unregistered, so a following SIGWINCH dispatch counts nothing.
Mechanism: review-terminal
Rationale: `release_signal_handler` dropped the lifecycle lock before `stop_signal_handler` joined the old worker and cleared the process-global handler vector, so a callback registered during the join was cleared with the old ones (T12).
Status: Agreed 2026-10-05

[PLT-014] Native terminal sessions (`UnixTty`, `WindowsTty`) initialized independently on the same terminal MUST share the ownership of its mode: the first to start saves the terminal's state and sets raw mode, the terminal stays raw while any of them lives, and the state the first one saved is restored when the last one ends, whether by an explicit restore or by drop.
Falsifier: Two `UnixTty` sessions initialized in turn on the slave of one pseudo-terminal pair that started cooked: dropping the first leaves the terminal cooked while the second lives, or dropping the second leaves it raw.
Mechanism: review-terminal
Rationale: Each session saved the state it found and restored it on its own drop, so dropping the earlier session put the terminal into cooked mode under the later one, and dropping the later one left the host's terminal raw (T13).
Status: Agreed 2026-10-05

[PLT-015] The adaptive frame-rate manager MUST keep every frame-rate target within its configured minimum and maximum: construction with a performance mode, `set_performance_mode` (Auto included) and every adaptive adjustment MUST clamp their result to the configured interval, and an adjustment MUST never panic.
Falsifier: `AdaptiveFpsManager::with_config` with minimum and maximum both 30 in Balanced mode reports a target other than 30, or with both 100 a target other than 100; or a manager with both 145 switched to Auto reports a target below 145, or panics when a dropped frame is recorded after the adjustment cooldown.
Mechanism: review-terminal
Rationale: `with_config` and the Auto branch stored the mode's fixed target without clamping, and a later reduction called `clamp` with the minimum above the target, which panics (T09).
Status: Agreed 2026-10-05

[PLT-016] Graphics initialization MUST not discard the process's standard error: while a hybrid renderer starts, output other threads write to descriptor 2 MUST reach the terminal (or wherever descriptor 2 points) in order, at the latest when initialization ends; the driver's conformance warning MUST be kept off the terminal by a targeted means (a filter on the lines the driver prints), and the manual MUST say what is filtered.
Falsifier: With descriptor 2 pointing at the slave of a pseudo-terminal pair, a marker line written to descriptor 2 by another thread while the guard is held is not read from the master once the guard is released; a line that is not the driver's conformance warning is dropped; or manual/wgpu-graphics.md still says that stderr goes nowhere or that another thread's writes are lost, or does not quote the driver line that is filtered (`not a conformant Vulkan implementation`).
Mechanism: review-terminal
Rationale: `QuietTerminalStderr` put /dev/null over descriptor 2 for the whole of `HybridRenderer::new`, whose GPU waits run up to five seconds each, so any diagnostic another thread wrote in that time was lost (T14).
Status: Agreed 2026-10-06
