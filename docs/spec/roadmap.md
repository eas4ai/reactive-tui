# Roadmap

Current: graphics-canvas

Order agreed with the developer on 2026-09-21: charts first on a cell canvas,
then a general graphics canvas over wgpu that replaces the rasterizer
underneath, then the remaining widget families measured against gpui-kit.
The quality bar (BAR) applies to every commitment. On 2026-09-22 the
developer ruled that the SuprTUI renderer plan lands before the radial
charts, so filled shapes are drawn once on the new blitters. On 2026-09-26
the developer replaced widget parity with a study of how gpui-kit builds
its widgets compared with ours, and ordered the dependency checks, then
that study, before the graphics canvas. On 2026-09-27 the developer put
input protocols next, split into mouse-input and then
keyboard-and-queries, and ruled that widget changes from the study wait
until input protocols, incremental layout and the graphics canvas are
done. After keyboard-and-queries the developer put incremental layout
next, then the graphics canvas. After incremental-layout-rows the developer
confirmed the graphics canvas next and kept the charts in characters until
the commitment after it.

## charts-plot-layer

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, CHT-010, CHT-025, CHT-011, CHT-012, CHT-013, CHT-014, CHT-017, CHT-018, CHT-019, CHT-020, CHT-021, CHT-022, CHT-023, CHT-024, CHT-026, CHT-027, CHT-028

Deliver the shared plot layer and the cartesian charts on it: line, area,
scatter, bar (vertical and horizontal) and candlestick, with braille and eighth-block rasterization on a
worker, theme colors, the swatch-row tooltip with crosshair, mouse and
keyboard selection, value transitions, fill-parent sizing with mini, medium
and large size classes, goldens at three sizes, a catalog page and a manual section per type.

Done when every named requirement passes, each mechanism has recorded a
failing violating example before its passing receipt, the five workspace
gates pass, and the review finds no chart type mapping data outside the
plot layer.

## suprtui-renderer

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, RAS-001, RAS-002, RAS-003, RAS-004, RAS-005, RAS-006, RAS-007, RAS-008, PNT-001, PNT-002, PNT-003, PNT-004, PIP-001, PIP-002, BLT-001, BLT-002

Deliver the SuprTUI renderer plan captured from charts-plot-layer: a
rasterizer that tracks the cursor and the last emitted style and writes
bytes without per-cell allocation, a row diff over the column arrays,
statistics and an overlay for them, replay equivalence against recorded
screens, pipelined presentation with one frame in flight and flush
failures reported, a painter fast path for untransformed nodes, a per-cell
hit grid that the event layer prefers, one element copy per present,
layout reuse for an unchanged spec, and image fallback through half-block,
quadrant, sextant, octant and braille blitters chosen by tier.

Order: rasterizer (RAS), presentation (PIP), painter and hit testing (PNT),
image fallback (BLT). Each phase lands with its mechanism recording a
failing violating example before its passing receipt; the charts goldens
and the frame-budget mechanism (BAR-005, CHT-021) stay green throughout;
every borrowed algorithm is credited in the crate's UPSTREAM.md.

Done when every named requirement passes, the manual sentence in PIP-002
is updated, and the review finds no per-cell allocation or per-cell reset
on the render path.

## suprtui

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, RAS-001, RAS-002, RAS-003, RAS-004, RAS-005, RAS-006, RAS-007, RAS-008, PNT-001, PNT-002, PNT-003, PNT-004, PIP-001, PIP-002, BLT-001, BLT-002

Supersedes suprtui-renderer by the developer's ruling on 2026-09-23. The
developer raised RAS-005's unchanged-frame bound to 1 millisecond while
suprtui-renderer was open, and a started commitment's frozen contract is
not amended, so this commitment freezes the revised text. Its
requirements and done-when are those of suprtui-renderer, and the
rasterizer, presentation, painter and blitter work delivered under it
carries over unchanged.

Deliver the SuprTUI renderer plan already landed under suprtui-renderer:
check every requirement on the current code, record each mechanism's
review against the frozen text, and close with the commitment review.

Done when every named requirement passes, the manual sentence in PIP-002
is updated, and the review finds no per-cell allocation or per-cell reset
on the render path.

## z-index-tests-segfault

Requirements: BAR-001

Promoted from backlog item 51085de3. tests/z_index_tests.rs crashed with
SIGSEGV once in the full workspace test run on 2026-09-22 (receipt
dba7b94c) and passed every standalone rerun. The tests use only
apply_utility_classes and StyleBuilder, with no unsafe code. The host's
memory overclock, the cause of the other crashes seen then, was turned
off on 2026-09-23.

Run the z_index_tests binary in a loop beside a full workspace test run,
with backtraces and core dumps on. A crash that turns up is fixed at its
cause, with a test that reproduces it; if none turns up, the item closes
as the host's fault.

Done when every quality-bar requirement passes and the z_index_tests
binary passes 1,000 runs under that load without a signal, or a crash
found in those runs is fixed and the test that reproduces it passes.

## expansion-depth-stack

Requirements: BAR-001

Promoted from backlog item e9a86157. With the embedded-terminal feature,
the test recursive_expansion_fails_with_a_bounded_error in
tests/api_component_expansion.rs overflows the 2 MB test-thread stack. The
component runtime stops expansion at depth 128 (src/component/runtime.rs),
and in a debug build each level's frame in expand is large enough that 128
levels fill the stack. With default features the test passes.

Recursive expansion returns its bounded error at depth 128 within the
default test-thread stack whatever features are enabled: each level's frame
shrinks, or expansion stops recursing. A regression test that the
default-feature gates run fails before the fix and passes after it, and it
reports a stack overflow as a failure instead of aborting the test run.

## animation-close-inflight-pass

Requirements: BAR-001

Promoted from backlog item 1a7574c9. An animation update pass that another
thread began before a hook owner closes can still set one value after
close() returns. update_animations delivers each task outside the registry
lock, and the stagger, spring and keyframe update closures check the
owner's generation and then set the value without holding the lock that
cancel_owned_work takes (src/hooks/animation.rs), so a pass already past the
check sets the item once more. The tests wait for such a pass; the library
does not.

After a hook owner's close() returns, no animation value it owns changes
again, whichever thread runs an update pass. The fix holds no lock across
ThreadSafeSignal::set, whose subscribers may start or cancel animations. A
regression test that the default-feature gates run makes the race
deterministic, for example by holding an update pass between its
generation check and its set, and fails before the fix and passes after it.

## hook-liveness-without-owner

Requirements: BAR-001

Promoted from backlog item 251656c1. The timer, throttle and clipboard
hooks check that their hook owner is open by upgrading a
Weak<HookResources>, as the animation owners did before cb91d5e5.
HookTimer::restart and HookTimer::fire do this while holding the timer's
state lock (src/hooks/timer.rs). If another thread drops the last Hooks
clone meanwhile, the upgraded reference is the last one, and dropping it
runs HookResources::close on this thread; the timer's cleanup then calls
HookTimer::close, which waits for the state lock this thread holds.
ThrottledFunction::call, install_timer's cleanup and the clipboard hook's
copy and paste callbacks (src/hooks/clipboard.rs) upgrade the same way
without holding a lock.

No timer, throttle or clipboard path takes a strong reference to its hook
owner to check that it is open; each checks through Liveness
(Hooks::liveness), as the animation owners do. A regression test that the
default-feature gates run drops the last Hooks clone while timer restarts
and fires run on other threads, hangs (stopped by a time limit) with the
Weak upgrade, and passes with the fix.

## suprtui-unused-modules

Requirements: BAR-001, BAR-007

Promoted from backlog item b12e1176. The renderer crate
(crates/reactive-tui-suprtui) keeps seven modules from its import that
nothing outside the crate uses: audio, clipboard, layout, sys, term,
term_embedded and text, about 7,300 lines of source and 3,500 lines of
tests. The app uses ansi, buffer, render, uni, blit, link and media. The
layout module is the crate's only user of taffy 0.13, so every build
compiles a second taffy beside the app's 0.9. term::Capabilities has ten
flags nothing sets and repeats the app's own probe in
src/core/capabilities.rs. UPSTREAM.md keeps the whole import on purpose, so
that renderer changes can be checked against upstream.

Delete the seven modules and their tests, and the crate's taffy and vte
dependencies (vt100 still brings in vte). Tests of kept modules inside the
deleted test files move to a kept file: sys_core.rs's LinkPool test and its
empty-image decode check. UPSTREAM.md names the removed modules, the date,
and the upstream commit where they can still be read; the renderer modules
stay as imported, so renderer changes can still be compared. Done when the
crate declares only the modules the app uses, Cargo.lock lists one taffy,
and the workspace gates pass.

## charts-radial

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, CHT-010, CHT-015, CHT-016, CHT-017, CHT-018, CHT-019, CHT-021, CHT-023, CHT-024, CHT-025, CHT-026, CHT-028, CHT-029, CHT-031

Deliver pie, donut and radar charts on the plot layer and the shared mask
canvas, with filled shapes drawn through the renderer's blitters as CHT-025
was revised on 2026-09-25: the canvas keeps a color per sample, and a
fill-only cell takes the two-color split of the blitter BLT-002 chooses.
The existing pie and donut (src/widgets/display/charts/live/canvas/pie.rs)
are reworked with inner and outer radius, pad angle and side labels with
leader lines; radar is new. Each type gets theme colors, radial mouse and
keyboard selection with the tooltip, fill-parent sizing with the three size
classes, goldens at three sizes on a fixed blitter tier, a catalog page and
a manual section. Line, bar, area, scatter and candlestick goldens stay
unchanged.

Done when every named requirement passes, each extended mechanism has
recorded a failing violating example before its passing receipt, the five
workspace gates pass, and the review finds no chart type writing glyphs or
colors outside the canvas.

## charts-flow

Requirements: BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, CHT-010, CHT-017, CHT-018, CHT-019, CHT-021, CHT-023, CHT-024, CHT-025, CHT-026, CHT-028, CHT-029, CHT-030, CHT-032

Deliver the Sankey chart on the plot layer and the shared mask canvas, as
CHT-030 and CHT-032 were agreed on 2026-09-25: a chart of nodes and links
laid out in columns by a port of the reference's d3-sankey layout, which
lives in the plot layer; nodes as filled rectangles and links as ribbons
drawn through the blitters, each ribbon shaded from its source node's color
to its target's at the link opacity; node labels beside the nodes; node
selection by pointer and keys, which keeps the selected node's links and
fades the rest, with the tooltip; an error message for a missing node or a
cycle. The builder takes nodes and links with the reference's method names
(CHT-029 as revised on 2026-09-25). The chart gets theme colors, fill-parent
sizing with the three size classes, goldens at three sizes on a fixed
blitter tier, a catalog page and a manual section.

Done when every named requirement passes, each extended mechanism has
recorded a failing violating example before its passing receipt, the five
workspace gates pass, the line, bar, area, scatter, candlestick, pie, donut
and radar goldens stay unchanged, and the review finds no chart type writing
glyphs or colors outside the canvas.

## pie-leader-own-slice

Requirements: BAR-001, BAR-004, CHT-015

Promoted from backlog item c2d20846. On a crowded pie or donut, a thin
slice's label leader can end on a neighbouring slice instead of its own.
On an 80 by 24 pie of [1, 1, 1, 1, 1, 20, 1, 1] at the default label gap,
p0's leader runs along row 0 to column 49, next to p1's cells, while p0's
own cells on that row end at column 44, so p0 reads as p1's label. The
charts-radial acceptance review (0fffbe6d, finding 1) found this in 192 of
440 configurations, on the top and bottom rows. The cause is in
place_labels (src/widgets/display/charts/live/canvas/pie.rs): a leader
starts past the outermost painted cell on its anchor row, whichever slice
painted it.

Every placed label's leader ends next to a cell its own slice paints; a
label whose leader cannot reach its own slice without crossing another
slice, label or leader is left out, and its slice stays in the legend. The
goldens test walks each leader to its end and requires that cell to show
the label's own slice. Done when no pie or donut leader ends on another
slice, any changed pie and donut goldens are regenerated and reviewed, and
the goldens and workspace gates pass.

## pie-labels-free-rows

Requirements: BAR-001, BAR-004, CHT-015

Promoted from backlog item bb776d2e. The charts-radial acceptance review
(0fffbe6d, finding 2) found pie and donut labels left out while free rows
remained: on an 80 by 24 pie of [1, 1, 1, 1, 1, 20, 1, 1], p1 and p7 were
left out because every leader had to run along its label's anchor row,
where an earlier leader already ran. pie-leader-own-slice (done 130af95d)
removed that cause: a leader now starts on the nearest row where its slice
alone paints the outer cell on its side. Measured at c8afb43e on 144
charts (pie and donut; six data sets; 80 by 24, 81 by 25 and 120 by 30;
label gaps 0 to 3), 176 of 1104 labels are left out. 152
of them belong to slices that paint no outer cell alone on their side. The
other 24 are all at label gap 0, where the label touches the circle and
the circle's widest rows leave no column for a leader. The goldens test
still requires only 3 placed labels per chart, so the old cause could
return without failing it.

At label gap 0, a label whose leader finds no room takes one more column,
so it is placed with a leader instead of left out. The goldens test sweeps
pie and donut over those data sets, sizes and gaps and requires that every
label left out belongs to a slice that paints no outer cell alone on its
side. Done when that holds at every gap, any changed pie and donut goldens
are regenerated and reviewed, and the goldens and workspace gates pass.

## debug-backend-deep-tree-stack

Requirements: BAR-001, BAR-005

Promoted from backlog item 9f8b2a5d. DebugBackend lays out and paints on
the thread that runs the app, so drawing the deepest tree component
expansion accepts (128 levels) takes about 1.8 MiB of that thread's stack
in a debug build: 1856 KiB with default features and 2110 KiB with the
embedded-terminal feature, as the expansion-depth-stack acceptance
(25438088, finding 1) measured. A test thread has 2 MiB, less the linked
libraries' thread-local storage (256 KiB with embedded-terminal), so a test
that draws such a tree through DebugBackend with that feature overflows its
stack and aborts the whole test run. SuprTuiBackend already lays out and
paints on its own renderer thread with an 8 MiB stack (RENDERER_STACK in
src/backend/suprtui.rs). For DebugBackend, manual/rendering-and-backends.md
only states the limit and tells tests to use a larger stack.

DebugBackend lays out and paints on a thread with its own stack, as
SuprTuiBackend does, so the app thread's stack no longer decides whether a
deep tree draws. The frames, cells, patches and geometry it reports stay
the same, and the per-frame work stays inside BAR-005's budget. A
regression test draws a 128-level tree through DebugBackend from an app
thread with a 1.5 MiB stack. It runs in a child process with core files
off, so an overflow fails the test instead of ending the run, and the
default-feature gates run it. Done when the test fails before the change
and passes after it, no golden changes, the manual no longer tells tests
to supply a larger stack, and the frame budget and workspace gates pass.

## test-timing-sweep

Requirements: BAR-001, BAR-002

Promoted from backlog item 9e36e7f3. Five tests failed the workspace gates
on a loaded host because they assumed the machine keeps up: a fixed sleep
used to wait for another thread, or a wall-clock bound of one to three
seconds timed from before an App started. Each was fixed when it tripped
(8b5a0b9c, 6862cfb8, 89c88e83, 670561b5). The tree still holds 41
`thread::sleep` calls in tests/ and 33 in test modules under src/, and 19
assertions that bound `elapsed()` at 3 s or less, in 12 files.

Every sleep and wall-clock bound in tests/ and in test modules under src/
is read and sorted into one of three kinds. A sleep that waits for
something another thread or the App does becomes a wait on that condition,
with a hang guard of at least 30 s. A wall-clock bound that only guards
against a hang is raised to such a hang guard. A sleep or bound that is
the behavior under test, such as a timer's delay, a debounce or a product
timeout, stays, measured from when the product starts timing rather than
from test setup, with a comment that says what it tests. Product code
outside test modules does not change, and no test is removed or loses an
assertion. Done when every site is listed in the review with its file,
line and kind, and the workspace gates and the assertion audit pass.

## cargo-deny-bans-duplicates

Requirements: BAR-001

Promoted from backlog item 02949462. `cargo deny --offline check bans`
fails: the dependency graph holds two versions each of sha2, digest,
block-buffer, crypto-common and cpufeatures, which deny.toml neither skips
nor explains, and it warns that the base64 skip is no longer needed. The
second line comes from one place. reactive-tui depends on sha2 0.10 on
Windows only, to check the hash of the pinned ConPTY runtime file in
src/terminal/pty/windows/runtime.rs, while lumis pulls sha2 0.11 through
lumis-wasm-runtime on every platform.

reactive-tui's Windows sha2 moves to the 0.11 line lumis already uses, so
one version of each crate remains. sha2 0.11's digest output has no hex
formatting, so the runtime check encodes the digest itself and compares it
with the pinned value as before. The base64 skip is removed. Done when
`cargo deny --offline check bans` passes with no error and no warning, the
library's pseudo-terminal tests, which check the pinned runtime's hashes
whenever they start a pseudo-terminal, pass on the Windows development
host at the final commit, and the workspace gates pass.

## crossterm-async-std-example

Requirements: BAR-001

Promoted from backlog item 9389bf70. `cargo deny --offline check
advisories` fails with RUSTSEC-2025-0052: async-std has been discontinued.
Its only user is the vendored crate crates/reactive-tui-crossterm, whose
dev-dependencies list async-std for one example copied from upstream,
event-stream-async-std. The same crate keeps the event-stream-tokio
example, which shows the same event stream on tokio,
and nothing reactive-tui ships depends on async-std.

The async-std example and the async-std dev-dependency are removed, and
the crate's REACTIVE_TUI_PATCH.md records the removal as a change from
upstream. Done when `cargo deny --offline check advisories` and `cargo deny
--offline check bans` pass with no error, no tracked file refers to the
removed example's path, and the workspace gates pass.

## timing-sweep-widened-timings

Requirements: BAR-001, BAR-002

Promoted from backlog item a0ffa583. The developer asked for a check of
two changes the test-timing-sweep commitment (ca57e784) made without
asking; both go past its rule that a timing under test stays as it is.

The debounce test in src/hooks/timer.rs keeps its 500 ms delay: its three
calls and the cancel must all run inside the delay, and 10 ms can pass
between two statements on a busy machine. Its fixed 600 ms sleep is
replaced. After the cancel, a second debounce with the same delay is
called on the same hooks, and the test waits for it to run, with a hang
guard of 30 s. The timer thread runs due timers in the order they were
added, so by then the cancelled call's time has passed, and a cancel that
does nothing fails the test however slowly the machine runs. The throttle
test keeps its 500 ms interval: a busy machine can fail it only by
stalling 500 ms between two calls, and a throttle that lets a call
through inside the interval still fails it.

The shutdown test in tests/embedded_terminal.rs keeps its 10 s bound,
which only has to tell killing a child that sleeps 30 s from waiting it
out, and also asserts that the child ended by a signal, since a child
that is waited out exits with code 0. Product code does not change.
Done when the debounce test fails while cancel leaves the pending call in
place, `cargo test --features embedded-terminal --test embedded_terminal`
passes, and the workspace gates and the assertion audit pass.

## dependency-checks

Requirements: BAR-008, BAR-001, BAR-007

Delivers backlog item dd046d9d. GitHub CI is off, so nothing runs cargo
deny, and BAR-008 puts its four checks into every commitment's checks for
the whole workspace. At 7c57d299 the advisory and source checks pass, and
the bans and license checks fail, all through the vendored libghostty
crates: libghostty-vt asks for allocator-api2 0.4 and png 0.18 while the
rest of the tree uses 0.2 and 0.17; bindgen 0.72, which builds
libghostty-vt-sys, uses prettyplease 0.2 and shlex 1.3 while lumis and cc
use 0.3 and 2.0; and bindgen's BSD-3-Clause license is not allowed.

The commitment leaves one version of each crate where a version change in
a vendored crate or a lockfile update can do it, and a bans skip entry with
its reason where it cannot; bindgen's license is allowed for bindgen alone,
as a license exception. Each new skip or exception is put to the developer
before it is committed. deny.toml also drops the x86_64-apple-darwin
target, since Intel Macs are not supported. Done when the dependency-checks
mechanism passes, and the workspace gates and the paths check pass.

## widget-study

Requirements: BAR-007

The developer asked on 2026-09-26 for a comparison, not parity: how
gpui-kit 0.6.6 builds its widgets and what they can do, set against ours,
and a shortlist of widgets we might add. The commitment writes the
study as widget-study.md beside the recon under docs, and changes no code.

For each widget family both libraries have (docs/recon.md section 13
lists twenty, from accordion to virtual list), the study compares the two
implementations: how the widget is structured, its features and options,
its builder API, the states it tracks, and its keyboard and screen-reader
support. It names what is worth adopting and why. For each gpui-kit
component reactive-tui lacks, it says what the widget would be in a
terminal and recommends building or skipping it, with one line on why.
Claims cite a file and line on both sides. Done when every module of
gpui-kit's component crate has an entry, compared, recommended, or marked
as support code rather than a widget, and the paths check passes.
The developer then chooses the widgets to build; each is its own
commitment.

## mouse-input

Requirements: INP-001, INP-002, INP-003, INP-004, INP-005, INP-006, PNT-002, BAR-001, BAR-002, BAR-007, BAR-008

The first half of input protocols (items 53eb7103 and dc0dc1c2, retired
into this plan on 2026-09-27). The widget-study review found that the
default backend never turns on mouse reporting: the widget catalog, run
in a pseudo-terminal, set only the modes 1049, 25, 1004 and 2026, so no
mouse event reaches a widget in a real terminal. The developer named
textual-rs as the model.

The commitment makes the default backend enter and leave with
crossterm's mouse-capture and bracketed-paste commands (INP-001), keeps
the wheel's direction, the buttons, the modifiers and pastes whole when
translating crossterm's events (INP-002), holds a drag and its release
for the element that got the press (INP-003), builds Click,
DoubleClick and TripleClick events on release (INP-004), sends the
wheel to the element under the pointer and passes it to the parent at
a scroll edge in the scroll view, the table and the tree (INP-005), and
merges waiting motion reports (INP-006). CrosstermBackend passes the
renderer's hit grid through, as PNT-002 already requires. Activation
stays on the press. A pseudo-terminal test runs an app on the real
default backend and writes real mouse bytes to it.

Done when the input-pty mechanism passes, painter-goldens passes with a
CrosstermBackend case, and the workspace gates, paths and dependency
checks pass; and when a Windows-only test that reads the console's
input mode after the backend enters passes on the Windows test host,
and the pseudo-terminal test passes on the Mac test host, both recorded
in the review.

## non-linux-build-warnings

Requirements: BAR-001

Backlog item 5a1889bc, promoted after mouse-input on the developer's word
("Clean them up", 2026-09-27). Every reactive-tui target builds with
warnings on the macOS and Windows test hosts, which the Linux-only gates
never see: 8 on macOS (Rust 1.97.1) and 18 on Windows (Rust 1.98), all in
code that predates mouse-input. Two are float literals that a later Rust
release makes an error (src/layout/style.rs:1532 and 1538). The commitment
gives each warned item the platform gate its only users already have, or
removes what no platform uses, and adds no behavior.

Done when the workspace gates pass on Linux, and `cargo build --locked -p
reactive-tui --all-targets` prints no warning on the Mac test host and on
the Windows test host, both recorded in the review.

## keyboard-and-queries

Requirements: INP-007, INP-008, INP-009, INP-010, INP-011, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009

The second half of input protocols, after mouse-input, confirmed by the
developer on 2026-09-27 with the Mac and Windows warnings check. The
default backend pushes the Kitty keyboard flags 1, 8 and 16 when the
terminal reports the protocol, as textual-rs does, and pops them on every
exit and before a suspend (INP-007); crossterm's CSI u parser in
crates/reactive-tui-crossterm learns the associated-text field and the
flag-16 push. Lock and media keys map to their KeyCode, and a modifier
key alone sends nothing (INP-008). The terminal's focus reports go to the
root component only (INP-009). Ctrl+Z that no handler consumes suspends
the App on Unix and resumes it with a full repaint (INP-010). Before the
first frame the backend asks for the keyboard protocol and the background
color, ends with a device-attributes query, waits at most 200 ms, keeps
every key typed meanwhile, and makes the light preset active on a light
terminal when the application set no theme (INP-011). BAR-009 adds a
check that builds every reactive-tui target on the Mac and Windows test
hosts over SSH, with the host addresses in a file outside the repository.
The DirectTty backend and a keyboard protocol on Windows are not part of
this commitment.

Done when input-pty passes with the new tests, host-builds passes, the
gates, paths and dependency checks pass, and the pseudo-terminal tests pass
on the Mac test host, recorded in the review.

## incremental-layout

Requirements: PNT-005, PNT-001, PNT-002, PNT-004, INP-007, INP-011, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009

The last phase of the SuprTUI renderer plan (item suprtui-incremental-layout),
confirmed by the developer on 2026-09-27 after a measurement at 700 by 200
in release: with 4,975 text elements and one changing per frame, layout took
7.1 ms of a 20 ms frame. When a frame's spec changes, the painter keeps the
Taffy nodes of unchanged elements at the same position in the tree, builds
nodes only for new or changed ones, and lays out again only what the change
can move, while its cells and hit grid stay those of a full layout
(PNT-005). PNT-001, PNT-002 and PNT-004 keep holding on the same check. The
same planning round revised INP-007 (text of several characters arrives as
one press per character, as keyboard-and-queries built it) and INP-011 (the
startup questions are Unix only); their checks run unchanged. The App's own
per-frame work, the conversion of elements to a paint spec, and the graphics
canvas are not part of this commitment.

Done when painter-goldens passes with the new PNT-005 tests, input-pty
passes, the gates, paths, dependency checks and host builds pass, and the
frame times at 700 by 200 before and after the change are recorded in the
review.

## incremental-layout-rows

Requirements: PNT-005, PNT-001, PNT-002, PNT-004, INP-007, INP-011, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009

Replaces incremental-layout, which the developer superseded on 2026-09-27
after rewording PNT-005's falsifier: with labels that share their row's
width by their text, a changed text moves its whole row, which may be
measured again, and with fixed-width labels only itself. The work carries
over: the painter keeps the Taffy nodes of unchanged elements at the same
position in the tree, builds nodes only for new or changed ones, and lays
out again only what the change can move, while its cells and hit grid stay
those of a full layout (PNT-005, 5003fc86 and d4672161). PNT-001, PNT-002
and PNT-004 keep holding on the same check. INP-007 and INP-011 carry their
revised text of 2026-09-27; their checks run unchanged. The App's own
per-frame work, the conversion of elements to a paint spec, and the
graphics canvas are not part of this commitment.

Done when painter-goldens passes with the PNT-005 tests, input-pty passes,
the gates, paths, dependency checks and host builds pass, and the frame
times at 700 by 200 before and after the change are recorded in the
review.

## graphics-canvas

Requirements: GFX-001, GFX-002, GFX-003, GFX-004, GFX-005, GFX-006, GFX-007, GFX-008, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BLT-001, BLT-002, INP-007, INP-011

Developer ruling 2026-09-21: stay on wgpu; take lessons from rust_pixel
(tile-first cell buffer, charts kept in characters) and beamterm (whole grid
as one instanced draw through a glyph atlas, sub-millisecond at 45,000
cells, overlay compositing) without adopting their GL stacks. The
commitment replaces the cube demo with a wgpu cell-grid renderer: a
CellFrame uploaded as instances against a glyph atlas in one draw, an
offscreen target sized to the widget, a worker, a 2D drawing layer for
paths, fills and gradients that rasterizes into the same target, and output
by host capability (half blocks everywhere, Kitty or Sixel pixels where the
host supports them). The keystone's "not a windowing system" stands; window
presentation of the same frame is a possible later commitment, not this
one.

Developer rulings 2026-09-27: the architecture stays as it is, a hardware
adapter first and the software renderer when none is usable, and the
canvas's features are designed for the GPU, not reduced for weak hardware;
the Windows test tablet (Iris Xe) is the lowest common denominator, so the
speed bounds are measured there; the charts stay in characters here and move
onto the canvas in the next commitment; the GPU draws at the App's frame
rate, and Sixel and slower links drop frames rather than fall behind; the
canvas stays an optional feature with its own gate (BAR-010); graphics
support is also asked of the terminal at startup; canvas text uses a
bundled DejaVu Sans Mono unless the application supplies a font; the pixel
smoke test runs on a Kitty, WezTerm, Ghostty or foot terminal, since
Windows detection waits with windows-startup-queries; and a speed bound the
tablet cannot meet comes back to the developer with its measurements, not a
smaller feature. The per-widget grid the ruling calls a CellFrame is the
CellGrid (glossary.md).

Deliver the `Canvas` widget (GFX-001) with its scene API, its hardware and
software renderers drawing the same picture (GFX-002) on a named worker
(GFX-003), fast enough on the tablet (GFX-004), shown as Kitty or Sixel
pixels or block glyphs (GFX-005) after the startup graphics queries
(GFX-006), surviving faults (GFX-007), and the cube and torus demos redrawn
as canvas scenes (GFX-008), with the widget bar (BAR-003 to BAR-006) met for
the canvas: a catalog page, a manual section in manual/wgpu-graphics.md,
goldens on the debug backend through the software renderer, and reference
images of each scene under tests/snapshots/canvas.

Done when every named requirement passes, each new mechanism (canvas-scenes,
canvas-hosts, canvas-output, canvas-gates) has recorded a fail on a
violating example, and the review records the developer's smoke test of the
Motion page and the reference scenes on a Kitty or Sixel terminal.

