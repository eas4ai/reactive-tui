commit: 6f7a52d560e69b260c3b265c942e65edc8cdeb52
examined:
  - CAT-001 through CAT-003 and the widget-catalog commitment.
  - Catalog source, live widget inventory, navigation, cube clock, and quit lifecycle.
  - Behavior and validator tests, PTY output, and all three Cairn receipts.
  - README, manual, local logo, and mechanism dependency declarations.
findings:
  - resolved: CAT-001 fcb971c9bbd1b7a66cb8c8faf8898cff7ef33e2a configures full-width scrolling, compact navigation and card spacing, wrapped coverage, and colored column spans. Native background-run geometry and owned Kitty captures verify both example columns at 144x50 and 200x60 and compact layout at 60x24.
  - resolved: CAT-002 0ad385ee0eef6e9e8f25a042172aafbbf78fce28 replaces fixed dotted strokes with viewport-sized Braille subpixels. Native cell counts, elapsed-time tests, distinct PTY pixel hashes, three clean quits, and inspected Kitty grow/shrink captures verify readable bounded edges. The optional shaded wgpu stage remains intact.
  - resolved: CAT-001 cfea6a441757f68abd0058dc6577cae535ef3afe mounts live InputDialog and AutocompleteDialog. F1/F2 selects one of twelve overlay demos. Real App input paints both dialog prompts and both menu item lists.
  - resolved: CAT-001 cfea6a441757f68abd0058dc6577cae535ef3afe fixes navigation sizing and scrolls long pages. Measured frames retain all eight shortcuts after resizing to 100, 60, 79, 80, and 40 columns.
  - resolved: CAT-001 cfea6a441757f68abd0058dc6577cae535ef3afe populates Table with two columns and two rows, starts Modal visible, and supplies a Popover button trigger. Tests inspect representative Table props and decode the mounted logo asset.

## Scope examined

Compared CAT-001 through CAT-003 with the catalog source, tests, checker,
mechanism declarations, README, and manual. No framework API or dependency
changed. Reviewed the actual PTY output and terminal restoration assertions.
The original review found false confidence in inventory labels, clipped
navigation, an empty Table, and an unreachable Modal. The separate repair
commit addresses those findings without changing framework APIs or dependencies.
Fourteen behavior tests and eleven validator tests now pass. Fresh committed
receipts pass CAT-001, CAT-002, and CAT-003:
20260916T141202292Z-1567096, 20260916T141214789Z-1570303,
and 20260916T141227980Z-1573710.

## Mechanism attacks

Validator fixtures reject missing pages, the wrong/network logo, missing test
evidence, identical animation samples, failed quit sequences, and omitted
documentation. Three isolation fixtures prove unfinished requirements cannot
fail an unrelated requirement. The corrected shell fixture passes. A new fixture
retains the InputDialog inventory label but removes its live constructor; the
checker rejects it. Before the repair, new behavior tests failed on absent live
dialogs, empty Table props, and clipped shortcuts. Corrected cases pass. This
demonstrates real failures, not only successful receipts.

The real PTY check hashes only visible cube strokes and vertices, not changing
labels or ANSI output. It observes two distinct frames separated by at least
160 ms without input, then checks Ctrl+Q, Ctrl+C, and Escape in separate runs.
Each normal exit restores termios, alternate screen, cursor, and child lifetime.
Deterministic animation tests check an 80 ms deadline and skip missed frames.

## Final verification and limits

Measured DebugBackend frames cover navigation geometry; routed SuprTui App
input covers dialog switching at 100x32 and 60x24. Structural tests cover Table
rows, dialog mounts, and Image; the tracked JPEG decodes successfully. These
checks do not replace a visual capture session in Kitty or exercise every
possible state of every widget.

Formatting and diff whitespace checks pass. The library suite passes with
1051 passed and eight ignored, after an initial tiny clock-noise failure in an
unchanged stagger test; its isolated rerun also passes. All fourteen integration
targets named by Ripwire pass, totaling 563 tests. The timing flake is captured
in the backlog rather than silently changed outside this commitment.

Unrestricted strict Clippy fails on inherited bundled-crossterm warnings and
an unchanged wizard collapsible-else-if warning. Package-only Clippy with
no dependency lints and that single existing lint allowed passes. The inherited
warnings are recorded in the backlog; this is not a claim of strict lint health.

GitNexus CLI impact reports low risk for resolved catalog symbols and no affected
framework processes; its MCP transport is unavailable and the card helper target
was unresolved. Staged file inspection confirms only the expected example,
tests, checker, docs, backlog, and evidence changes. Rust-analyzer resolves the
saved motion module and its Instant type; no claim of a diagnostics sweep is made.

Ripwire quality-delta and test-gate do not pass. Their reports include repeated
Cairn commit churn, declarative setup length, test-only symbol reachability,
similar constructor tokens in unrelated state types, and ignored generated
GitNexus metadata. These are reviewed tradeoffs, not grounds for unrelated
abstraction or metric-only refactors. The test gate's named integration targets
were executed successfully; its graph report does not record test execution.

The Kitty embedded-shell crash remains explicitly outside this commitment.
The System page remains a bounded command, not a repaired interactive shell.
The release self-audit considered all fourteen production rules: scope and
contracts stay intact, assets remain local, there are no new secrets or
dependencies, cube work is bounded, and PTY cleanup has a deadline. Persistence,
auth, and migrations are not introduced. No further in-scope revision was found.
No executable code changed while this closing review was performed.

## Review after embedded-shell retention approval, 2026-09-16

Examined the 13 manual lines retained from da046fe3 and the developer's
approval in loop-035-2. The launcher instructions require the optional feature
and a Cargo rebuild, distinguish host quit from child input, and describe
control-text replacement without weakening frame validation. The historical
statement about an unproven Kitty segfault repair remains a limit, not evidence
that the shell currently crashes. The developer subsequently observed a usable
prompt and a normal GDB exit in Kitty.

Read fresh captured output for CAT-001, CAT-002, and CAT-003:
20260916T181612410Z-3434802, 20260916T181646035Z-3439519, and
20260916T181706765Z-3443419. Each ran eleven validator tests, fourteen behavior
tests, and a locked example build. CAT-002 additionally observed distinct cube
frames and exit status zero for all three advertised quit sequences.

Challenged what these mechanisms miss against the developer's screenshots.
Navigation breakpoint tests do not prove the stage uses the whole width.
Inventory and text-presence checks do not prove compact spacing or legible
wireframe edges. The open findings above supersede the earlier conclusion that
no further in-scope revision was needed. No executable code changed during
this review; passing receipts do not close these visual findings.

## Review on approved resumption

Shawn approved resumption after the graphics requirements passed and their
review was clean. The renewed catalog baselines pass: CAT-001
20260916T204844693Z-796920, CAT-002 20260916T204920229Z-801882,
and CAT-003 20260916T204947423Z-806472. They do not close the existing
visual findings. Source inspection confirms the outer convenience ScrollView
uses its default 80x24 props and wraps pages in a start-aligned Stack.
Sidebar entries have flex-1 despite h-1, so they grow with terminal height.
Page gap-1 uses the four-cell CSS spacing scale. ScrollView also supplies
whitespace-pre, so long coverage prose needs explicit wrapping.

Repair the catalog configuration, not the shared spacing scale or framework
API. Add regression checks for viewport-sized scrolling, full-width colored
backgrounds, one-row navigation entries, compact page spacing, and actual
colored grid-span geometry. The default cube still rasterizes dotted edges
into a fixed 40x16 canvas; its quality finding remains open independently of
the opt-in shaded wgpu page. No executable code changed during this review.

## CAT-001 visual repair review

Examined candidate 175fc950, its source, documentation, geometry tests, and fresh
receipts CAT-001 20260916T205954389Z-880277, CAT-002
20260916T210034678Z-889720, and CAT-003 20260916T210051530Z-892417.
Nineteen default tests pass. The separately executed feature-enabled suite
passes twenty-one tests; its graphics stage remains unchanged.

Attacked percentage widths inside grid tracks, class replacement, wrapped
coverage, fixed-size scroll defaults, stretched sidebar rows, and label-only
span checks. The regression tests failed on the old configurations and the
intermediate mistakes before passing on the correction. Background runs prove
the one-, two-, three-, and four-column cells have actual colored geometry.
Both wide example columns are visible, not merely present in an element tree.

Inspected the six corrected PNGs in local capture 20260916T205859937742Z-870701;
all six formal capture SHA-256 values in 20260916T205959702219Z-881809 match
those inspected images exactly. They show full available width, wrapped
TerminalWidget coverage, consecutive sidebar entries, compact gaps, correct
spans, both example columns, and intact header/footer. Longer compact pages
scroll rather than shrinking their controls. Host acceptance is limited to
owned Xvfb X11 Kitty 0.45, DejaVu Sans Mono 12 pt, software host glyph rendering;
this is not a claim about every font or terminal.

No framework API, dependency, or asset changed. Focused Clippy passes with the
same single inherited wizard lint allowed; unrestricted strict lint remains a
known limit. No executable code changed during this review. CAT-001 is closed;
the original coarse default wireframe remains the separate open CAT-002 finding.

## Closing review after the wireframe repair

Examined candidate 6f7a52d5, the approved CAT-001 through CAT-003 contract,
source, dependency declarations, documentation, and committed receipts:
CAT-001 20260916T210951304Z-990273, CAT-002 20260916T211025481Z-997460,
and CAT-003 20260916T211059625Z-1003895. All three pass against the same
declared-input digest 8a0f164653b048d8b75d7cc6ab9491fd16c0f5cc6a335e60f927fb3c74117004.
Each runs eleven validator tests, two host/pixel-selection tests, twenty-one
default behavior tests, and a locked example build. The separately executed
feature-enabled suite passes twenty-three tests.

Attacked zero and oversized viewports, pre-clamp arithmetic, allocation bounds,
resize without changing the timer deadline, multiline text measurement,
label-only animation hashes, stale screens after resizing, and host diagnostic
backpressure. The canvas clamps before allocation to 240x100. Twelve edges
use bounded Bresenham traversal and Braille masks; the compatibility canvas
stays 40x16. Native App observations establish full canvas cell counts; the
limited DebugBackend multiline path is not substituted for native proof.
The real PTY observes nonblank Braille pixel changes without input and checks
Ctrl+Q, Ctrl+C, and Escape separately, including termios, alternate screen,
cursor restoration, and absence of surviving owned children.

Inspected all eight formal Overview/Layout PNGs in
20260916T210952821244Z-990798 and all four formal Motion PNGs in
20260916T211027584545Z-997978. The same owned instance grows from 60x24 to
144x50 and 200x60, then shrinks to 60x24. Coverage wraps completely; spans
have correct widths; both wide example columns are visible; the cube remains
bounded with fine connected edges and intact controls. Compact long pages
retain scrolling. These visual findings are now closed, not inferred from
passing hashes alone.

The harness regression reproduced a real child blocking on its stderr pipe
before the fix and succeeding with a private diagnostic file afterward.
Kitty's existing mode-change warnings remain observable in the host manifests
and are captured in the backlog; the shared scanout warning is not claimed
fixed. Partial and experimental capture directories are diagnostics, not
acceptance evidence. Acceptance uses only the named committed receipts and
their complete capture manifests.

To challenge the shared Host change, built the feature-enabled example and ran
the existing wgpu capture-only harness. Inspected all four PNGs in
wgpu/catalog-regression-20260916T211150Z. They retain the shaded cube, full
canvas, truthful AMD Radeon AI PRO R9700 Vulkan label, and intact controls
through grow/shrink. This is a fresh integration smoke check, not new GPU
benchmark evidence or a refresh of all five earlier graphics receipts.

The fourteen Ripwire-named integration targets pass, as does the library suite
(1051 passed, eight ignored). The manual checker passes with twenty pages,
twenty-eight modules, six widget families, and eight Cargo features. Formatting,
whitespace checks, and focused Clippy pass; the one inherited wizard lint is
allowed explicitly. Unrestricted strict Clippy and Ripwire's quality gates are
not claimed clean. Reviewed Ripwire's constructor-token similarity, graph-only
test reachability, rasterizer complexity, and Cairn commit churn; no metric-only
abstraction or unrelated framework change is warranted.

GitNexus impacts were run before each existing symbol edit, including shared
Host methods; reports are low risk with incomplete Rust-method caller coverage.
Focused source and Ripwire inspection resolve the actual example callers, and
all private rasterizer signature dependents were updated. Staged detection and
file inspection confirm the approved example, tests, harnesses, records, docs,
and evidence scope. Rust-analyzer resolves the saved CubeAnimation definition.
No diagnostics-sweep claim is made.

The fourteen-rule self-audit finds no further in-scope revision: no framework
API or dependency changed, assets stay local, work and waits are bounded,
diagnostics remain actionable, ownership stays private, and documentation
matches the delivered example. No persistence, auth, or migration surface is
introduced. Host acceptance remains Kitty 0.45 on owned Xvfb X11 with DejaVu
Sans Mono 12 pt and software host glyph rendering; no 60 FPS or universal-host
claim is made. Embedded-shell and release work remain outside this commitment.
No executable code changed during this closing review.

## CAT-001 rewording review (spec lint repair)

Re-read revised CAT-001 and its falsifier against mechanism `widget-catalog`.
The revision splits one two-obligation sentence into two sentences with
identical meaning: catalog keys select pages, and resize events change the
navigation layout without restarting. The falsifier is unchanged in meaning.
The mechanism proves its validator rejects missing pages, bad assets,
compile-only evidence, identical frames, and lingering quit sequences, then
runs catalog behavior tests and locked example compile — so both revised
obligations remain covered.

Failure demonstration: ran the mechanism's validator suite
(`scripts/test-widget-catalog.py`), which feeds violating fixtures and
corrected cases. All 11 tests pass: violating cases are rejected and corrected
cases accepted. No mismatch found. No code changed during this review.

## CAT-002 rewording review (spec lint repair)

Re-read revised CAT-002 and its falsifier against mechanism
`widget-catalog-motion`. The revision splits one two-obligation sentence into
two sentences with identical meaning: animation advances without keyboard
input, and it requests redraws at a bounded cadence. The falsifier is
unchanged in meaning. The mechanism runs deterministic behavior tests, locked
example compile, and a real PTY check requiring distinct cube frames, bounded
redraws, three normal exits, restored settings, and no surviving children —
so both revised obligations remain covered.

Failure demonstration: the shared catalog validator suite
(`scripts/test-widget-catalog.py`, also run for CAT-001) feeds violating
fixtures and corrected cases; all 11 tests pass with violating cases rejected
and corrected cases accepted. No mismatch found. No code changed during this
review.
