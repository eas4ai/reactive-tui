commit: 922db00f329f7dd9b07a33cbc23896187c2391d4
examined:
  - GPU-001 through GPU-005, the commitment, implementation plan, and measurement decision.
  - Offscreen shader, checked dimensions, frame conversion, CPU fallback, canvas ownership, and worker shutdown.
  - Five fresh Cairn receipts, violating/corrected validator fixtures, real-adapter pixels, seven PTY runs, and six native-host measurements.
  - Kitty screenshots at 60x24, 144x50, 200x60, and shrink-back 60x24; manual commands and platform limits.
  - Public raw renderer concurrency and adapter labeling, which the serial catalog path does not exercise.
findings:
  - resolved: GPU-002 Commit 2cd4e131 gives each shared-reference render its own uniform buffer and bind group. Inspection confirms no frame can overwrite another frame's time/aspect; the 64-frame concurrent hardware regression passes in renewed GPU-002 evidence.
  - resolved: GPU-004 Commit 2cd4e131 labels nonhardware adapters Software wgpu and corrects the variant documentation. The metadata-only label regression passes; hardware acceptance still rejects software adapters.
  - open: GPU-005 The newly inspected shrink-back PNG shows Starting graphics and Preparing viewport even though the host predicate passed. A single accepted text sample can observe stale pre-resize contents after geometry changes, before the application redraw. Strengthen capture synchronization with stable accepted samples and post-capture validation, and demonstrate that a valid/loading/valid resize sequence cannot pass prematurely. GPU-002 shrink-back visual acceptance remains unproven by this artifact.

## Evidence and falsifier attacks

Fresh receipts are GPU-001 20260916T201855295Z-557057, GPU-002
20260916T201939105Z-565133, GPU-003 20260916T201957407Z-569439,
GPU-004 20260916T202016072Z-572456, and GPU-005
20260916T202101432Z-578311. These are passing mechanisms, not a substitute
for resolving the two public-API findings above.

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
during either review. The new capture finding requires separate implementation.

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
