# Reactive TUI

Status: Observed

## What it is

Reactive TUI is a Rust framework for terminal applications. An application
declares a tree of retained components with reactive state and CSS-like
utility classes; the framework lays the tree out with Taffy, paints complete
grapheme-aware cell frames, and presents them through the SuprTUI backend,
which owns terminal setup, input, presentation and restoration. A debug
backend renders into memory so behavior can be checked without a terminal.
The crate ships a widget library (inputs, tables, trees, menus, dialogs,
images, charts, an embedded terminal), an optional C ABI with a TypeScript
binding, Linux screen-reader support over AT-SPI, and an optional graphics
canvas drawn with wgpu (canvas.md).

Version as recorded in Cargo.toml and git tags: 1.0.0, a source-only release
with all companion crates marked publish = false. The README still describes
a 0.1.0 candidate; this keystone follows the manifest and tags until the
developer rules otherwise (docs/recon.md, question 1).

## The problem it solves

Terminal applications today target large, fast terminals: 240 to 512
columns and more, on 1440p or larger screens, hosts that support Kitty or
Sixel graphics. Most terminal UI libraries assume 80 columns, redraw
everything every frame, and offer widgets with 1990s density. Reactive TUI exists so an application can
be written the way a desktop or web application is written, with components,
state and layout, and still run inside the user's terminal at 60 frames per
second with widgets whose function and appearance match a modern desktop
component library as closely as a cell grid allows.

## What it is not

- Not a pixel renderer. Output is a grid of cells; braille, block and
  box-drawing glyphs give sub-cell resolution, and image protocols carry
  pixels only where the host supports them.
- Not a windowing system. There is one terminal, one application, one
  frame at a time. The wgpu module is an offscreen texture, not a window.
- Not a registry release. Companion crates are vendored under crates/ and
  are not published; consumers use a path or git dependency.
- Not verified on every host. Behavior is proven on the debug backend and on
  Linux; Kitty, iTerm2, WezTerm, GNOME Terminal, Orca, Windows ConPTY and
  macOS clipboard claims are unverified until a mechanism records them.

## Spec map

| File | Prefix | Covers |
|---|---|---|
| quality-bar.md | BAR | the bar every commitment must clear: gates, assertions, widget behavior, goldens, frame budget, docs, dangling paths, dependency checks |
| charts.md | CHT | the plot layer and the chart widget family modeled on gpui-kit |
| rasterizer.md | RAS | the SuprTUI rasterizer: cursor and style elision, allocation-free emission, replay equivalence, byte and time bounds |
| painter.md | PNT | the frame painter's fast path, the per-cell hit grid, one element copy per present, layout reuse |
| presentation.md | PIP | pipelined presentation: geometry returned before the terminal write, one frame in flight, flush failure reporting |
| input.md | INP | terminal input on the default backend: mouse and paste modes, event translation, drag capture, clicks, wheel routing, motion merging, the Kitty keyboard protocol, lock and media keys, terminal focus reports, suspend and resume, startup queries |
| blitters.md | BLT | image fallback to block glyphs: half, quadrant, sextant, octant and braille blitters and their tier choice |
| layout.md | LAY | gaps, grid tracks and spans in whole cells, the spacing classes' unit, the gap and grid classes, layout that settles |
| theme.md | THM | the theme's color roles, their contrast, what a theme that lacks a role gets, and a change of theme |
| menus.md | MNU | the menu bar, context menu, popup menu and dialog menu: their colors by role, their size, where a panel opens, what it is painted over |
| overlays.md | OVL | the modal, popover, toast and the five dialogs: their colors by role, their size in cells, where each is placed and what it is painted over, what the screen reader is told |
| input-widgets.md | CTL | the text input, checkbox, radio button, select, slider and button: their colors by role, the width they fill, where a select's list opens and what it is painted over, what the screen reader is told |
| layout-widgets.md | NAV | the tabs, accordion, breadcrumb, scroll view and stack: their colors by role, the size they fill, what happens when their content does not fit, what the screen reader is told |
| data-widgets.md | DAT | the table, data table, tree, file explorer and progress bar: their colors by role, the size they fill, numeric sorting and a revealed selection, what the screen reader is told |
| canvas.md | GFX | the graphics canvas widget: its scene, the hardware and software renderers, its worker, the tablet speed floor, pixel and block output, graphics detection, faults, the demos, and pictures made ready off the App's wait |

Vocabulary is in glossary.md; the commitment order is in roadmap.md.

## Areas not yet specified

These areas exist in the code and are outside the current radius. Each
becomes a domain file when a commitment reaches it. Until then, nothing here
is contract.

- Application loop, terminal restoration on panic and signals (src/app.rs,
  src/backend other than presentation). Orphaned runners for panic and signal fixtures,
  docs/recon.md section 8.
- Components, elements, hooks, signals, scheduler, wake (src/component,
  src/reactive).
- Layout and utility classes other than layout.md's (src/layout), and the
  parts of a theme other than its color roles: spacing variables, loading
  a theme from a file, the syntax colors (src/theme, src/syntax).
- Widgets other than the charts, the canvas, the menus, the overlays, the
  controls, the layout widgets and the data widgets (src/widgets): the
  images and the embedded terminal.
- Image protocols and terminal capability detection (src/widgets/display/image
  other than fallback, src/core/capabilities).
- Embedded terminal and PTY, Windows ConPTY (src/embedded, src/terminal).
- Accessibility (src/accessibility).
- C ABI and TypeScript binding (src/ffi, include, bindings/typescript).
- Build, CI and release packaging (Cargo.toml, .github/workflows, scripts).
  The dependency checks on deny.toml are BAR-008.
- Documentation (README, manual/).
