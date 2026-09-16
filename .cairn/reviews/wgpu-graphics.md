commit: 44022a653d82a1646a66c24eda721d9872cc925f
examined:
  - GPU-001 through GPU-005, the commitment, implementation plan, and measurement decision.
  - Offscreen shader, checked dimensions, frame conversion, CPU fallback, canvas ownership, and worker shutdown.
  - Five fresh Cairn receipts, violating/corrected validator fixtures, real-adapter pixels, seven PTY runs, and six native-host measurements.
  - Kitty screenshots at 60x24, 144x50, 200x60, and shrink-back 60x24; manual commands and platform limits.
  - Public raw renderer concurrency and adapter labeling, which the serial catalog path does not exercise.
findings:
  - resolved: GPU-002 Commit 2cd4e131 gives each shared-reference render its own uniform buffer and bind group. Inspection confirms no frame can overwrite another frame's time/aspect; the 64-frame concurrent hardware regression passes in renewed GPU-002 evidence.
  - resolved: GPU-004 Commit 2cd4e131 labels nonhardware adapters Software wgpu and corrects the variant documentation. The metadata-only label regression passes; hardware acceptance still rejects software adapters.
  - resolved: GPU-005 Commit 8af7c18e requires three consecutive complete host samples and immediate post-capture validation. A stale-valid/loading/valid fixture fails before the fix and passes after it; a separate fixture rejects incomplete post-capture state. All four fresh formal PNGs were inspected and show complete chrome and resized cubes. Text predicates still cannot prove coherent host pixels; visual review remains mandatory, and the earlier loading PNG is not accepted evidence.

## Evidence and falsifier attacks

Initial closing-review receipts were GPU-001 20260916T201855295Z-557057, GPU-002
20260916T201939105Z-565133, GPU-003 20260916T201957407Z-569439,
GPU-004 20260916T202016072Z-572456, and GPU-005
20260916T202101432Z-578311. These are passing mechanisms, not a substitute
for resolving the two public-API findings recorded at that review.

Dependency fixtures fail on missing/implicit/default-enabled wgpu and missing
native features. Rendering fixtures fail on missing GPU operations, placeholder
pixels, absent shading/aspect, and new-window paths. Actual R9700/Vulkan pixels
and exact native cell-color comparisons cover all three viewport sizes.
Clock/worker attacks include equal elapsed times under irregular delivery,
1000 requests behind a blocked render (999 replacements), oversized dimensions,
Duration::MAX exhaustion, and the original independent-axis wrap seam.
Lifecycle fixtures reject detached ownership and mislabeled CPU output. Actual
adapter, destroyed-device, and readback injections persist in CPU mode. Seven
PTY runs observe animation without input, resize, normal exits, restored
termios/alternate screen/cursor, and no surviving owned process group.
Report fixtures reject omitted metadata/stages/quality, nonfinite costs,
hidden readback, software hardware claims, mislabeled CPU, and false FPS.

## Named host visual inspection

Inspected all four PNGs in
.cairn/evidence/captures/wgpu/20260916T202103249972Z-578460/.
Kitty 0.45.0 runs in an owned Xvfb X11 display with DejaVu Sans Mono 12 pt.
The cube uses actual R9700 Vulkan hardware; Kitty glyph rendering is software
OpenGL in this isolated setup. Header, navigation, mode, footer, and stage fit
at each size and after shrinking back. Edges are readable colored cell steps,
not pixel-resolution image output. Face-on snapshots are consistent with
rotation; other angles show cyan/purple/orange shaded faces. There is no fixed
40-column stage or clipped/stale cube after resizing. The initial 60-column
capture exposed RADV startup stderr scrolling the raw screen; pre-terminal
initialization fixed it, and the strengthened host predicate then passed.
Native Wayland desktop Kitty and other platforms/hosts remain unverified.
Paused CAT-001/CAT-002 layout/wireframe findings remain open; this review does
not close them or finish the release.

## Measurement and production-standard audit

The six one-second debug samples record real adapter and terminal identity,
duration, viewport, actual mode, achieved unpaced FPS, and average total
latency. Draw completion precedes copy/map/de-padding. Conversion and native
backend layout/paint/ANSI completion are timed separately. Initialization,
App reconciliation, host acknowledgement, and display scanout are excluded
explicitly. GPU-vs-CPU output differs by at most 1/255 per RGB channel in the
fixed-time samples. Terminal presentation dominates wider frames; the
measurements do not establish a universal GPU speedup or 30/60 FPS guarantee.

Formatting, both manual checks, nineteen selected integration targets, and
focused Clippy pass. The strict project check still has existing vendor/wizard
lints recorded outside this commitment. Clippy's incremental metadata crash
was avoided by rerunning without incremental compilation/compiler wrapper.
Ripwire exposed excessive renderer complexity and a duplicate command wrapper;
the stage separation and inline checker commands address them. Its remaining
churn, framework-invoked test reachability, feature-gated API reachability, and
owned-process cleanup similarity with archived scratch scripts are reviewed
tradeoffs, not claimed clean gates. GitNexus change detection ran before each
commit; actual staged paths were checked because its README symbol mapping
also names unrelated README nodes. No unrelated source file was staged.

Reviewed scope, API compatibility, errors, bounds, ownership, secrets, and
generated evidence against all fourteen production rules. No auth, migration,
network asset, or persistence subsystem is added. Reports refuse overwrite,
host artifacts use fresh directories, and the harness controls only its own
display/socket/processes. Driver calls hung inside the OS cannot be safely
preempted by a Rust thread; the documented deadlines/cancellation bound the
ordinary poll path, not arbitrary driver hangs. No executable code changed
during the reviews. Each finding was repaired in a separate implementation action.

## Review after raw-renderer repairs

Fresh receipts are GPU-001 20260916T202522020Z-613197, GPU-002
20260916T202819212Z-639208, GPU-003 20260916T202837684Z-642626,
GPU-004 20260916T202854110Z-645601, and GPU-005
20260916T202926558Z-651173. All mechanisms pass, including the two new
public-API regressions. These passes do not resolve the capture finding.

Inspected all four PNGs in
.cairn/evidence/captures/wgpu/20260916T202928481421Z-651332/.
The first three show bounded shaded hardware output and intact chrome.
The fourth is a loading frame after shrinking back to 60x24, not a cube.
This supersedes any claim that this newer artifact set proves shrink-back
visual acceptance. The earlier set remains preserved, not edited.
Source inspection found only one accepted geometry/text sample before a
fixed 0.2-second delay and capture. Geometry and app redraw are asynchronous;
the old small-frame text can satisfy the predicate during the resize.

## Closing review after capture synchronization

Fresh committed receipts are GPU-001 20260916T203218675Z-676692,
GPU-002 20260916T203253001Z-683919, GPU-003
20260916T203309557Z-686668, GPU-004 20260916T203326406Z-689814,
and GPU-005 20260916T203358309Z-694994. All five pass. The host tests
exercise the previously premature acceptance and reject incomplete state
after capture. Inspection confirms transient errors reset the acceptance
counter, waits remain bounded, and no developer window or process is used.

Inspected all four committed PNGs in
.cairn/evidence/captures/wgpu/20260916T203400240389Z-695153/.
They show readable colored edges, complete header/navigation/mode/footer,
the full available stage, and no stale target after 60x24 -> 144x50 ->
200x60 -> 60x24. The shrink-back image shows three shaded faces, not loading.
A preceding local capture had partially painted text despite valid terminal
contents. It was rejected visually and is not acceptance evidence. Consecutive
text checks reduce the resize race but do not acknowledge host pixel painting;
manual inspection is still required, as the checker and manual state.

The six fresh debug comparisons yield about 33.6/34.5 FPS at 60x24,
10.9/10.0 at 144x50, and 7.7/7.6 at 200x60 for GPU/CPU respectively.
These are one-second unpaced samples, not a demo FPS guarantee. Terminal
presentation remains the largest stage. Fixed-time output differs by at
most 1/255 per channel. Hardware R9700/Vulkan acceptance does not establish
native desktop Wayland, other terminal hosts, or other-platform acceptance.

As an additional read-only review check, locked Rust 1.91 default library
tests passed: 1051 passed, zero failed, eight ignored. Fresh Ripwire checks
compare against the already committed HEAD, so their empty changed-symbol
test map does not replace the earlier integration obligations. Its only new
quality row is generated GitNexus index metadata, not production code.
The earlier source findings and remaining tool limits remain documented above.
The production-rule audit remains satisfied for this scoped increment; known
unrelated strict-lint findings and paused catalog repairs are not claimed fixed.
No executable code changed during the closing review; no graphics finding
remains open. This closes only the graphics review, not the catalog or release.
