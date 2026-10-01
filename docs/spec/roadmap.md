# Roadmap

Current: data-widgets

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
the commitment after it. On 2026-09-29, after the graphics canvas, the developer
opened the widget work, which includes the charts ("charts are widgets are
they not?"), asked for it to be measured against gpui-kit 0.7.0, and put
the layout first, because every widget and every chart sits in it. The
order after it, which the developer confirmed the same day: the theme's
colors with the menus and dialogs, the input widgets, the layout and data
widgets, canvas pictures made ready off the App's thread, the charts with
two axes, the radial and flow charts, behavior shared by the widgets
(change callbacks, screen-reader actions, copying over SSH), and new
widgets. A widget family is brought to the widget bar in a commitment of
its own, because BAR-003 asks a reworked widget to meet all of it at once.
After layout-cells the developer opened the second piece the same day. It
is cut in two, because the menus and the dialogs are two families of about
6,000 and 9,000 lines: theme-menus first, which gives the theme its color
roles and brings the menus to the widget bar, then the dialogs, the modal,
the popover and the toasts on the same roles.
After input-widgets the developer opened the fourth piece, the layout and
data widgets, and confirmed its cut in two on 2026-09-30: layout-widgets
first (the tabs, the accordion, the breadcrumb, the scroll view and the
stack), then data-widgets (the table, the data table, the tree, the file
explorer and the progress bar), each with its own review, adversary and
goldens.

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
support is also asked of the terminal at startup; canvas text uses the
application's font, else on Linux the first loadable font fontconfig sorts
for monospace, else a bundled DejaVu Sans Mono; the pixel
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


## layout-cells

Requirements: LAY-001, LAY-002, LAY-003, LAY-004, BAR-001, BAR-002, BAR-004, BAR-005, BAR-007, BAR-008, BAR-009, BAR-010, PNT-001, PNT-002, PNT-004, PNT-005

The first piece of the widget work. It takes in backlog item
wide-grid-gap-collapse (3b93aecd) and the next-feature items
data-table-panel-growth (b1947cfb), glossary-canvas-citations (62adabf0)
and bar-007-dated-reports (9fe6d0d5). In the widget catalog on 2026-09-29,
at 160 columns, the first two cards of a row touched and the third stood
two cells off; a cell spanning a whole grid ended one cell short; and the
data table's card grew on every layout while its filter and column panels
were open.

The commitment makes a gap the number of cells its class asks for, at every
width, with equal tracks at most one cell apart and spanning items ending
where their tracks end (LAY-001). The padding, margin, gap and space classes
count in cells, as the width and height classes do (LAY-002): every such
class in src, examples, tests, the manual and the README is rewritten to
the number that paints what it painted before, so `p-1` becomes `p-4` and
`gap-0.25` becomes `gap-1`, and no golden changes because of the unit.
`gap-x-N` and `gap-y-N` keep the other direction, `col-span-full` spans the
grid it is in, and the auto-fit and auto-fill classes make columns of at
least the width they name (LAY-003). A layout settles within three presents (LAY-004):
each panel of the data table takes the height of its content, up to half
of the space the table's parent gives. BAR-007 carries the developer's
ruling of 2026-09-29 on dated reports, and the path check leaves out the
widget study as it does the recon. PNT-001, PNT-002, PNT-004 and PNT-005
and the goldens of BAR-004 keep holding on the same checks.

Widget sizes, widget colors and the charts are not part of this commitment.
Each widget family is brought to the widget bar (BAR-003 to BAR-006) in a
commitment of its own, where the defects found in it are fixed with it.

Done when every named requirement passes; the new mechanism (layout) has
recorded a fail on a violating example for each of its requirements; every
golden that changed is regenerated and looked at; the manual's layout page
states the unit and the changelog names its change as breaking; and the
review records screenshots from Kitty of the catalog's Input, Layout and
Data display pages at 100, 160, 240 and 400 columns.

## theme-menus

Requirements: THM-001, THM-002, THM-003, MNU-001, MNU-002, MNU-003, MNU-004, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The second piece of the widget work, first half. In the widget catalog on
2026-09-29 the menu bar, the context menu and the popup menu were white
boxes on the dark page, the context menu covered the page's header line,
and the dialog menu's card was empty, because its demo never opens it. No
widget under src/widgets named a color of the theme.

The commitment gives the theme the color roles a widget needs beyond the
twelve it has: the text drawn on each fill, the current row, the row under
the pointer, a field, the focus border, the veil behind a modal and a
shadow (THM-001). Every built-in preset defines them, with text that
contrasts with its fill by at least 4.5 to 1. A theme an application wrote
before these roles existed still resolves each of them (THM-002), and a
change of theme shows in the next frame (THM-003). The roles are defined
once, here, for every later piece of the widget work.

The menu bar, the context menu, the popup menu and the dialog menu are
brought to the widget bar (BAR-003 to BAR-006): every color from a role,
one default look for the props and the builder (MNU-001), panels as wide
and as tall as their rows up to the viewport (MNU-002), opened beside what
opened them (MNU-003) and painted whole over what was on the screen, also
inside a modal, a popover or a box that clips (MNU-004), a
key that opens the context menu, goldens at two sizes, and for each of the
four a page in the catalog and a heading in the manual. The catalog opens
its dialog menu and its context menu inside their cards, gets a key that
changes the theme, so every page can be seen under each preset, and gives
every card one cell of padding inside its border. The
widget-bar check learns the palette classes (`bg-gray-800`, `text-white`)
as color literals and reads the menu code; what it then finds in the
chart, image and canvas code is fixed here too.

The dialogs, the modal, the popover and the toasts are not part of this
commitment; they keep their colors until the next one. The spacing
variables of a theme, loading a theme from a file and the syntax colors
are not part of it either.

Done when every named requirement passes; the new mechanisms (theme,
menus) have recorded a fail on a violating example for each of their
requirements; every golden that changed is regenerated and looked at; the
manual's theme page lists the roles and what a theme that lacks one gets,
and the changelog names what changed for an application; and the review
records screenshots from Kitty of the four menus under each of the five
presets at 100 and 240 columns.

## overlays

Requirements: OVL-001, OVL-002, OVL-003, OVL-004, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The second piece of the widget work, second half. In the widget catalog on
2026-09-29 the five dialogs drew light text on a light grey box under the
dark preset, a toast was a green box with white text under every preset,
and the modal was centered in its card and cut by it. `DialogThemes` gave a
button 16 cells by 8 rows of padding, a popover opened 8 rows below its
trigger with an arrow 8 rows deep, and two toasts at one corner were
painted on the same cells (defect 5 of the next-feature item
widget-study-defects, 6ddc057b, which is fixed here because it breaks
OVL-003; the item keeps its other six).

The commitment brings the modal, the popover, the toast and the
confirmation, input, autocomplete, progress and wizard dialogs to the
widget bar (BAR-003 to BAR-006) on the color roles of theme.md: every
color from a role and one default look for the props, the builder and the
dialog engine, with the looks `light`, `dark` and `high_contrast` taken
from the presets (OVL-001); padding in cells, a box no wider than half the
viewport with its message wrapped, and a popover one row from its trigger
(OVL-002); a dialog centered on the screen and a toast in its corner with a
margin, both painted whole even when their owner stands inside a card or a
box that clips, toasts that stack instead of overlapping, a popover that
flips to the side that holds it, and one stacking order under the menu
panels (OVL-003); the title as the screen reader's label, `AlertDialog`
for a confirmation dialog, and the focus moving into a popover opened by a
key and back on close (OVL-004). Goldens at 80 by 24 and 400 by 100 for
each of the eight; a heading of its own in the manual for each; the
catalog's eight cards kept. `DialogBounds`, `DialogComponent::get_bounds`,
`DialogUtils::calculate_size` and `DialogBuffer`, which nothing reads or
calls, are removed and the changelog names the removal as breaking. The
widget-bar, goldens, catalog-manual and frame-budget checks learn the
overlay family: the modal's fade and the progress dialog are measured
against the frame budget.

The input, layout and data widgets, loading a theme from a file, the
`border-<color>` classes (which paint a background) and a utility class for
the unclipped painting are not part of this commitment.

Done when every named requirement passes; the new mechanism (overlays) has
recorded a fail on a violating example for each of its requirements; every
golden that changed is regenerated and looked at; the manual has a heading
for each of the eight and the changelog names what changed for an
application; and the review records screenshots from Kitty of the eight
cards under each of the five presets at 100 and 240 columns.

## input-widgets

Requirements: CTL-001, CTL-002, CTL-003, CTL-004, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The third piece of the widget work. In the widget catalog on 2026-09-30 the
TextInput card was empty under every preset, the checkbox, the radio
button, the select and the slider were text in the page's color with a
`▶` before the focused one, the slider's value wrapped under its label at
100 columns, and no control named a role of the theme. `builder::button()`
painted 16 cells by 8 rows of padding in one blue, the text input's cursor,
selection, placeholder and error line were palette classes that the input
and autocomplete dialogs inherited, and a select's open list was drawn
inside its own box, five rows at most, pushing the page down.

The commitment brings the text input, the checkbox, the radio button, the
select, the slider and the button (`builder::button()` and
`builder::primary_button()`) to the widget bar (BAR-003 to BAR-006) on the
color roles of theme.md: every color from a role, one look for the props
and either builder, and focus, hover and disabled shown by color alone
(CTL-001); a field, a select's row and a slider's track that fill the width
their parent gives, and a select's list that shows every option (CTL-002);
the select's list and the text input's suggestion list opened as a panel
over the page, flipped up when there is no room, painted whole inside a
modal, a popover or a box that clips (CTL-003); a spoken name and state for
each control, an `aria_label` on the props and the builders, the radio
group's orientation, the select's name, the error line as an alert, and a
key for every pointer action (CTL-004). The input and autocomplete dialogs
take the text input's new field look, and their goldens are regenerated.
Goldens at 80 by 24 and 400 by 100 for each of the six; a heading of its
own in the manual for each; the catalog's TextInput card painted and a
Button card added. The widget-bar, goldens and catalog-manual checks learn
the input family; nothing in it animates, so the frame-budget check keeps
holding on the overlays and the charts.

Change callbacks on the builders, copying through OSC 52 and the screen
reader's SetValue are the eighth piece; the number input, the one-time code
input, the switch and the rating are the ninth; the text input's
typing-time mask, its prefix and suffix cells, merged undo steps and
Alt+arrow word moves are not part of this commitment. The two builders of
the text input and of the radio button keep their names.

Done when every named requirement passes; the new mechanism
(input-widgets) has recorded a fail on a violating example for each of its
requirements; every golden that changed is regenerated and looked at; the
manual has a heading for each of the six and the changelog names what
changed for an application; and the review records screenshots from Kitty
of the catalog's Input page under each of the five presets at 100 and 240
columns, with a control focused and the select open.

## layout-widgets

Requirements: NAV-001, NAV-002, NAV-003, NAV-004, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The fourth piece of the widget work, first half. In the widget catalog on
2026-09-30 the tab with the focus carried a `▶`, a disabled tab was written
`~label~`, the selected tab of two variants and every tooltip were palette
colors, the accordion's focused header carried a `▶ ` and a disabled one
was grey, the breadcrumb gave each segment four cells of padding at each
side, so `/catalog/layout/widgets` showed as `Root / … / widgets` in a card
of 85 cells and cut `Root` to `Ro` in one of 36, the scroll view was 80 by
24 cells unless the application sized it, gave up a column to its bar
whether or not the content overflowed and took no click or drag on it, and
a tab bar wider than its parent was cut at the edge while the arrow keys
still reached the tabs outside it.

The commitment brings the tabs, the accordion, the breadcrumb, the scroll
view and the stack (`widgets::layout`, `builder::tabs()`,
`builder::accordion()`, `builder::simple_accordion()`,
`builder::breadcrumb()`, `builder::path_breadcrumb()`,
`builder::scroll_view()` and `builder::stack()`) to the widget bar (BAR-003
to BAR-006) on the color roles of theme.md: every color from a role, one
look for the props and either builder, the tab variants on `surface`,
`secondary` and `primary`, and focus, hover and disabled shown by color
alone (NAV-001); the widgets fill the width their parent gives and the
scroll view its height too, one cell of padding on a tab's label and a
segment, and a bar column only while the content overflows (NAV-002); a
tab bar that scrolls to keep the focused or selected tab in view, a
breadcrumb that keeps its first and last segments whole, and a scroll view
that scrolls on the wheel, on a click on its track and on a drag of its
thumb (NAV-003); a name from `aria_label` on the props and the builders, a
tab's, a header's and a segment's position and count, the scroll view's
offsets, and a key for every pointer action (NAV-004). Goldens at 80 by 24
and 400 by 100 for each of the five; a heading of its own in the manual
for each. The widget-bar, goldens and catalog-manual checks learn the
layout family; the accordion animates its body, so the frame-budget check
measures it.

The data widgets are the next commitment. Change callbacks on the builders,
copying through OSC 52 and the screen reader's actions beyond focus and
click are the eighth piece; new widgets are the ninth.
The tabs' overflow menu, a tab's `max_width`, the breadcrumb's overflow
strategies other than the default, and the stack's alignment options are
not changed by this commitment.

Done when every named requirement passes; the new mechanism
(layout-widgets) has recorded a fail on a violating example for each of its
requirements; every golden that changed is regenerated and looked at; the
manual has a heading for each of the five and the changelog names what
changed for an application; and the review records screenshots from Kitty
of the catalog's Layout page under each of the five presets at 100 and 240
columns, with a tab focused and the accordion's first section open.

## data-widgets

Requirements: DAT-001, DAT-002, DAT-003, DAT-004, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The fourth piece of the widget work, second half. In the widget catalog on
2026-09-30 the table, the data table and the tree sat in a grey box with a
white border under every preset, the table showed one of its two columns
in a card of 85 cells, no row showed focus, selection or hover, the data
table showed at most thirteen rows, the file explorer painted its
selection and its errors in palette colors and named itself in fixed
English, and the progress bar was blue on light grey under every preset;
the table sorted numbers as text and a tree node selected under a
collapsed parent stayed hidden.

The commitment brings the table, the data table, the tree, the file
explorer and the progress bar (`widgets::display`, `builder::data_table()`,
`builder::tree()`, `builder::progress_bar()`, `builder::file_explorer()`
and the file explorer presets) to the widget bar (BAR-003 to BAR-006) on
the color roles of theme.md: every color from a role, one look for the
props and either builder, a box without a fill of its own, selected rows in
`accent`, the cursor row in `selection`, and focus, hover and selection
shown by color alone (DAT-001); the widgets fill the width and the height
their parent gives, columns share the width by weight with a title's
width as the minimum, and no default caps a data table's rows (DAT-002);
numeric columns sort by their numbers, a selected tree node is revealed
and a collapse keeps the cursor near (DAT-003); names from `aria_label` on
the props and the builders, counts, indices, levels and the sort direction
for the screen reader, and a key for every pointer action, the column
sort among them (DAT-004). Goldens at 80 by 24 and 400 by 100 for each of
the five; a heading of its own in a new manual chapter for each. The
widget-bar, goldens and catalog-manual checks learn the data family in
this spec phase; the progress bar animates, so the frame-budget check
measures it. Three defects of `widget-study-defects` (6ddc057b) are fixed
here: sorting as text, the column minimums and the hidden selection; the
item keeps its other four.

The two-axis charts are the sixth piece and canvas pictures off the App's
thread the fifth, as agreed; the data table's filter and column panels, its
export, the file explorer's operations and preview, and the progress bar's
stripes and pulse keep their behavior and are not changed by this
commitment; change callbacks on the builders and the screen reader's
actions beyond focus and click are the eighth piece.

Done when every named requirement passes; the new mechanism
(data-widgets) has recorded a fail on a violating example for each of its
requirements; every golden that changed is regenerated and looked at; the
manual has a heading for each of the five and the changelog names what
changed for an application; and the review records screenshots from Kitty
of the catalog's Data display page under each of the five presets at 100
and 240 columns, with a row of the table focused and a tree node selected.
