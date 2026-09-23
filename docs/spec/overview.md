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
binding, Linux screen-reader support over AT-SPI, and an offscreen wgpu
demo.

Version as recorded in Cargo.toml and git tags: 1.0.0, a source-only release
with all companion crates marked publish = false. The README still describes
a 0.1.0 candidate; this keystone follows the manifest and tags until the
developer rules otherwise (docs/recon.md, question 1).

## The problem it solves

Terminal applications today target large, fast terminals: 500 to 700
columns, 1440p or larger, hosts that support Kitty or Sixel graphics. Most
terminal UI libraries assume 80 columns, redraw everything every frame, and
offer widgets with 1990s density. Reactive TUI exists so an application can
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
| quality-bar.md | BAR | the bar every commitment must clear: gates, assertions, widget behavior, goldens, frame budget, docs, dangling paths |
| charts.md | CHT | the plot layer and the chart widget family modeled on gpui-kit |
| rasterizer.md | RAS | the SuprTUI rasterizer: cursor and style elision, allocation-free emission, replay equivalence, byte and time bounds |
| painter.md | PNT | the frame painter's fast path, the per-cell hit grid, one element copy per present, layout reuse |
| presentation.md | PIP | pipelined presentation: geometry returned before the terminal write, one frame in flight, flush failure reporting |
| blitters.md | BLT | image fallback to block glyphs: half, quadrant, sextant, octant and braille blitters and their tier choice |

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
- Layout, utility classes, themes (src/layout, src/theme).
- Widgets other than charts (src/widgets).
- Image protocols and terminal capability detection (src/widgets/display/image
  other than fallback, src/core/capabilities).
- Embedded terminal and PTY, Windows ConPTY (src/embedded, src/terminal).
- Accessibility (src/accessibility).
- Graphics canvas over wgpu (src/graphics), planned as the commitment after
  charts.
- C ABI and TypeScript binding (src/ffi, include, bindings/typescript).
- Build, CI, dependency policy, release packaging (Cargo.toml, deny.toml,
  .github/workflows, scripts).
- Documentation (README, manual/).
