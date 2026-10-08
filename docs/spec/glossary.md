# Glossary

Status: Observed

Terms as the code uses them. Each names where it is defined.

- **App.** The object that owns the backend, the root, the component
  runtime, the scheduler, the waker and the event router, and runs the
  frame loop (src/app.rs:99-135).
- **RootComponent.** The top-level trait an application implements; it
  renders an Element tree or hands back a CellFrame, and may be wake-driven
  (src/app.rs:50-85).
- **Element.** One node of the declarative tree: a component, text, a layout
  container, a fragment or empty, with a utility-class string
  (src/component/element.rs:143-194).
- **Component.** A keyed child with Props and State that renders Elements
  and handles events; instances are owned by the App's ComponentRuntime
  (src/component/mod.rs:52-125).
- **Backend.** The trait that presents frames and polls input; SuprTUI is the
  terminal backend, Debug renders into memory (src/backend/mod.rs:43-127).
- **Default backend.** SuprTuiBackend, which the widget catalog and the
  examples use, and CrosstermBackend, which wraps it
  (src/backend/mod.rs:238-248); DirectTtyBackend is an older path.
- **CellFrame.** A validated, owned grid of graphemes with colors and
  attributes, at most 262,144 cells, the unit a wake-driven root hands to the
  backend (src/backend/cell_frame.rs:9-40).
- **AppWaker.** The thread-safe handle that asks the loop to redraw, run
  scheduled work or stop; workers deliver results through it
  (src/reactive/wake.rs:27-74).
- **Scheduler.** The per-App queue of updates and timers the loop drains
  each iteration (src/reactive/scheduler.rs:141).
- **Signal.** A reactive value whose readers are re-run and whose changes
  request a redraw (src/reactive/signal.rs:22-300).
- **ThreadSafeSignal.** A value behind a mutex that any thread may read, set or
  update, with a version and the Apps that read it as subscribers; `update`
  runs on a copy with the lock released, `update_atomic` under the lock
  (src/reactive/hooks.rs:193-300).
- **Worker.** A named thread spawned with thread::Builder, fed by a bounded
  channel, stopped by a flag and joined on drop, that publishes results
  through the AppWaker (src/backend/suprtui.rs:196, src/embedded/mod.rs:79).
- **Chart.** The display widget that draws one or more DataSeries as bars,
  lines, areas, candles, pies, radar polygons, flows or scatter points on a
  cell canvas (`Chart` in src/widgets/display/charts.rs).
- **DataSeries.** A named list of DataPoints with a color, a line style and a
  fill style (`DataSeries` in src/widgets/display/charts.rs).
- **Plot layer.** The shared scales, ticks, axes, grid, legend, labels and
  tooltip that every chart type draws through
  (src/widgets/display/charts/plot); the name follows gpui-kit's plot
  module.
- **CellGrid.** A widget-sized grid of interned glyphs with packed
  foreground and background colors, attached to an Element with
  `with_cells`; charts and image fallback hand their cells to the painter
  this way (src/layout/paint_tree/cells.rs:17-39).
- **Canvas.** The graphics widget of canvas.md: it draws a scene the
  application describes on a hardware wgpu adapter or on the software
  renderer, and shows it as pixels or block glyphs. Distinct from the chart's
  cell canvas.
- **Hardware adapter.** A wgpu adapter whose device type is a discrete or
  integrated GPU (`hardware_adapters` in src/graphics/gpu.rs); WARP and
  lavapipe are software adapters and do not count.
- **Software renderer.** The CPU renderer that draws every canvas scene the
  GPU would when no hardware adapter is usable (src/graphics/cpu.rs).
- **Drawing thread.** The `rtui-canvas-*` thread that draws the pictures
  of every canvas in the process that shares its renderer options, with
  one adapter, device, set of pipelines and glyph atlas
  (src/graphics/worker.rs); a worker an application starts and hands to
  its canvases is one (GFX-003).
- **Pixels per cell.** How many picture pixels a canvas draws for one
  cell: the terminal's cell size, learned at startup and on resize, unless
  the application pins fewer or a hard limit forces fewer, whole in each
  direction, and the terminal scales the picture to its cells (GFX-010).
- **Glyph atlas.** A texture holding each glyph a CellGrid uses once, so the
  GPU draws the whole grid as one instanced draw.
- **Cell canvas.** The chart's shared mask canvas, which resolves each cell
  to a glyph and its colors (src/widgets/display/charts/mask.rs) and hands
  them to the painter as a CellGrid.
- **Pixel look.** A widget's or an element's appearance drawn as one pixel
  picture over the rectangle it paints where the terminal takes pixels,
  around its text, with its cell look as the fallback (pixel-looks.md).
- **Cut-out cell.** A cell of a picture's rectangle that holds text: the
  picture's pixels there are cut out on the cell's edges and the cell is
  painted in the flat color the look names, so text reads over a picture
  on Kitty graphics and on Sixel alike (PIX-001).
- **Plot picture.** The pixel picture a line, area, scatter, bar or
  candlestick chart draws its plot area as where the terminal takes Kitty
  graphics or Sixel: a canvas at the plot rectangle, drawn on the drawing
  thread from the chart worker's scene, with the axes, labels, legend and
  tooltip left as cell text (CHT-037).
- **Cell.** One character position of the terminal's grid, the unit of
  every length in the layout: a width of 24 is 24 cells across, a height
  of 3 is 3 rows.
- **Spacing class.** A padding, margin, gap or space class (`p-1`, `mx-2`,
  `gap-1`, `space-y-1`); its number is a count of cells (layout.md).
- **Theme.** Named color variables with presets that utility classes resolve
  against (src/theme/mod.rs); the active theme is the one the application
  set last, and the dark preset until it sets one (theme.md).
- **Color role.** A named color of the theme with one purpose, such as
  `surface`, `selection` or `primary-foreground`; a class names it as
  `bg-surface` or `text-primary-foreground` (theme.md).
- **Color literal.** A color written into a widget's code or its default
  style instead of taken from the theme: a hex or `rgb()` value, a color
  built from numbers, a color name, or a palette class such as
  `bg-gray-800` or `text-white`.
- **Panel.** The box of rows a menu opens: the list under a menu bar's
  title, a submenu, a context menu or a popup menu (menus.md).
- **Overlay.** A box painted over the screen's content that closes on a
  key, a button or after its time: the modal, the popover, the toast and
  the five dialogs of the dialog engine (overlays.md).
- **Veil.** The layer an overlay or a dialog menu lays over the whole
  screen behind its box, in the theme's `overlay` role, so what is under
  it shows dimmed and takes no click.
- **Control.** An input widget: the text input, the checkbox, the radio
  button, the select, the slider and the button (input-widgets.md). It
  shows a value or a choice, changes it on a key or a click, and reports
  the change through a callback.
- **Field.** The row of cells a text input or a select shows its value or
  its placeholder in, in the theme's `input` role, between the two cells
  of its frame.
- **Frame.** The first and the last cell of a field, of a checkbox's box
  and of a radio's ring (`[` and `]`, `(` and `)`), painted in `border`,
  and in `ring` while the control holds the focus.
- **Layout widget.** A widget that arranges other content and moves
  through it: the tabs, the accordion, the breadcrumb, the scroll view
  and the stack (layout-widgets.md). It shows a set of places, panels or
  rows, moves the current one on a key or a click, and reports the move
  through a callback.
- **Tab bar.** The row (or, for tabs at the left or the right, the
  column) of a tabs widget's labels, one per tab, which scrolls when the
  labels do not fit.
- **Trail.** A breadcrumb's row of segments and separators, from the
  first segment to the current one.
- **Thumb.** The cells of a scroll bar that mark the part of the content
  in view, which a drag moves; the rest of the bar is its track.
- **Data widget.** A widget that shows a set of records: the table, the
  data table, the tree, the file explorer and the progress bar
  (data-widgets.md). It shows rows of a set, or one value's share of a
  range, moves the current row on a key or a click, and reports a choice
  through a callback.
- **Cursor row.** The row a data widget's keys act on, painted in
  `selection` while the widget holds the focus; a selected row, which a
  choice has marked, is painted in `accent` whether or not the widget
  holds the focus.
- **Kitty keyboard protocol.** Progressive keyboard enhancement flags a
  terminal accepts with `CSI > flags u` and drops with `CSI < u`; with them
  it reports keys as `CSI code ; modifiers ; text u`, so keys that legacy
  encoding merges (Shift+Enter and Enter, Ctrl+H and Backspace) arrive apart.
- **Test hosts.** The developer's Apple Silicon Mac and Windows 11 machine,
  reached over SSH for the builds and tests the Linux development host cannot
  run; their addresses stay outside the repository.
