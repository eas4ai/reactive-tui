# Reactive TUI

<table>
  <tr>
    <td width="220" align="center" valign="middle">
      <img src="manual/assets/logo.jpg" alt="Reactive TUI logo" width="180">
    </td>
    <td valign="middle">
      <a href="#overview">Overview</a> &nbsp;•&nbsp;
      <a href="#installation">Installation</a> &nbsp;•&nbsp;
      <a href="#quick-start">Quick start</a> &nbsp;•&nbsp;
      <a href="#framework">Framework</a> &nbsp;•&nbsp;
      <a href="#widgets">Widgets</a> &nbsp;•&nbsp;
      <a href="#features-and-platforms">Features and platforms</a> &nbsp;•&nbsp;
      <a href="#manual">Manual</a> &nbsp;•&nbsp;
      <a href="#examples">Examples</a> &nbsp;•&nbsp;
      <a href="#ffi-and-typescript">FFI and TypeScript</a> &nbsp;•&nbsp;
      <a href="#development">Development</a> &nbsp;•&nbsp;
      <a href="#contributing">Contributing</a> &nbsp;•&nbsp;
      <a href="#license">License</a>
    </td>
  </tr>
</table>

## Overview

Reactive TUI is a Rust framework for terminal applications. It combines retained
components, reactive state, CSS-like layout, routed input, accessible semantics,
images, dialogs, and owned terminal sessions.

Applications render complete, grapheme-aware cell frames. The primary SuprTUI
backend owns terminal setup, input, presentation, and restoration. A debug backend
renders into memory for deterministic tests.

The current checkout is a **pre-release 0.1.0 candidate**. It has not been tagged as
a public release. The remaining release work is tracked in the
[recovery roadmap](docs/spec/roadmap.md).

Main capabilities:

- Retained root components and keyed child components.
- Thread-safe signals, hooks, effects, scheduling, and wake-driven applications.
- Flex and grid layout through Taffy, plus utility classes and themes.
- Keyboard, mouse, paste, focus, capture, bubble, and custom event routing.
- Input, layout, display, menu, dialog, image, and terminal widgets.
- SuprTUI, Crossterm compatibility, and in-memory debug backends.
- Optional embedded Unix PTY sessions interpreted by libghostty.
- Optional offscreen wgpu graphics with a shaded CPU fallback.
- Optional C ABI and a TypeScript SDK.

## Installation

The workspace requires **Rust 1.91 or newer**. Clone the current source and build
with locked dependencies:

```sh
git clone https://github.com/eas4ai/reactive-tui.git
cd reactive-tui
cargo build --locked --jobs 8
```

Until the final crates.io release is tagged, applications can use a path dependency:

```toml
[dependencies]
reactive-tui = { path = "../reactive-tui" }
```

Optional tools depend on the features an application uses:

- Remote dialog validation and autocomplete require curl 8.4 or newer.
- Chafa and Viu are optional image-rendering fallbacks.
- `embedded-terminal` release builds require Zig 0.16.0.
- `simd` requires a nightly Rust compiler.
- The TypeScript SDK requires Node.js 20 or newer and a matching native library.

## Quick start

This example paints a terminal frame through SuprTUI and exits on Escape or Ctrl+C:

```rust,no_run
use reactive_tui::app::RootComponent;
use reactive_tui::backend::SuprTuiBackend;
use reactive_tui::prelude::*;

struct Root;

impl RootComponent for Root {
    fn render(&self) -> Element {
        div()
            .class("flex-col w-full h-full p-1 bg-blue-900")
            .text("Hello, Reactive TUI!")
            .build()
    }
}

fn main() -> Result<()> {
    App::builder()
        .backend(SuprTuiBackend::new()?)
        .root(Root)
        .build()?
        .run()
}
```

Run the supported rendering example from this repository:

```sh
cargo run --locked --example gradient_blocks
```

This example demonstrates color gradients and block rendering.

## Framework

| System | What it provides | Manual chapter |
| --- | --- | --- |
| Applications and components | App lifecycle, roots, retained components, props, and keyed identity | [Applications and components](manual/app-and-components.md) |
| Elements and virtual DOM | Element builders, macros, keyed nodes, diffing, and patches | [Elements, builders, and the virtual DOM](manual/elements-builders-and-vdom.md) |
| Reactive state | Signals, refs, hooks, effects, timers, schedulers, and application wakeups | [Reactive state and hooks](manual/reactive-state-and-hooks.md) |
| Layout and style | Flex, grid, utility classes, colors, gradients, borders, and themes | [Layout, style, and themes](manual/layout-style-and-themes.md) |
| Events and focus | Routed keyboard and mouse input, hit testing, focus, capture, and bubble | [Events, focus, and input](manual/events-focus-and-input.md) |
| Rendering | Cell surfaces, render trees, reconciliation, terminal capabilities, and backends | [Rendering and backends](manual/rendering-and-backends.md) |
| Animation and screens | Easing, keyframes, springs, timelines, transitions, and navigation | [Animation and screens](manual/animation-and-screens.md) |
| Text systems | Editors, gap buffers, Markdown, and incremental syntax highlighting | [Text editing, Markdown, and syntax](manual/text-editing-markdown-and-syntax.md) |
| Accessibility | AccessKit semantics and an owned Linux AT-SPI connection | [Accessibility](manual/accessibility.md) |
| Native integration | C ABI ownership rules and TypeScript wrappers | [FFI and TypeScript](manual/ffi-and-typescript.md) |

`App` owns the main lifecycle. A root component produces an element tree. The
framework reconciles that tree, computes layout, paints cells, presents the frame,
and routes host input back to the retained application state.

## Widgets

The widget library is grouped by purpose:

- **Input:** text input, checkbox, radio button, select, and slider.
- **Layout:** accordion, breadcrumb, scroll view, stack, and tabs.
- **Display:** charts, tables, data tables, trees, file explorer, progress,
  modals, popovers, and images.
- **Menus:** menu bars, context menus, popup menus, actions, shortcuts, and themes.
- **Dialogs:** confirmation, input, autocomplete, progress, toast, and wizard flows.
- **Terminal:** an owned child process with PTY input, resize, scrolling, focus,
  screen state, and error reporting.

Start with the [input](manual/input-widgets.md),
[layout](manual/layout-widgets.md), [display](manual/display-widgets.md),
[menu](manual/menus.md), [dialog](manual/dialogs.md), and
[terminal widget](manual/terminal-widget.md) chapters.

## Features and platforms

### Cargo features

| Feature | Behavior |
| --- | --- |
| `default` | Enables Tokio support. |
| `async-capabilities` | Enables asynchronous capability detection through Tokio. |
| `ffi` | Exports the C ABI and builds the native integration surface. |
| `embedded-terminal` | Enables the libghostty-based embedded session on Unix. |
| `simd` | Enables nightly `portable_simd` color paths. |
| `wgpu-graphics` | Opt-in offscreen GPU canvas, with shaded CPU fallback. |
| `debug` | Enables general diagnostic paths. |
| `debug_patches` | Enables reconcile patch diagnostics. |

### Platform behavior

- SuprTUI and Crossterm provide the normal terminal-host paths.
- The legacy pseudo-terminal API has Unix and Windows implementations. Windows
  uses the bundled OpenConsole/ConPTY runtime.
- The libghostty embedded-session API is available only on Unix.
- AccessKit semantics are cross-platform data. The owned AT-SPI connection is
  Linux-specific and requires a compatible desktop session.
- Image output selects a supported terminal protocol, Chafa or Viu, or a cell
  fallback.
- Supported-platform release CI and final cross-platform evidence remain part of
  the pre-release roadmap.

The framework bounds external work. Remote dialogs use explicit configured
endpoints, HTTP or HTTPS only, a five-second deadline, a 1 MiB request limit, and a
64 KiB response limit. Static image decoding has a 256 MiB total memory budget.
File-explorer copy and delete operations reject an entry whose filesystem identity
changes after inspection. See [Dialogs](manual/dialogs.md),
[Images and clipboard](manual/images-and-clipboard.md), and
[Display widgets](manual/display-widgets.md) for the exact rules.

## Manual

The [Reactive TUI manual](manual/README.md) is the main documentation entry point.
The [supported-API matrix](manual/supported-api.md) maps every public crate module
to its supported route, behavior checks, and verification limits.
It contains a linked overview and 20 focused chapters grounded in the exported
source, Cargo features, examples, and behavior tests.

Recommended starting points:

- [Getting started](manual/getting-started.md)
- [Applications and components](manual/app-and-components.md)
- [Reactive state and hooks](manual/reactive-state-and-hooks.md)
- [Layout, style, and themes](manual/layout-style-and-themes.md)
- [Events, focus, and input](manual/events-focus-and-input.md)
- [Rendering and backends](manual/rendering-and-backends.md)
- [Terminal and embedded sessions](manual/terminal-and-embedded-sessions.md)

## Examples

| Example | Command | Purpose |
| --- | --- | --- |
| Widget catalog | `cargo run --locked --example widget_catalog` | Responsive live widget pages, local project logo, and a spinning wireframe cube for screenshots and video. |
| Gradient blocks | `cargo run --locked --example gradient_blocks` | Color gradients and block rendering. |

For the optional shaded GPU cube, run
`cargo run --locked --features wgpu-graphics --example widget_catalog -- --motion`.
The visible mode identifies hardware rendering or CPU fallback. See
[Offscreen graphics](manual/wgpu-graphics.md) for failure options, finite limits,
verified host details, and reproducible GPU-versus-CPU measurements.

The catalog uses arrows or `1`–`8` to select pages and Tab to focus controls.
Use F1/F2 on the menus/dialogs page to capture one overlay demo at a time.
Mouse-wheel scrolling reveals longer widget pages.
It switches from a sidebar to compact navigation below 80 columns. Open page
`3` for a full-width colored column-span grid. Example cards use compact gaps
and switch between one and two columns with the terminal width. Open page
`7` to record the cube animation, or page `6` for the tracked project logo.
Ctrl+Q always quits. Ctrl+C and Escape quit when the active widget does not
consume them (for example, Escape first closes an open menu).

The system page runs only a bounded command. It does not claim to fix the
known Kitty embedded-shell crash. All catalog assets are local; no download
is needed.

## FFI and TypeScript

Enable `ffi` to build the Rust `cdylib` and C ABI:

```sh
cargo build --locked --features ffi --jobs 8
```

The public C header is [`include/reactive_tui.h`](include/reactive_tui.h). Opaque
handles have explicit ownership, error, callback, and release rules described in
the [FFI manual](manual/ffi-and-typescript.md).

The [TypeScript SDK](bindings/typescript/README.md) wraps the native ABI with typed
owners and builders. Build and verify it with Node.js 20 or newer:

```sh
cargo build --locked --features ffi --jobs 8
cd bindings/typescript
npm ci
npm run build
npm test
```

The SDK does not download a native library. Set `RTUI_LIBRARY_PATH` or package a
matching library with the TypeScript application.

## Development

Keep build concurrency at eight jobs on this repository:

```sh
cargo fmt --all -- --check
cargo test --locked --lib --jobs 8
cargo clippy --locked --package reactive-tui --lib --no-deps --jobs 8 -- -D warnings
python3 -B scripts/check-framework-manual.py MAN-001
python3 -B scripts/check-framework-manual.py MAN-002
```

Run build and test commands sequentially. Some optional features require their
platform toolchain or external program. Cairn specifications, commitments,
decisions, mechanisms, evidence, and reviews record the release contracts.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md), create a focused
branch from `main`, add behavior-focused tests, update the relevant manual chapter,
and keep public API documentation current.

Reactive TUI was created and is maintained by
[Shawn McAllister](https://github.com/entrepeneur4lyf).

## License

Reactive TUI is available under the [MIT License](LICENSE).

The project builds on Taffy for layout, Crossterm for terminal control, Comrak for
Markdown, Syntect for syntax highlighting, AccessKit for accessibility semantics,
and libghostty for embedded terminal interpretation.
