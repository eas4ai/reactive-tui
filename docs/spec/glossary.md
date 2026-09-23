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
- **Worker.** A named thread spawned with thread::Builder, fed by a bounded
  channel, stopped by a flag and joined on drop, that publishes results
  through the AppWaker (src/backend/suprtui.rs:196, src/embedded/mod.rs:79).
- **Chart.** The display widget that draws one or more DataSeries as bars,
  lines, areas, pies or scatter points on a cell canvas
  (src/widgets/display/charts.rs:513).
- **DataSeries.** A named list of DataPoints with a color, a line style and a
  fill style (src/widgets/display/charts.rs:341-354).
- **Plot layer.** The shared scales, ticks, axes, grid, legend, labels and
  tooltip that every chart type draws through. Planned; the name follows
  gpui-kit's plot module. Today each chart type maps values privately
  (src/widgets/display/charts/live/canvas/cartesian.rs:3-23).
- **Cell canvas.** The chart's in-memory grid of glyphs and colors, painted
  today as absolutely positioned text runs
  (src/widgets/display/charts/live/canvas.rs:99-142).
- **Theme.** Named color variables with presets that utility classes resolve
  against (src/theme/mod.rs:10-80).
