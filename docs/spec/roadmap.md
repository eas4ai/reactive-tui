# Roadmap

Current: pie-labels-free-rows

Order agreed with the developer on 2026-09-21: charts first on a cell canvas,
then a general graphics canvas over wgpu that replaces the rasterizer
underneath, then the remaining widget families measured against gpui-kit.
The quality bar (BAR) applies to every commitment. On 2026-09-22 the
developer ruled that the SuprTUI renderer plan lands before the radial
charts, so filled shapes are drawn once on the new blitters.

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

## graphics-canvas

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
one. Requirements to be drafted when this commitment is next.

## widget-parity

Remaining widget families measured against gpui-kit 0.6.6 (docs/recon.md
section 13). Requirements to be drafted when this commitment is next.
