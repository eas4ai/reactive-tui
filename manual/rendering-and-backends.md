# Rendering and backends

Crate modules: `backend`, `core`, `display`, `platform`, `render`, `ui`

## Purpose

The rendering system turns an element tree into terminal cells, sends changed
output to the host terminal, receives input, and reports frame and display
information.

## Main API

- `Backend` defines frame rendering, cell rendering, patch application,
  presentation, clearing, size, resize, and event polling.
- `SuprTuiBackend` is the primary terminal backend and supports cell frames and
  terminal image output.
- `CrosstermBackend` is the compatibility backend.
- `DebugBackend` renders to an in-memory `Surface` for deterministic tests.
- `RenderTree`, `Reconciler`, `PatchOp`, `DirtyRegion`, `RenderCache`, and
  `RenderScheduler` expose rendering internals for advanced use.
- Core types cover geometry, graphemes, styled text, surfaces, render
  operations, windows, writers, and statistics.
- Display types report terminal capabilities, connection type, frame rate, and
  adaptive performance settings.
- `Updater` connects external work to application updates.

## Basic use

Use `SuprTuiBackend` for a real terminal application. Use `DebugBackend` when a
test needs to inspect text, cells, frame count, or patches without controlling a
terminal. Implement `Backend` only when another host owns output and input.

`SuprTuiBackend`, and `CrosstermBackend` and `DirectTtyBackend`, which are
built on it, lay out and paint on their own renderer thread. That thread's
stack is large enough for the deepest tree the app accepts (128 levels).
`DebugBackend` lays out and paints on the thread that runs the app. In a debug
build, a tree 128 levels deep takes about 1.8 MiB of that thread's stack. That
is close to the 2 MiB a test thread has, and with the `embedded-terminal`
feature it is more than such a thread has left. Run a test that draws a tree
that deep through `DebugBackend` on a thread with a larger stack.

## Behavior

The app expands components and reconciles a render tree. The component bridge
builds layout nodes. The paint tree computes geometry and writes a complete
grapheme-aware cell frame. SuprTUI compares frames and writes the required ANSI
output. Presentation makes the frame geometry available to focus, events,
animation targets, and accessibility.

Components learn their layout after a frame is presented. After a terminal
resize the App first lays the frame out at the new size without painting it
(`Backend::layout_frame`), gives each component its new layout and each
dialog anchor its new bounds, and only then renders and presents, so the
first frame at the new size shows nothing at its old size. A backend that
cannot lay out ahead of a present returns `None` and keeps the old order;
backend wrappers should forward `layout_frame`.

The backend owns host terminal setup and restoration. Capability detection
selects color, synchronized output, keyboard, mouse, Unicode, and image behavior.
Performance monitors measure frame work and can lower adaptive quality.

A widget that rasterizes off the main thread hands the painter a prepared
grid instead of one text node per colored run: it fills a `CellGrid`
(`reactive_tui::layout::CellGrid`) with `.set(x, y, glyph, color)` and
attaches it to one element with `.with_cells(grid)`. The element keeps its
size from its styles; the painter blits the grid at the element's content
box, through the element's transform, clip and masks, using each cell's
color or the element's foreground. The App's cost per element is then paid
once for the whole grid, which is how a 700 by 200 chart stays under the
frame budget. The charts widget is the first user.

After an App or renderer panic, built-in terminal backends restore their owned
screen state and replay a bounded, control-encoded panic message through their
output writer. The original panic payload still propagates. Normal diagnostics
use logging. Backend wrappers should forward `shutdown_after_panic`; custom
backends default to shutdown and logging rather than writing to process streams.

## Limits

- A backend must keep its reported size consistent with submitted cell frames.
- Wide graphemes occupy a lead cell and a continuation cell.
- `present` returns a frame's geometry as soon as layout and painting finish;
  the render worker writes and flushes the bytes afterwards, with at most one
  frame in flight. A write or flush failure is reported by the next
  `present`, by `sync` or by `shutdown`, and forces a full repaint. Until the
  failure is reported, the backend reports the newest presented frame's
  geometry; from then on it reports the geometry of the last frame whose
  flush was acknowledged, so the failed frame is never used as the fallback
  for a later failure.
- Debug backend dimensions are limited to 65,535 cells per axis and 262,144
  total cells.
- Capability detection can be incomplete in redirected, remote, or unusual
  terminal environments.
- A cell grid paints through the SuprTUI frame painter; the legacy surface
  painter used by the debug backend ignores it, as it ignores images.

## Source map

- Cell grid: [`src/layout/paint_tree/cells.rs`](../src/layout/paint_tree/cells.rs)

- Backend contract and implementations: [`src/backend/mod.rs`](../src/backend/mod.rs)
- SuprTUI backend: [`src/backend/suprtui.rs`](../src/backend/suprtui.rs)
- Core exports: [`src/core/mod.rs`](../src/core/mod.rs)
- Render exports: [`src/render/mod.rs`](../src/render/mod.rs)
- Display capability exports: [`src/display/mod.rs`](../src/display/mod.rs)
- Renderer acceptance tests: [`tests/suprtui_renderer.rs`](../tests/suprtui_renderer.rs)
- Surface integration tests: [`tests/surface_integration.rs`](../tests/surface_integration.rs)

## Related chapters

- [Layout, style, and themes](layout-style-and-themes.md)
- [Terminal and embedded sessions](terminal-and-embedded-sessions.md)
- [Images and clipboard](images-and-clipboard.md)

[Back to the manual](README.md)
