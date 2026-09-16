# Optional wgpu graphics

Status: Draft 2026-09-16
Prefix: GPU

The next iteration adds an optional offscreen graphics canvas, demonstrated
by a shaded spinning cube in the widget catalog. It still runs inside the
host terminal. The first presentation path converts rendered pixels into
colored half-block terminal cells through the existing frame renderer.
Terminal image protocols are not required for this first increment.

[GPU-001]
The graphics layer MUST be enabled by an opt-in Cargo feature.
The default dependency graph MUST exclude wgpu.
Existing public APIs MUST remain backward-compatible.
The dependency selection MUST preserve the repository's declared Rust minimum.
Falsifier: A default build compiles wgpu; an existing consumer requires source
changes; or an enabled-feature build fails on the declared minimum toolchain.
Mechanism: Locked default and feature-enabled builds, dependency-tree
inspection, and existing consumer compatibility tests on the minimum toolchain.

[GPU-002]
The graphics canvas MUST render a shaded cube to an offscreen wgpu texture.
The canvas MUST present the texture's pixels through the existing terminal
frame path without opening another window.
The graphics area MUST use its available widget viewport rather than a fixed
40-column canvas.
Resizing MUST update the render target to the new viewport.
Projection MUST account for the terminal cell aspect ratio.
Falsifier: GPU mode bypasses wgpu; the cube is only a text placeholder; a
separate window opens; or resizing leaves clipped, stretched, or stale output.
Mechanism: A real-adapter smoke check records adapter identity and rendered
pixel samples. Frame tests inspect composition at 60x24, 144x50, and 200x60.
A host-terminal capture verifies readable edges and aspect after resizing.

[GPU-003]
Cube rotation MUST advance from elapsed time without keyboard input.
Rendering MUST use a bounded scheduled cadence rather than busy-spinning.
The renderer MUST limit work to one active frame and one replaceable pending
frame so missed deadlines cannot build a backlog.
Allocation MUST use checked dimensions subject to documented finite limits.
Falsifier: Rotation depends on input or frame count; missed deadlines create
an accumulating queue; the idle loop spins; or oversized dimensions overflow
or trigger an allocation beyond the documented limit.
Mechanism: Deterministic clock tests compare rotation after different frame
delivery schedules. Slow-render and oversized-viewport fixtures verify queue
bounds and allocation rejection. A live smoke check observes distinct frames.

[GPU-004]
GPU initialization, device-loss, or readback failure MUST select a CPU cube fallback
without terminating the catalog.
The demo MUST visibly identify the selected rendering mode.
Shutdown MUST cancel pending graphics work through the normal App lifecycle.
Ctrl+Q, Ctrl+C, and Escape MUST restore the host terminal on exit.
Falsifier: An unavailable adapter crashes the example; fallback is labeled
as GPU rendering; graphics work survives shutdown; or a quit key leaves the
terminal in raw mode or on the alternate screen.
Mechanism: Injected adapter, device-loss, and readback failures exercise the
fallback. Separate real PTY runs verify every advertised quit key and cleanup.
GPU acceptance requires a real GPU run; a software adapter or fallback run
is recorded separately and cannot substitute for that evidence.

[GPU-005]
The repository MUST provide a reproducible GPU-versus-CPU measurement command.
The report MUST identify the adapter, terminal, viewport, sampling duration,
rendering mode, achieved frame rate, and total frame latency.
The report MUST state any output-quality differences between GPU and CPU modes.
The report MUST separately measure rendering, readback, pixel-to-cell
conversion, and terminal presentation costs.
Runnable documentation MUST explain the feature, fallback, and verified hosts.
Falsifier: The comparison cannot be rerun; readback costs are hidden; fallback
results are reported as GPU results; or documentation claims untested support.
Mechanism: A bounded benchmark records both modes at the three viewport sizes
above. Review checks commands and measurements against captured output.
No 30 or 60 FPS guarantee is made before measurement.

## Reference

The local OpenTUI shader-cube example and Three renderer provide guidance for
offscreen rendering, elapsed-time animation, resize, cell-aspect correction,
and separate rendering/readback timing. This is a Rust integration with wgpu,
not a port of Three.js or a dependency on OpenTUI.
