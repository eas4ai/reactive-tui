# Graphics canvas

Crate modules: `graphics`

## Purpose

The `wgpu-graphics` feature adds the `Canvas` widget. A canvas draws a scene
that the application describes: paths, paint, images, text and cell grids.
It draws on a hardware GPU when the host has one and on a software renderer
otherwise. It shows the picture as Kitty graphics, as Sixel, or as block
glyphs, whichever the terminal takes.

The default build does not compile wgpu. Rust 1.91 remains the minimum. The
canvas opens no window and replaces no backend: it is a widget in the
Element tree.

```sh
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion
cargo run --locked --features wgpu-graphics --example animation_showcase
```

The catalog's Motion page draws a lit cube. The showcase's Shader page
(press `6`) draws a lit torus. Both are canvas scenes. The gallery shows the
scenes the canvas's checks draw, one for each feature; the arrow keys step
through them:

```sh
cargo run --locked --features wgpu-graphics --example canvas_gallery
```

## Canvas widget

Build a `Scene`, wrap it in `CanvasProps`, and place the canvas like any
other widget. The canvas fills the area its parent gives it, up to 4096 by
4096 pixels.

```rust
use reactive_tui::component::Element;
use reactive_tui::graphics::{
    Canvas, CanvasProps, Color, GradientStop, Paint, Path, PathBuilder, Scene, Stroke, Transform,
};
use std::sync::Arc;

let mut scene = Scene::new();
scene.fill(
    &Path::rect(0.0, 0.0, 320.0, 192.0),
    &Paint::linear(
        (0.0, 0.0),
        (320.0, 0.0),
        vec![
            GradientStop::new(0.0, Color::token("blue-500")),
            GradientStop::new(1.0, Color::rgba(10, 10, 40, 255)),
        ],
    ),
);
scene.push_transform(Transform::translate(160.0, 96.0));
scene.stroke(
    &PathBuilder::new()
        .move_to(-60.0, 40.0)
        .quad_to(0.0, -80.0, 60.0, 40.0)
        .build(),
    &Stroke::new(3.0),
    &Paint::solid(Color::rgba(255, 255, 255, 255)),
);
scene.pop_transform();
scene.text(
    (12.0, 180.0),
    14.0,
    "drawn on the canvas",
    &Paint::solid(Color::token("primary")),
);
let canvas = Element::typed::<Canvas>(CanvasProps::new(Arc::new(scene)).view(320.0, 192.0));
```

Scene coordinates are pixels from the top left corner. One terminal cell is
as many pixels as the terminal reports, and 8 by 16 where it reports none.
With `.view(width, height)` the scene is drawn for a picture of that size:
the canvas scales it to fit its area, keeps its shape and centres it.
`.label(text)` names the picture for a screen reader, which also hears
which renderer draws it.

A scene is a list of drawing commands in painting order:

| Command | What it draws |
| --- | --- |
| `.fill(path, paint)` | The inside of a path, by the non-zero winding rule. |
| `.stroke(path, stroke, paint)` | The outline of a path. A `Stroke` has a width, a join (`Miter`, `Round`, `Bevel`), a cap (`Butt`, `Round`, `Square`) and an optional dash pattern. |
| `.image(rect, image)` | A `CanvasImage` scaled smoothly into a rectangle. |
| `.text(origin, size, text, paint)` | One line of text from its baseline. |
| `.cells(origin, grid, cell)` | A `CellGrid`, each cell `cell` pixels. The GPU draws the whole grid in one instanced draw from its glyph atlas. |
| `.push_transform(transform)` and `.pop_transform()` | Move, turn or scale what is drawn between them. Transforms nest. |
| `.push_clip(path)` and `.pop_clip()` | Show what is drawn between them only inside the path. Clips nest. |

A `Path` is made of lines, quadratic and cubic curves and elliptical arcs
(`PathBuilder`), or is a ready shape: `Path::rect`, `Path::rounded_rect`,
`Path::ellipse`. A `Paint` is one color, a linear gradient or a radial
gradient, with an opacity. A `Color` is exact (`Color::rgba`) or a token the
layout classes accept (`Color::token`), such as `primary`, `blue-500` or a
hex color. A token takes the active theme's color when the scene is drawn.

### Text and fonts

Text is drawn in the font the application supplies through
`GraphicsOptions::font`. Without one, on Linux the canvas takes the first
font file of `fc-match -s monospace` that it can read. Where that file is a
collection of fonts (`.ttc`), it draws in the face fontconfig matched. On
other systems, and when no such file can be read, it uses DejaVu Sans Mono,
which the crate bundles. `FontSource::Bundled` selects the bundled font on
every system, so a picture is the same everywhere. A font file the
application names with `FontSource::File` is read as its first face.

The canvas reads the font's outlines with the `skrifa` crate and draws
them with its own rasterizer, without hinting. It does no shaping: each
character is one glyph, placed by its advance.

### How the picture reaches the terminal

| The terminal takes | The canvas sends |
| --- | --- |
| Kitty graphics | Pixels through the Kitty protocol. When the terminal runs on the same machine and read the startup query, each picture travels through POSIX shared memory. |
| Sixel, not Kitty | Pixels as Sixel, drawn from the cursor in whole bands of six pixel rows. |
| Neither | Block glyphs, drawn with the blitter image fallback uses. |

On Unix the backend asks the terminal at startup what it takes. See
[Events, focus and input](events-focus-and-input.md). On Windows the
backend asks no terminal, and a Kitty picture is always sent in the command
itself: Windows has no POSIX shared memory.

A new picture replaces the one before it where it is. The canvas does not
clear the screen and writes no cell outside its area. The App writes one
frame at a time, so when the terminal is slower than the renderer, the
pictures in between are dropped and none waits in a queue.

To choose the output yourself, set `GraphicsOptions::output` to
`CanvasOutput::Kitty`, `CanvasOutput::Sixel` or `CanvasOutput::Blocks`. The
environment variable `REACTIVE_TUI_CANVAS` (`kitty`, `sixel` or `blocks`)
wins over both, so a user can correct a terminal that reports what it cannot
show. The variable is read once, when the first canvas is drawn. A value
that names none of the three is logged as a warning and changes nothing.

## Renderers and faults

The canvas draws on a hardware adapter (Vulkan, Metal or DX12; a discrete
GPU before an integrated one) whenever one gives a device. A software
adapter such as WARP or lavapipe does not count. Without a hardware adapter
the canvas draws on its software renderer, which draws every feature.

Both renderers start from the same shapes and compute coverage the same
way, so they draw the same picture. The check allows a mean difference of 1
of 255 per channel and no 8 by 16 pixel block that differs by more than 8 of
255.

The canvas never ends the App over a fault. When the adapter cannot be
used, the device is lost or reading a picture back fails, the canvas
switches to the software renderer for the rest of its life. When the
software renderer fails too, the canvas shows a message in its own area.

`GraphicsMode::label` names the renderer: `GPU · <adapter> · <backend>`, or
`CPU fallback · <reason>`. The demos show it above the picture. Select the
software renderer or inject a fault with:

```sh
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --cpu
cargo run --locked --features wgpu-graphics --example animation_showcase -- --cpu
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault adapter
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault device-loss
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault readback
cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion --graphics-fault software
```

The first three faults leave the software renderer to draw. The `software`
fault makes both renderers fail: the hardware renderer's first picture and
every picture of the software renderer, so the canvas shows its message.

## The worker

Each canvas draws on its own thread, named `rtui-canvas-` and a number. The
thread owns the adapter, the device and the software renderer. The App's
thread only submits the scene and shows the newest finished picture. It
never waits for the GPU or for a software render.

A scene submitted while the worker draws replaces the scene that still
waits. The worker waits for nothing between pictures: it draws as fast as
scenes arrive. When a picture is finished, the worker wakes the App.

On Windows the thread that makes a renderer asks the system not to slow it
down to save power. Windows otherwise moves a thread that waits for the GPU
to the processor's efficiency cores, where a picture takes more than twice
as long. A canvas that animates therefore keeps one thread on the
performance cores; a canvas that does not animate draws nothing.

A graphics driver may print to the terminal while it starts. To keep that
off the App's screen, start the worker before the terminal is set up and
hand it to the canvas:

```rust
use reactive_tui::graphics::{GraphicsOptions, GraphicsWorker};
use std::sync::Arc;
use std::time::Duration;

let worker = Arc::new(GraphicsWorker::spawn(GraphicsOptions::default())?);
worker.wait_ready(Duration::from_secs(10));
// Set the terminal up and start the App here. Give each canvas its
// worker with CanvasProps::worker.
```

One worker serves one canvas. Both demos start theirs this way.

## Limits

- A picture is at most 4096 by 4096 pixels. A larger area is drawn up to
  that size in whole cells, from its top left corner.
- A Kitty picture that is sent in the command itself, because the terminal
  does not take shared memory, is at most 12 million pixels, so that a
  frame's output holds it.
- When the operating system refuses the shared memory for a picture, for
  example because it is full, that one picture is sent in the command
  itself. A picture of more than about 12 million pixels does not fit the
  frame's output then, and the canvas shows the reason in its own area.
- A frame holds 64 MiB of new pictures and 64 MiB of output. When a
  canvas's picture does not fit, the frame is shown without it and the
  canvas shows the reason in its own area, until its picture has another
  size.
- An image is at most 4096 by 4096 pixels.
- A gradient keeps its first 16 stops.
- A dash pattern draws at most 65 536 dashes on one line. A line that would
  take more is drawn solid. So is a line whose pattern's lengths add up to
  less than a quarter of a pixel of the picture, as a pattern of zeros is.
- Text up to 96 pixels tall under a transform that only moves it is drawn
  from glyph bitmaps on whole pixels. Other text is drawn as filled
  outlines.
- A cell grid moves with the transform in force and stretches with it along
  the axes, its cells rounded to whole pixels. It does not turn. Block,
  quadrant, sextant, octant and braille glyphs are drawn as the shapes they
  name. Box drawing is stretched to the cell. Other glyphs are fitted inside
  the cell.
- Waiting for the adapter, the device or a finished picture has a deadline
  of five seconds. A driver call that hangs inside the operating system
  cannot be interrupted safely.
- Sixel has no partial transparency. The canvas blends its picture with the
  background color of the cells below it.

## Measure the renderers

Set the terminal to the requested size, then run each mode. Reports are
JSON. `--report NEW_FILE` refuses to overwrite an existing file.

```sh
cargo run --locked --features wgpu-graphics --example wgpu_benchmark -- --columns 144 --rows 50 --seconds 1 --report gpu-144x50.json
cargo run --locked --features wgpu-graphics --example wgpu_benchmark -- --columns 144 --rows 50 --seconds 1 --cpu --report cpu-144x50.json
```

The command draws the demo cube as block glyphs and presents it, as fast as
the loop runs. Sampling accepts 0.25 to 30 seconds and excludes startup and
the first picture. The comparison needs a hardware GPU in both modes,
because it also compares one picture from each renderer. A GPU failure
fails the GPU measurement.

A report names the adapter, the terminal environment, the size, the mode,
the frame count, the sampled time and the frames per second. It gives the
mean time to draw the block glyphs, to present them, and in total.
Presentation ends when the bytes are written, not when the terminal has
shown them. Results vary with the build profile, the host, the driver and
the machine's load.

The captures of the catalog in a private Kitty host can be made again
without touching desktop windows. They need `kitty`, `Xvfb` and
`/usr/bin/import` (ImageMagick built with X11 support), and the hardware
Vulkan driver:

```sh
cargo build --locked --features wgpu-graphics --example widget_catalog --example wgpu_benchmark
python3 -B scripts/check-wgpu-host.py --output /tmp/reactive-canvas-captures-NEW
```

Use a new directory each time. `CARGO_TARGET_DIR` is honored. The harness
creates and stops only its own display and Kitty processes. Look at the
screenshots: a passing report alone does not show that the picture is
right.

## Source map

- Scene model: [`src/graphics/scene.rs`](../src/graphics/scene.rs)
- Widget: [`src/graphics/widget.rs`](../src/graphics/widget.rs)
- Worker: [`src/graphics/worker.rs`](../src/graphics/worker.rs)
- Renderer choice and faults: [`src/graphics/hybrid.rs`](../src/graphics/hybrid.rs)
- Hardware renderer and its shader: [`src/graphics/gpu.rs`](../src/graphics/gpu.rs),
  [`src/graphics/canvas.wgsl`](../src/graphics/canvas.wgsl)
- Software renderer: [`src/graphics/cpu.rs`](../src/graphics/cpu.rs)
- Fonts and glyphs: [`src/graphics/fonts.rs`](../src/graphics/fonts.rs),
  [`src/graphics/glyphs.rs`](../src/graphics/glyphs.rs)
- Output choice: [`src/graphics/output.rs`](../src/graphics/output.rs)
- Scene, worker and fault tests: [`tests/canvas_scenes.rs`](../tests/canvas_scenes.rs)
- Output tests: [`tests/canvas_output.rs`](../tests/canvas_output.rs)
- Renderer comparison on each host: [`tests/canvas_hosts.rs`](../tests/canvas_hosts.rs)
- Gallery of the reference scenes: [`examples/canvas_gallery.rs`](../examples/canvas_gallery.rs)
- Measurement command: [`examples/wgpu_benchmark.rs`](../examples/wgpu_benchmark.rs)

## Related chapters

- [Rendering and backends](rendering-and-backends.md)
- [Images and clipboard](images-and-clipboard.md)
- [Animation and screens](animation-and-screens.md)
- [Applications and components](app-and-components.md)

[Back to the manual](README.md)
