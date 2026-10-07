# Roadmap

Current: ci-rust-version-bump

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
After pixel-output-holds-the-app the developer opened the sixth piece, the
two-axis charts, on 2026-10-01, ruled that their plot areas move onto
pixels with braille as the fallback and that every widget follows in order
(pixel-widget-looks, 95feb091), asked for the work complete rather than
small ("Break it up into multiple commitments if one is too large"), and
so it is cut in three by dependency: two-axis-charts-correct (everything
visible in braille, to the widget bar and gpui-kit 0.7.0), then
canvas-serves-many-pictures (one drawing thread per App, pictures 1:1 up
to the adapter's limit), then charts-plot-on-pixels (the plot areas as
pictures, hover drawn in them).
After push-ci-workflow-repair the developer confirmed on 2026-10-04 that
pixel-widget-looks goes first while two backlog items wait (escalation
ecdb2397): the shared groundwork of the pixel looks, proven on buttons,
cards, text inputs, checkboxes, radios, sliders and progress bars; each
other family follows in a commitment of its own, its cell look kept as
the fallback.

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

## pixel-output-holds-the-app

Requirements: GFX-003, GFX-005, GFX-009, PIP-001, PIP-002, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010

The fifth piece of the widget work: canvas pictures made ready off the
App's wait. Promoted from the backlog item pixel-output-holds-the-app
(9daf165b) by the developer's ok on 2026-10-01 (escalation b91a6900), with
GFX-009 agreed the same day, since no agreed requirement caught the wait.
Measured on 2026-09-29 and again on 2026-10-01: on a terminal that takes
pixels the backend's worker copies a canvas's picture, finds the cells it
covers and writes it to shared memory or encodes it as base64 or Sixel
before it answers `present` (Graphics::prepare and draw in
src/backend/suprtui/graphics.rs), so with a picture of 1920 by 960 pixels
every frame the App waited a median 129 ms with Kitty and 479 ms with Sixel
on the Windows tablet, and at the 95th percentile 25 ms with Kitty in the
command and 103 ms with Sixel on the Linux development host.

The commitment makes a canvas's picture ready on a picture thread or on
the canvas's worker, which the App does not wait for; the terminal keeps
the last picture until the next is ready, at most one picture of a canvas
waits, and a picture made ready for cells that something now covers is
made ready again rather than written (GFX-009). The canvas worker keeps
its scene rule (GFX-003), frames still drop rather than queue on a slow
terminal (GFX-005), and `present` keeps one frame in flight and reports a
failed flush next (PIP-001, PIP-002). On Windows the picture thread asks
the system not to slow it down, as the canvas's drawing thread does
(decision 01M3NFWY). The Sixel encoder is not made faster here, so a full
screen Sixel canvas on the tablet shows as many new pictures as the
encoder makes, and the rest drop; the canvas's scenes, its renderers, its
choice of output and the block glyphs keep their behavior. The macOS test
hang and the Windows startup queries stay in the backlog by the
developer's ok.

Done when every named requirement passes; the new mechanism
(canvas-pictures) has recorded a fail on a violating example, the code as
it is at the start of this commitment; the manual's chapters on the canvas
and on rendering and the changelog say where a picture is made ready and
what an application sees; and the review records the App's wait printed by
canvas-pictures on the three hosts and screenshots of the widget catalog's
Motion page in Kitty and in a Sixel terminal on a private display.

## two-axis-charts-correct

Requirements: CHT-010, CHT-011, CHT-012, CHT-013, CHT-014, CHT-017, CHT-018, CHT-019, CHT-020, CHT-021, CHT-022, CHT-023, CHT-024, CHT-025, CHT-026, CHT-027, CHT-028, CHT-033, CHT-034, CHT-035, CHT-036, THM-003, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010

The sixth piece of the widget work, the two-axis charts, first of three
commitments. On 2026-10-01 the developer ruled that the line, area,
scatter, bar and candlestick charts draw their plot areas as pixel
pictures where the terminal takes pixels, with today's braille as the
fallback, and asked for all widgets to follow in order (item
pixel-widget-looks, 95feb091). Two read-only surveys the same day found
twenty-four defects and gaps in the cartesian charts against the widget
bar and gpui-kit 0.7.0, and that the canvas serves one picture per
drawing thread with a 4096-pixel cap; measured in release on the Linux
host, a GPU drawing thread starts in 17 to 24 ms and a chart-like plot
costs 1 ms at 640 by 384 and 6 to 7 ms at 1920 by 960 on the GPU against
6 to 7 and 51 to 54 ms on the software renderer. The developer ruled:
"do not defer... I WANT IT DONE CORRECTLY... FULLY", "Break it up into
multiple commitments if one is too large", and, on a resolution cap for
weak hosts, "Don't let the lowest common denominator dictate to the
highest" and "I also can't run Crisis on the tablet". So the work is cut
in three by dependency: this commitment, everything visible in braille;
then canvas-serves-many-pictures (ff62d964), one drawing thread per App
and pictures 1:1 up to the adapter's limit; then charts-plot-on-pixels
(c5ba0fa8), the plot areas as pictures with hover drawn in them.

The commitment brings the five cartesian types to the widget bar
(BAR-003 to BAR-006) on the color roles of theme.md and to gpui-kit
0.7.0: the stacked domain (CHT-011); dots off by default, each area
series with its own stroke, fill and curve, the fill at 0.4 opacity and
the gradient fading to the baseline (CHT-012); bar padding, band width,
minimum length, label colors, gradients and labels that neither overlap
nor leave the plot (CHT-013); the candle body ratio and candles that
animate (CHT-014); chrome colors from roles and `--color-chart-grid` in
every preset (CHT-017); the tooltip opaque on `surface` with a title row,
four candle rows, cell-measured widths, per-point title, value, color and
content, and a ring on a scatter's selected point (CHT-018); every
reference method name per type (CHT-020); the axis options: pinned range,
point count, tick counts and formats, labels inside or outside, dashed
grid and grid columns, reference lines, headroom and `tick_margin` on
both orientations (CHT-034); numeric scatter x (CHT-033); the empty,
error and never-panic rules with the worker failure shown (CHT-026);
hover reaching thinned points (CHT-027); ASCII for the whole chart
(CHT-028); the three builder routes equal (CHT-035); the screen reader at
DAT-004's depth and Up and Down choosing the series (CHT-036); the
catalog with every variant and a real large chart at 240 columns
(CHT-023); and the theme switch repainting a settled chart (THM-003), the
mini title and the medium legend rows (CHT-024) and `.label()` leaking
into the tooltip (CHT-018) fixed under the text that already binds. Each
defect gets a test that fails on the code as it is at the start, and a
defect the test cannot reproduce is dropped with the test kept. The
charts-goldens, plot-layer, charts-builders and charts-interaction checks
learn CHT-033 to CHT-036 in this spec phase, and every revised
requirement records a fail on the code at the start.

Not here, by dependency: the plot pictures, rounded bar corners, hover
halos, bar fading and the gliding band, which cells cannot show, are the
third commitment's; the shared drawing thread and the picture cap the
second's; the radial and flow charts keep their behavior.

Done when every named requirement passes; the four mechanisms have
recorded a fail on a violating example, the code as it is at the start,
for every new and every revised requirement; every golden that changed is
regenerated and looked at; the manual's chart sections name the new
methods and the changed defaults (dots off, the fill at 0.4 opacity, a
numeric scatter x, what `x_axis` hides) and the changelog names what
changed for an application; and the review records screenshots from
Kitty of the catalog's Charts page under the five presets at 100 and 240
columns, with a point in a scatter's second series selected by key and
its tooltip over the plot.

## ffi-feature-does-not-compile

Requirements: BAR-011, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010

The first of the two P1 findings of the developer's independent review of
e30afa23 (2026-10-01), promoted from the backlog on 2026-10-02 after the
developer ranked the two ahead of the canvas work: the library does not
compile with the `ffi` feature, which the C ABI and the TypeScript
binding build it with. PointerTracker<T> carries a PhantomData<T>, so the
static OnceLock<PointerTracker<Renderer>> needs Renderer: Sync, and the
renderer holds the picture thread's mpsc::Receiver through DiffWriter and
Graphics. No gate built the feature, so nothing reported it. Delivered
here: the tracker identifies its type with a PhantomData<fn() -> T> and
keeps its synchronized address registry, and neither the renderer nor the
receiver is made Sync artificially; the feature's warnings under clippy
-D warnings are fixed; the eight ffi test targets build and run; and
BAR-011 adds the ffi gate (ffi-gates), which builds, lints, documents and
tests the crate with `ffi` and builds it with `ffi,wgpu-graphics` on the
Linux host, and builds it with `ffi,wgpu-graphics` on the two test hosts.

Done when every named requirement passes, the ffi-gates mechanism has
recorded a fail on the tree as it was at the start, and the changelog
names the fix for an application that builds the native library.

## thread-safe-signal-lost-update

Requirements: SIG-001, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

The second of the two P1 findings of the developer's independent review of
e30afa23 (2026-10-01), promoted from the backlog on 2026-10-02 after
ffi-feature-does-not-compile: ThreadSafeSignal::update runs its callback
on a copy with the lock released, so two concurrent updates both start
from the same value and the later store overwrites the earlier (two
increments of zero leave 1), and use_reducer dispatches every action
through it. Delivered here: ThreadSafeSignal::update_atomic runs its
callback under the signal's lock and wakes the subscribers afterwards,
mirroring Ref::update_atomic; use_reducer's dispatch and the framework's
own read-modify-writes (the dialog engine's change counter and the
wizard's visited set, the image worker's, popover's and terminal monitor's
revision counters, the clipboard and pointer processor hooks' states) go
through it; a call back into the same signal from an update_atomic
callback fails at once with a message instead of waiting forever; update
keeps running on a copy and its documentation says concurrent updates may
overwrite each other; the manual's reactive chapter names both methods;
and the reactive-signals mechanism (SIG-001) runs the concurrency tests
and scans the eight files for a read-modify-write left on update.

Done when every named requirement passes, the reactive-signals mechanism
has recorded a fail on the tree as it was at the start, and the changelog
names the fix for an application that dispatches reducer actions from
several threads.

## canvas-serves-many-pictures

Requirements: GFX-001, GFX-002, GFX-003, GFX-004, GFX-005, GFX-006, GFX-007, GFX-008, GFX-009, GFX-010, GFX-011, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

The second of the three commitments for the two-axis charts (item
ff62d964), after two-axis-charts-correct and before charts-plot-on-pixels,
opened on 2026-10-02 after the developer's ok let three backlog items wait
(escalation cbd3f018). Until now each canvas drew on a thread of its own
with its own GPU connection, a picture was capped at 4096 by 4096 pixels,
and a picture's pixels were taken to match its cells one to one. The
developer said on 2026-10-01 "It may be that we need to make the canvas
more capable" and ruled, when a fixed 1440p cap was proposed, "Don't let
the lowest common denominator dictate to the highest", "I am not concerned
with users that have a potato" and "I also can't run Crisis on the
tablet": no fixed resolution cap, no automatic step-down for a slow host,
speed bounds bind on the Linux development host and the tablet's numbers
are recorded.

The commitment makes one drawing thread serve every canvas in the process
that draws with the same renderer options, with one adapter and device,
shared pipelines and glyph atlas, and a waiting scene per canvas that only
that canvas's newer scene replaces (GFX-003 revised); draws pictures one
pixel per screen pixel at the terminal's cell size up to the adapter's
limits, lets an application pin fewer whole pixels per cell, and where a
hard limit stands in the way (the renderer's, the frame's room for a
picture, a Kitty command's room, a Sixel picture's text) draws the
largest whole pixels per cell that fit, the
terminal scaling the picture to its cells through the Kitty placement's
`c` and `r` keys and our Sixel encoder scaling before it encodes (GFX-010,
GFX-001 revised); and shows that fifteen canvases on one thread each still
receive 30 new pictures a second on the Linux host, with the tablet's and
the Mac's numbers recorded (GFX-011). The canvas keeps its scenes and
renderers (GFX-001, GFX-002), the tablet's speed floor (GFX-004), its
output choice and in-place replacement (GFX-005, GFX-006), its fault
handling (GFX-007), the demos (GFX-008) and pictures made ready off the
App's wait (GFX-009).

Not here, by dependency: the charts' plot areas as pictures and the hover
drawn in them are the third commitment's, so the Charts page itself draws
no pictures yet; the proof that many pictures share one thread is a tree
of fifteen canvases and the catalog's Motion page. Shared memory and the
frame's 64 MiB budget for new pictures keep their rules. iTerm2 is not an
output of the canvas (GFX-005), so no scaling keys are written for it.

Done when every named requirement passes; canvas-scenes, canvas-output and
canvas-hosts have recorded a fail on a violating example, the code as it
is at the start, for GFX-001, GFX-003, GFX-010 and GFX-011; the manual's
canvas chapter says that one thread serves every canvas, how an
application pins pixels per cell, and the limits as they now are, and the
changelog names what changed for an application; and the review records
GFX-011's numbers from the three hosts and screenshots from Kitty on the
private display of the catalog's Motion page and of a canvas at 520
columns shown one pixel per screen pixel.

## charts-plot-on-pixels

Requirements: CHT-012, CHT-013, CHT-014, CHT-017, CHT-018, CHT-019, CHT-021, CHT-022, CHT-023, CHT-024, CHT-025, CHT-026, CHT-027, CHT-028, CHT-035, CHT-036, CHT-037, CHT-038, CHT-039, GFX-003, GFX-005, GFX-009, GFX-010, THM-003, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

The third of the three commitments for the two-axis charts (item c5ba0fa8),
after two-axis-charts-correct and canvas-serves-many-pictures, opened on
2026-10-03 after the developer's ok let three backlog items wait again
(escalation 37eb2e8d). The developer ruled on 2026-10-01 that the line,
area, scatter, bar and candlestick charts draw their plot areas as pixel
pictures where the terminal takes pixels, with today's braille as the
fallback, and confirmed the contract draft on 2026-10-03.

The commitment makes each of the five chart types draw its plot area, the
grid, reference lines, strokes, fills, markers, bars, candles and the hover
marks, as one picture per chart where the backend reports Kitty graphics
or Sixel at startup: a canvas at the plot rectangle, drawn on the drawing
thread every canvas shares from a scene the chart's worker builds, one
pixel per screen pixel at the terminal's cell size, redrawn when the cell
size changes (CHT-037, CHT-021 revised); axes, ticks, titles, legend,
value labels, inside tick labels and the tooltip stay cell text, painted
over the picture. The plot is blank until its first picture, never braille
that then switches, and without pixels, or with the feature off, the bytes
are the fallback's. The hover moves into the picture as gpui-kit 0.7.0
draws it: a crosshair with dots in halos, a band that glides between bars
while the other bars fade, a ring on a scatter's point, one new picture per
hover change and per frame of the glide, snapped under `reduced-motion`
(CHT-038). Strokes, markers, gradients and pattern fills take sizes from
the cell height (CHT-012 revised); bars get exact pixel edges, pixel
gradients and a `corner_radius` option (CHT-013 revised); thinning is per
pixel column (CHT-027 revised); the mask canvas is the cell path and draws
nothing inside a plot that is a picture (CHT-025 revised). On the Linux
host in release, the nine cartesian charts of the catalog's Charts page in
a 3 by 3 grid at 240 by 60 cells all show their pictures within a second
and keep the App's work per frame and its wait in present within a frame
while a bar chart is hovered every frame, with the tablet's and the Mac's
numbers recorded (CHT-039); the page itself at 240 columns shows one card
at a time, so the measurement puts every chart on screen. The backend records what it learned at startup, pixels
and the cell size, in a process-wide report the chart reads on its first
frame; a canvas can be told that a parent describes it to the screen
reader; `REACTIVE_TUI_CANVAS=blocks` and a process-wide `GraphicsOptions`
for charts turn the plots back to cells or choose the renderer, which the
catalog's `--cpu` uses. The canvas keeps its rules (GFX-003, GFX-005,
GFX-009, GFX-010), the charts keep theirs (CHT-014, CHT-017 to CHT-019,
CHT-022 to CHT-024, CHT-026, CHT-028, CHT-035, CHT-036, THM-003), and the
chart goldens on the debug backend, which takes no pixels, stay as they are.

Not here: pie, donut, radar and sankey charts in pixels; pixels under real
text; the shared groundwork for the other widgets, which is item
pixel-widget-looks (95feb091), though the process-wide report and the
silent canvas are built so it can reuse them.

Done when every named requirement passes; charts-pictures has recorded a
fail on a violating example, the code as it is at the start, for CHT-037,
CHT-038 and CHT-039, and charts-goldens and frame-budget for the revised
CHT-012, CHT-013, CHT-021, CHT-025 and CHT-027; every reference picture is
checked in and looked at; the manual's chart sections say when the plot is
a picture, what the hover looks like and the switches, and the changelog
names what changed for an application; and the review records screenshots
of the catalog's Charts page from Kitty on the private display at 100 and
240 columns in the five presets, with a bar hovered, and from a Sixel
terminal.

## macos-pty-stop-hang

Requirements: TRM-001, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

Promoted from the backlog item macos-pty-stop-hang (64ffce50) by the
developer's choice on 2026-10-03, with TRM-001 agreed the same day, since
no agreed requirement covered the stop of a pseudo-terminal child. The
terminal widget's worker (src/terminal/pty/unix.rs) and the embedded
session's worker (src/embedded/worker.rs) stop reading the child's output
and then call PtyChild::stop (src/terminal/owned_pty.rs), which kills the
child and waits for it without reading the master. On macOS a process that
exits with output still unread on its pseudo-terminal stays in exit until
that output is read, so a child that printed faster than it was read never
finishes exiting, and the stop, the worker and whatever joins it wait
forever: the Mac run of the INP-011 tests at a9231484 hung 20 minutes this
way, the macOS CI runner ran the flooding-child test until its 45-minute
limit on 2026-10-03 (run 37151822540), and the Mac test host ran it for
nine minutes with no exit the same day. Delivered here: while
PtyChild::stop waits for the killed child it reads and discards what the
child left unread on the master until the child is reaped, so the stop
returns within a second on Linux and on macOS (TRM-001), for the terminal
widget, PseudoTerminal::kill and its drop, and the embedded session alike;
the tests stop a flooding child on a thread of their own with a ten-second
guard, so a hang is a failed test and not a stuck run; the pty-stop
mechanism runs them here and on the macOS test host over SSH, and records
the run unverified when the host cannot be reached; the manual's terminal
widget and embedded session chapters say what stopping a printing child
does; and the changelog names the fix.

Done when every named requirement passes, the pty-stop mechanism has
recorded a fail on the tree as it was at the start, on the macOS test
host, and the changelog names the fix for an application that closes a
terminal widget or an embedded session while its shell is still printing.

## push-ci-workflow-repair

Requirements: BAR-012, BAR-001, BAR-002, BAR-004, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

Promoted from the backlog item push-ci-workflow-repair (c001439a) by the
developer's choice on 2026-10-03, with BAR-012 agreed the same day, since no
agreed requirement covered the push workflow. GitHub Actions was re-enabled
on 2026-10-03 for the ffi gates, and the dormant Supported-platform CI
workflow (.github/workflows/ci.yml, 2026-09-16) then ran on every push to
main and failed on all three runners at 95c5f787 (run 37166418577). On
Windows every golden and a fixture's section marker compare against CRLF,
because the runner checks out with core.autocrlf and the repository has no
.gitattributes; the string-path inventory test prints backslash paths; the
two command-shell tests and the terminal widget test find no ConPTY runtime
beside the test binaries; and the dialog HTTP client gets no output back
from its curl child, which fails thirteen dialog tests and one unit test.
On Ubuntu cht_024 waits for a fifth frame after a resize that a runner whose
chart worker finishes inside the resize's frame never paints, and the
pre-Sudus dependency step (scripts/check-pre-release-dependency-maintenance.py)
calls cargo audit's empty warning list a failure. On macOS the large radar golden differs in 169 braille cells and
its color digest, because sine and cosine differ in the last bit between
x86-64 and Apple Silicon. The ffi gates workflow's Linux test step fails
test_terminal_dimensions for want of a controlling terminal. Delivered
here: .gitattributes gives tracked text files LF line endings and marks the
binary fixtures; the inventory test normalises path separators; cht_024
waits for the large class's grid instead of a frame count; the radial
charts take their sines and cosines from libm, so the goldens are the same
on every platform, which the macOS runner checks; the Windows workflow step
installs the ConPTY runtime beside the test binaries; the dialog HTTP
client's Windows failure is found on the Windows test host and fixed with a
test; the workflow runs cargo deny in place of that obsolete step, at
Cargo.toml's rust-version, and scripts/check-pre-release-ci.py follows; the
ffi gates workflow's Linux tests run under a pseudo-terminal; the
ci-workflow mechanism (BAR-012) reads the workflow for its commands,
toolchain, ConPTY step and the line-ending attributes, then looks up or
starts the workflow's run for the current commit and reads its jobs;
CONTRIBUTING.md says what CI runs; and the changelog names the Windows
dialog HTTP fix and the radar goldens' platform independence.

Done when every named requirement passes, the ci-workflow mechanism has
recorded a fail on the tree as it was at the start, the workflow's run for
the final commit is green on the three runners, and the changelog names the
two fixes for an application that validates dialogs over HTTP on Windows or
draws radar charts on Apple Silicon.

## pixel-widget-looks

Requirements: PIX-001, PIX-002, PIX-003, PIX-004, PIX-005, PIX-006, CTL-001, CTL-002, CTL-004, DAT-001, THM-003, GFX-003, GFX-005, GFX-009, GFX-010, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The first of the pixel-look commitments (item pixel-widget-looks, 95feb091),
opened on 2026-10-04 after the developer's ok let two backlog items wait
(escalation ecdb2397). The developer asked on 2026-10-01 for every widget to
move onto the pixel canvas with its cell look as the fallback, and agreed
that the shared groundwork comes first, proven on six kinds of widget, and
that pixels under real text wait: pictures around text are clean in Kitty,
Konsole, WezTerm and Sixel, pictures under text are not.

The commitment builds the groundwork: a look is one picture over the
widget's rectangle, drawn on the drawing thread every canvas shares, one
pixel per screen pixel, with every cell that holds text cut out and painted
in the look's flat color so text reads over it on Kitty graphics and Sixel
alike, in theme roles, with no screen-reader node of its own, and back to
cells where the terminal takes no pixels or the switch says so, byte for
byte the fallback (PIX-001); pictures kept and drawn only when their look
changes, placed again rather than sent again when they move under Kitty,
deleted when they leave, redrawn when the cell size changes, and hidden
under panels, lists, modals, popovers, toasts and dialogs (PIX-002). On it,
the six kinds: rounded containers with borders and rings from their
classes, which gives `builder::button()`, `builder::primary_button()`,
`builder::card()` and `card_builder()` their rounded fills, borders and
focus rings and brings the card helpers to theme roles (PIX-003); the text
input's field, the checkbox's box and the radio's circle (PIX-004); the
slider's track and thumb and the progress bar's track and fill at the exact
pixel of their values, with the indeterminate bar's glide (PIX-005); and the
proof of speed and scale on the catalog's Input page and on a page of 192
looks, binding on the Linux host, measured on the tablet and the Mac
(PIX-006). CTL-001 says that where a control draws its pixel look the frame,
mark, dot, track and thumb are the picture's in the same roles. The controls
keep their widths, keys and screen-reader nodes (CTL-002, CTL-004), the
progress bar its roles (DAT-001), a theme change repaints the pictures
(THM-003), and the canvas keeps its rules (GFX-003, GFX-005, GFX-009,
GFX-010).

Not here: pixels under real text; the select's list, the menus, the
overlays, the tabs, the accordion, the breadcrumb, the scroll view, the
stack, the data widgets, the radial and flow charts, the images and the
terminal, each a later commitment on this groundwork; shadows (`shadow-*`)
and animation beyond the indeterminate bar's glide; a switch widget, which
the crate does not have; pixel-shaped hit testing, so a click and a drag
keep their cell targets.

Done when every named requirement passes; pixel-looks has recorded a fail on
a violating example, the code as it is at the start, for PIX-001 to PIX-006
(CTL-001's revision is a cross-reference whose observation is PIX-004's and
PIX-005's); every reference picture is checked in and looked at; the
manual's input, data and layout chapters say when a look is a picture, what
it looks like and the switches back to cells, and the changelog names what
changed for an application; and the review records PIX-006's numbers from
the three hosts and screenshots from Kitty on the private display of the
catalog's Input page at 240 and 100 columns under the dark and the light
preset, with a text input focused and a button under the pointer, and the
same page from a Sixel terminal.

## review-high-findings

Requirements: SIG-002, FFI-001, INP-012, THM-004, CHT-026, CHT-040, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The first commitment cut from the next-feature item
review-2026-10-04-remediation (2e1476c7), the developer's production code
review of 2026-10-04 over the tree at 65e618ec, opened by the developer's
ok on 2026-10-04 (escalation 290cdb92, which lets the backlog items
windows-startup-queries and syntax-editor-loses-multiline-context wait
until the next Done). It takes the review's six high-priority findings,
each confirmed in the code before the requirements were agreed the same
day: an animation's `on_update` callback runs while `update` holds the
animation's state lock, so a callback that reads its own progress waits on
its own thread forever (N01, SIG-002); 118 of the 243 functions exported to
C are safe on the Rust side although they read or write through their
caller's pointers, which clippy's lint misses because each dereference sits
in the closure handed to `catch_panic` (N02, FFI-001); on Windows the
direct TTY backend's console reader returns before its first read when the
App polls with a zero timeout, so no key, click or focus change reaches the
App (T01, INP-012); two theme variables naming each other recurse until
the stack overflows (W02, THM-004); the smallest subnormal value makes the
tick step zero and the tick loop never ends, in the worker and in
`build()` (W01, CHT-026, revised); and one `band_count`, `point_count` or
`grid_columns` integer makes a chart format a label and keep values for
every requested slot (W04, CHT-040).

Delivered here: an animation copies what its callbacks need and runs them
with its locks released; every export that uses its caller's pointer is an
`unsafe extern "C" fn` with a `# Safety` section, the C headers and the
TypeScript binding unchanged, and the crate's Rust callers wrap their calls;
the Windows console reader tries one read before a zero timeout counts as
spent; theme resolution follows names with a set of the names it passed
and a bound of 32; tick generation needs a finite positive step, falls back
to a representable one and ends; a chart's labels, positions and stacks
follow its data and its plot's columns. The review-high mechanism
(scripts/cairn/review_high.py, tests/review_high.rs) checks SIG-002,
FFI-001, THM-004 and CHT-040 on the Linux host and INP-012 on the Windows
test tablet, where a pseudo console (ConPTY) runs an App on the direct TTY
backend and a key is written to it; charts-goldens checks the revised
CHT-026. The other 58 findings and four risks of the review stay in the
next-feature item for later commitments by subsystem.

Done when every named requirement passes; review-high has recorded a fail
on the tree as it was at the start for SIG-002, FFI-001, INP-012, THM-004
and CHT-040, and charts-goldens one for the revised CHT-026; the manual
says that the C interface's pointer-taking functions are unsafe to call
from Rust and what each needs; and the changelog names what changed for an
application: animation callbacks that may read and change their
animation, the unsafe exports, Windows input through the direct TTY
backend, theme variables that name each other, and charts with tiny values
or huge slot counts.

## review-core-findings

Requirements: CMP-001, CMP-002, CMP-003, CMP-004, CMP-005, CMP-006, CMP-007, STY-001, STY-002, STY-003, STY-004, PNT-006, SIG-003, SIG-004, SIG-005, SIG-006, SIG-007, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The second commitment cut from the next-feature item
review-2026-10-04-remediation (2e1476c7), the developer's production code
review of 2026-10-04 over the tree at 65e618ec. The developer confirmed on
2026-10-05 that the review's 19 core findings go next, ahead of the backlog
items windows-startup-queries and syntax-editor-loses-multiline-context,
which wait until the next Done (escalation cdd35e8a, answer ce9fd969), and
confirmed the recommended choice for each finding with an alternative. Each
of the 19 was checked against the code at 9f177efb before the requirements
were drafted and holds: C01 a dropped render tree unmounts its replacement
(CMP-001); C02 and C15 nothing polls `Component::poll_change` and its
wrapper's pointer check cannot fail (CMP-002); C03 a screen never feeds the
mouse hooks (CMP-003); C04 a move out leaves the mouse position inside
(CMP-004); C09 the virtual DOM diff patches changed names, props and
handlers in place (CMP-005); C10 and C11 the handler cache confuses node ids
256 apart and reports handled input as ignored (CMP-006); C14 the
compile-time component table is an empty map (CMP-007); C05 nine class
variants apply unconditionally (STY-001); C06 `css!` drops
`display: none` (STY-002); C12 `aspect-auto` keeps the earlier ratio
(STY-003); C13 the `css!` documentation promises compile-time validation
(STY-004); C16 explicit cell colors ignore their element's own opacity
(PNT-006); C07 a `Memo` never computes again (SIG-003); C08 context effects
run once (SIG-004); C17 a hook animation paused past its duration is lost
(SIG-005); C18 a spring drops an impulse (SIG-006); C19 the hook
animation's loop settings never reach its driver (SIG-007).

Delivered here, as the developer chose: the App and screens poll a
component's `poll_change` and redraw on its change, through an honest pin
projection; the conditional variants are decided by the App (sibling
position, a held press, an ancestor marked `group`) and `visited:` never
applies; context effects rerun on the signals they read, their function
becoming an `Fn`; the `css!` documentation says what it checks; and the
empty component table is removed. The other fixes make the code do what
each requirement says. The review-core mechanism
(scripts/cairn/review_core.py, tests/review_core.rs and unit tests named by
requirement in the modules whose private parts they need) checks every new
requirement on the Linux host. The review's other 39 findings and four
risks stay in the next-feature item for later commitments by subsystem.

Done when every named requirement passes; review-core has recorded a fail
on the tree as it was at the start for each new requirement; and the
changelog names what changed for an application: the polled components, the
screens' mouse hooks, the variants that now wait for their condition, the
effects' `Fn` bound, the removed component table and the hook animations'
pause, spring and loops.

## review-native-findings

Requirements: ANI-001, ANI-002, ANI-003, ANI-004, ANI-005, ANI-006, ANI-007, ANI-008, ANI-009, TXT-001, TXT-002, TXT-003, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The third commitment cut from the next-feature item
review-2026-10-04-remediation (2e1476c7), the developer's production code
review of 2026-10-04 over the tree at 65e618ec. On 2026-10-05 the developer
chose the review's remaining findings over the two waiting backlog items,
windows-startup-queries and syntax-editor-loses-multiline-context, which
wait until this commitment's Done (escalation 78ac0232, answer 576ccba1),
and confirmed the twelve requirements with the choices they carry: the
alternate animation drivers are removed rather than repaired, `update`
reports whether the animation is still active, a repeated explicit id
replaces the earlier animation and the documentation says so, and a
Markdown table's cells are aligned and padded along with its separator.
Each of the fourteen findings was checked against the code at 2e58a739
before the requirements were drafted and holds, N09 and N13 in part: N03 a
reversed animation stops (ANI-001); N04 a parallel timeline completes while
its animations wait and `update` returns true on the completing frame
(ANI-002); N05 the loop callback and `auto_reverse` live in dead helpers
(ANI-003); N06 stale cleanup removes an animation updated every frame
(ANI-004); N07 two animations made in one millisecond share an id
(ANI-005); N08, N09 and N10 the interpolation cache, the optimized batch
and the lock-free state give wrong values and lose updates, and nothing
uses them (ANI-006); N11 a stagger over a 200-cell grid overflows
(ANI-007); N23 a spring's configured velocity moves it the wrong way
(ANI-008); N24 a spring-eased animation completes short of its target
(ANI-009); N12 the syntax editor paints each line without the document's
context (TXT-001, which also delivers the backlog item
syntax-editor-loses-multiline-context, to retire at Done); N13 the
highlighter re-parses on every call and the byte limits guard only the
checked entry points (TXT-002); N21 a Markdown table's header is followed
by a bare `├` (TXT-003).

Delivered here: the animation module's playback agrees with its states and
its documentation, its timelines complete when their animations do, its
loops run their callbacks and alternate when asked, its manager keeps what
is advancing and names what it makes uniquely, and its two unused drivers
are gone; stagger delays cannot overflow; the spring's analytic velocity is
the derivative of its position and a spring easing reaches its target; the
syntax editor paints with the document's context and the highlighter
parses once per change, with the byte limits at every entry point; and a
Markdown table is drawn from its columns. The review-native mechanism
(scripts/cairn/review_native.py, tests/review_native.rs and unit tests
named by requirement in the modules whose private parts they need) checks
every new requirement on the Linux host. The review's other findings, the
C facade (N14 to N20, N22), the terminal and platform code (T02 to T17, four
of them risks) and the widgets (W03, W05 to W08), stay in the next-feature
item for later commitments by subsystem.

Done when every named requirement passes; review-native has recorded a
fail on the tree as it was at the start for each new requirement; and the
changelog names what changed for an application: reversal, timelines, loop
callbacks and `auto_reverse`, stale cleanup, generated ids, the removed
drivers, stagger delays, the spring's velocity and easing, the editor's
highlighting, the highlighter's reuse and limits, and Markdown tables.

## review-terminal-findings

Requirements: PLT-001, PLT-002, PLT-003, PLT-004, PLT-005, PLT-006, PLT-007, PLT-008, PLT-009, PLT-010, PLT-011, PLT-012, PLT-013, PLT-014, PLT-015, PLT-016, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The fourth commitment cut from the next-feature item
review-2026-10-04-remediation (2e1476c7), the developer's production code
review of 2026-10-04 over the tree at 65e618ec. On 2026-10-05 the developer
chose its terminal and platform findings, T02 to T17, four of them risks,
as the next commitment, ranking them above the waiting backlog item
windows-startup-queries, which follows this commitment (escalation
0d8bb098), and confirmed the sixteen requirements with the choices they
carry: a lone Escape becomes the Escape key after 50 ms of silence; a paste
is one event bounded at 1 MiB; the startup exchange's pending state ends
with the exchange and a late reply is consumed, not shown as keys; native
sessions on one terminal share its mode rather than refusing a second
session; a stopped Tokio event loop starts again; the Tokio loop reads the
terminal through its own file description; graphics startup forwards the
process's stderr through a filtered pipe rather than silencing it or
letting the driver's warning onto the screen; the public escape parser is
fixed, not removed; a wide glyph without room is not placed. Each finding
was checked against the code at 34b7958b before the requirements were
drafted and holds: T02 a lone Escape stays pending (PLT-001); T03 a paste
becomes keys (PLT-002); T07 the startup probe swallows typed keys
(PLT-003); T15 the startup exchange's pending flag never clears (PLT-004);
T05 the public parser casts UTF-8 bytes (PLT-005); T06 it mishandles BEL
and `ESC \` (PLT-006); T04 the writer resets colors with attributes
(PLT-007); T16 `diff_with_stats` skips the resize redraw (PLT-008); T17
wide glyph overwrites leave half a glyph (PLT-009); T10 the WCAG contrast
skips linearization (PLT-010); T08 the Tokio loop leaves stdin non-blocking
(PLT-011); T11 a stopped Tokio loop cannot restart (PLT-012); T12 resize
teardown clears a new owner's callback (PLT-013); T13 independent sessions
restore each other's terminal state (PLT-014); T09 frame-rate targets leave
their bounds (PLT-015); T14 graphics startup discards stderr (PLT-016).

Delivered here: the direct TTY backend delivers Escape, pastes and the keys
typed at startup as they are and takes the environment's word when the
terminal says nothing; the default backend's startup exchange releases what
it held when it ends; the public escape parser decodes UTF-8 and ends its
strings; the legacy writer keeps its colors, its diff clears on resize, its
grapheme surface keeps wide glyphs whole and its contrast ratio is WCAG's;
the Tokio event loop leaves stdin alone and starts again after a stop; the
resize dispatcher and the terminal's mode have one owner at a time; the
frame-rate manager keeps its bounds; and graphics startup keeps the
process's stderr. The review-terminal mechanism (scripts/cairn/review_terminal.py,
tests/review_terminal.rs and unit tests named by requirement in the modules
whose private parts they need, one of them on a pseudo-terminal) checks
every new requirement on the Linux host. The review's other findings, the C
facade (N14 to N20, N22) and the widgets (W03, W05 to W08), stay in the
next-feature item for later commitments by subsystem.

Done when every named requirement passes; review-terminal has recorded a
fail on the tree as it was at the start for each new requirement; and the
changelog names what changed for an application: the Escape deadline, paste
events, the startup probe and exchange, the public escape parser, the
writer's colors, `diff_with_stats`, wide glyphs, the contrast ratio, the
Tokio loop's stdin and restart, resize callbacks, terminal-mode ownership,
frame-rate bounds and graphics startup's stderr.

## windows-startup-queries

Requirements: INP-013, BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

Promoted from the backlog item windows-startup-queries (1947687f), which
keyboard-and-queries left waiting on 2026-09-27 with the developer's ok and
which the developer ranked next on 2026-10-05, after the review's terminal
findings. INP-011 asks the terminal for its background color on Unix only,
so an App on the default backend on a light Windows Terminal started with
the dark preset. On 2026-10-06 a program run as a pseudo console's child on
the Windows test tablet showed the way: the pseudo console forwards the
child's background and device-attributes questions to its terminal
unchanged and hands the child the reply as one console input record per
character with no key code, while typed keys arrive with their key codes;
the crossterm copy's Windows parser read such a reply as typed characters
and dropped its Escape. The developer confirmed INP-013 the same day with
its choices: only the background color is asked on Windows, no
keyboard-protocol or graphics question, and the device-attributes reply
only ends the exchange (its Sixel attribute is not read on Windows, GFX-006
being a Unix requirement); the contract test runs on the Windows test
tablet in a pseudo console and is recorded unverified when the tablet
cannot be reached.

Delivered here: the crossterm copy's Windows terminal module gains the
startup exchange the Unix one has, `query_startup` and `StartupReplies`,
which writes `OSC 11 ; ?` and `CSI c`, collects the no-key-code records
that form a reply while the exchange is pending, ends on the
device-attributes reply or after 200 ms, and gives every other record back
as the key it is, in order; the default backend calls the same function on
both systems and loses its Windows-only stub that answered nothing; the
copy's patch record names the Windows change; the startup-windows
mechanism (scripts/cairn/startup_windows.py) ships tests/startup_windows.rs
to the Windows test tablet over SSH through the test host file, where a
pseudo console answers an App's questions as a light terminal, a dark one,
a silent one, and one whose user types while the replies are due; and the
changelog says what changes for an application started in Windows Terminal.

In the same spec phase the developer confirmed two rewordings of Agreed
text: PLT-016 no longer names the driver's own switch, which the developer
ruled out on 2026-10-05, and the animation specification names the removed
drivers as modules rather than file paths, so the dangling-paths exemption
table is empty again.

Done when every named requirement passes; startup-windows has recorded a
fail on the tree as it was at the start, on the Windows test tablet; and
the changelog names the startup exchange on Windows for an application
started in Windows Terminal.

## review-facade-findings

Requirements: FFI-002, FFI-003, FFI-004, FFI-005, FFI-006, FFI-007, TXT-004, ANI-010, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011, BAR-012

The fifth commitment cut from the next-feature item
review-2026-10-04-remediation (2e1476c7), the developer's production code
review of 2026-10-04 over the tree at 65e618ec: the C facade findings N14,
N15, N17, N19, N20 and N22, and the two findings beside them that the
earlier commitments left, N16's remainder (the animation debugger, after
ANI-006 removed the performance module) and N18 (the gap buffer). Each was
checked against the code at 0aade996 on 2026-10-06: six hold as written and
two in part, N16 as said and N22 for `renderSurfaceToTerminal` alone, since
`renderWithStats` renders through its own renderer and only ignores the
terminal it is given. The developer confirmed the eight requirements the
same day with the choices they carry: one width policy for the C text
buffers, the width-method argument kept for compatibility; the widget
constructors connected to the native controls rather than documented as
pictures; the hit grid, render offset, host statistics and buffer dump
implemented rather than declared unsupported, the grid built by
registrations and read after a completed render as the original design
works, the dump written to `rtui-buffers-<timestamp>.txt` in the current
directory; `rtui_effect_create` made a real effect on a thread-confined
runtime the C signals join, with `rtui_effect_run` kept; one version
everywhere from Cargo.toml, the TypeScript package included; the gap buffer
bounded in bytes with fallible constructors and inserts, the editor refusing
an edit past the limit instead of panicking; and the animation debugger
instrumenting what it runs.

Delivered here: the C text-buffer renderers paint whole grapheme clusters by
their width as the editor does; `rtui_text_input_create`,
`rtui_checkbox_create` and `rtui_progress_bar_create` return the native
text input, checkbox and progress bar, so an App edits, toggles and shows
them; the C renderer gains a hit grid, a render offset and kept host
statistics that the debug overlay shows and `dumpBuffers` writes to its
file; `renderSurfaceToTerminal` paints through the caller's terminal with a
complete copy of the surface and opens no second session;
`rtui_effect_create` runs its callback when made and again when a C signal
it read changes, with cleanup before each rerun and at destroy, and the
hooks are documented as the keyed signal storage they are; `rtui_version`,
README.md, the umbrella header and the TypeScript package report Cargo.toml's
version and README names Lumis; the gap buffer's documentation states its
cost and its byte limit is enforced by every constructor and insert, which
fail instead of panicking; `DebugAnimationManager::update` records frame
times, logs updates and takes verbose snapshots. The review-facade mechanism
(scripts/cairn/review_facade.py, tests/review_facade.rs and its modules, run
with the `ffi` feature, three of them on a pseudo-terminal because the C
renderer takes raw mode, and unit tests named by requirement where a
module's private parts are needed) checks every new requirement on the
Linux host. The review's widget findings (W03, W05 to W08) stay in the
next-feature item for the next commitment.

Done when every named requirement passes; review-facade has recorded a fail
on the tree as it was at the start for each new requirement; and the
changelog names what changes for a C or TypeScript program: text painted by
grapheme, native controls from the widget constructors, a working hit grid
and statistics, surface rendering through the caller's terminal, effects
that run on their signals, one version, and for a Rust program the gap
buffer's limit and the animation debugger's instrumentation.

## ci-rust-version-bump

Requirements: BAR-012, BAR-001, BAR-002, BAR-003, BAR-004, BAR-005, BAR-006, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011

The backlog item ci-rust-version-bump (3516be48), from the developer's
"1.91 is old" of 2026-10-06 when the first push-workflow run of
windows-startup-queries failed its Windows clippy step at 1.91.0, and the
day after, the review-facade commitment's first run failed only on the GNU
Windows toolchain the hosted job built, a flavor no user runs and the
Windows test host never verified. On 2026-10-07 the developer ruled that
runners must be free to hold different toolchains, that the workflow pins
1.95 ("Yes 1.95 pin"), the same as the floor and the Linux gates'
toolchain, and that running CI on the developer's own machines is a
separate project outside this repository; BAR-012 was revised accordingly
and keeps GitHub's hosted runners.

Delivery: Cargo.toml's `rust-version` rises to 1.95; `.github/workflows/ci.yml`
pins `1.95.0` by full name on every job, `1.95.0-x86_64-pc-windows-msvc` on
Windows, installed by rustup beside whatever the runner holds and named on
every cargo call, with no `rustup default`, no `rustup set default-host`
and no MSYS2 step, the weekly advisories job under the same rules; README.md,
manual/wgpu-graphics.md and CHANGELOG.md say the new minimum; whatever
clippy 1.95 raises on the macOS and Windows runners under `-D warnings` is
fixed; and scripts/cairn/ci_workflow.py's static part checks the new
clauses of BAR-012's falsifier (a pin per job, in full, at or above the
floor; the floor's build and test on Linux; no default or default-host
change; no GNU Windows toolchain), with its own tests extended and the
mechanism rebound on today's workflow as the failing example.

Done when the workflow's run for the final commit is green on the three
hosted runners, the three test machines still build the crate with their
default toolchains, and the quality bar passes.
