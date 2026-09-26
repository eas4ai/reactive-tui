# Optional offscreen graphics

Crate modules: `graphics`

## Purpose

Enable `wgpu-graphics` to render a shaded, elapsed-time spinning cube in the
catalog's Motion page. The default build does not compile wgpu. Rust 1.91
remains the minimum. This is an offscreen texture, not another application
window or a replacement terminal backend. Pixels become colored `▀` cells
through SuprTUI; no terminal image protocol is required.

```sh
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion
```

The Motion canvas uses the whole available stage and changes its target on
resize. Two vertical pixels share a terminal cell, assuming a cell is twice
as tall as it is wide. Different font metrics can change the apparent aspect.
Cell-sized edge stair-stepping is expected; this is not pixel-resolution image
output. Other catalog layout repairs remain paused.

## Behavior

The visible label identifies the actual GPU adapter or CPU fallback and its
reason. Software adapters do not count as hardware acceptance. Initialization,
device-loss, and readback failures select the shaded CPU renderer rather than
terminating the catalog. Force CPU or reproduce failures with:

```sh
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --cpu
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault adapter
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault device-loss
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault readback
```

The catalog initializes the adapter before entering the raw terminal so driver
startup diagnostics cannot scroll its alternate screen. Frame rendering runs
on one owned worker: one active frame, one replaceable pending request, and
one replaceable output. Requests are capped at 20 Hz and use elapsed time;
missed deadlines are skipped. Shutdown cancels work, clears pending output,
and joins the worker. Ctrl+Q, Ctrl+C, and Escape restore the terminal on Motion.
Menus on other pages may consume Escape first.

## Limits

Dimensions are checked before allocation, with a finite limit of 800x600
pixels (800 columns and 300 canvas rows). Oversized or empty stages display
an error instead of allocating beyond that limit. Native initialization and
completion waits have five-second deadlines and short cancellation polls;
a driver call that hangs inside the operating system cannot be forcibly
interrupted safely by this thread-based layer.

## Main API

`graphics::GraphicsCanvas` attaches to an `AppWaker`; `advance` submits timed
viewport work, `element` composes ordinary Elements, and dropping its owned
worker cancels and joins. `with_renderer` accepts a `HybridCubeRenderer`
initialized before terminal setup. `new` instead initializes lazily on the
worker; hosts must account for driver startup diagnostics when using it.

## Reproduce the comparison

Set the terminal to the requested dimensions, then run each mode. Reports
are JSON. `--report NEW_FILE` refuses to overwrite an existing file.

```sh
cargo run --locked --features wgpu-graphics --example wgpu_benchmark -- --columns 144 --rows 50 --seconds 1 --report gpu-144x50.json
cargo run --locked --features wgpu-graphics --example wgpu_benchmark -- --columns 144 --rows 50 --seconds 1 --cpu --report cpu-144x50.json
```

Repeat with 60x24 and 200x60. Sampling accepts 0.25–30 seconds, excludes
initialization/warmup, and permits the final in-flight frame to finish. The
comparison requires a hardware GPU even for CPU sampling because it also
compares fixed-time output against real GPU pixels. A GPU failure fails the
GPU measurement; fallback cannot masquerade as GPU results.

Reports name the adapter, terminal environment, viewport, mode, frame count,
actual sampling duration, achieved unpaced FPS, and average total latency.
Separate wall-clock stages include drawing through GPU completion, texture
copy/map/de-padding, pixel-to-Element conversion, and native backend
layout/paint/ANSI writes. Explicit draw/copy synchronization is part of the
measured production path. Presentation ends at terminal-write completion,
not host acknowledgement or display scanout. App reconciliation is excluded.
This benchmark is unpaced; the live demo separately caps requests at 20 Hz.
No 30/60 FPS guarantee or general GPU speedup is claimed.

CPU and GPU use the same shaded ray-box, rotations, and sRGB output. Reports
measure differing pixels, mean absolute RGB error, and maximum channel error
at one second. Initial debug samples at all three sizes differed by at most
1/255 per channel; conversion/presentation dominated total frame costs.
Results vary with build profile, font, host, driver, and machine load.

## Verified host and captured evidence

Verified locally: Linux, Kitty 0.45.0, X11 in a private Xvfb display,
DejaVu Sans Mono 12 pt, and AMD Radeon AI PRO R9700 / RADV GFX1201 / Vulkan
for the cube. Kitty's own OpenGL glyph renderer used software rendering in
this isolated display; the cube's Vulkan adapter was real hardware.
Native desktop Wayland Kitty, other terminals, Windows, and macOS remain
unverified for this increment. Compiled native backend options are not host
acceptance claims.

The complete documented comparison and host captures can be rerun without
touching desktop windows. It needs `kitty`, `Xvfb`, and `/usr/bin/import`
(ImageMagick built with X11 support), plus the hardware Vulkan driver:

```sh
cargo build --locked --features wgpu-graphics --example widget_catalog --example wgpu_benchmark
python3 -B scripts/check-wgpu-host.py --output /tmp/reactive-wgpu-captures-NEW
```

Use a new directory each time. `CARGO_TARGET_DIR` is honored. The harness
creates and stops only its own display and Kitty processes. It records six
JSON measurements, three viewport screenshots plus a shrink-back capture, host details, and
artifact hashes. It keeps artifacts beneath
`target/evidence/captures/wgpu/`, which `cargo clean` removes. Inspect the screenshots;
passing report validation alone cannot establish visual quality.

## Source map

- Renderer and pixels: [`src/graphics/mod.rs`](../src/graphics/mod.rs)
- Owned canvas: [`src/graphics/canvas.rs`](../src/graphics/canvas.rs)
- Fallback: [`src/graphics/hybrid.rs`](../src/graphics/hybrid.rs)
- Real-adapter composition tests: [`tests/wgpu_graphics.rs`](../tests/wgpu_graphics.rs)
- Lifecycle tests: [`tests/wgpu_lifecycle.rs`](../tests/wgpu_lifecycle.rs)
- Measurement command: [`examples/wgpu_benchmark.rs`](../examples/wgpu_benchmark.rs)

## Related chapters

- [Rendering and backends](rendering-and-backends.md)
- [Animation and screens](animation-and-screens.md)
- [Applications and components](app-and-components.md)

[Back to the manual](README.md)
