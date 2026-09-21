# Catalog Visual Repairs Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans inline, task by task. Cairn controls action order, commits, evidence, and review; its working agreement overrides generic branch and delegation guidance.

**Goal:** Resolve the approved CAT-001 layout and CAT-002 cube findings without promoting unrelated work.

**Architecture:** Configure the example with measured terminal dimensions and existing public builders. Keep optional wgpu rendering unchanged. Render the default wireframe with Unicode Braille subcells instead of dotted cell strokes.

**Tech Stack:** Rust 1.91, existing ScrollView/grid/style builders, native SuprTUI, owned Kitty/Xvfb visual captures.

## Task 1: Responsive pages (CAT-001)

Files: `examples/widget_catalog/catalog.rs`, `tests/widget_catalog_behavior.rs`.

- [x] Write viewport and spacing regressions, then run them before implementation:

```rust
catalog.resize(144, 50).unwrap();
let element = catalog.render();
let props = find_props::<reactive_tui::widgets::layout::ScrollViewProps>(&element).unwrap();
assert!(props.viewport_width > 100);
assert!(props.viewport_height > 40);
```

Run `cargo +1.91.0 test --locked --test widget_catalog_behavior`; expect failure on the current 80x24 defaults, stretched navigation, and missing colored spans.

- [x] Configure the outer scroll viewport directly, avoiding the start-aligned convenience Stack:

```rust
let page = self.selected_page();
let page_class = format!("{} w-full min-w-0 shrink-0 whitespace-normal",
                         page.class.as_deref().unwrap_or_default());
reactive_tui::widgets::layout::ScrollViewBuilder::new(page.with_class(page_class))
.viewport_size(usize::from(self.width.saturating_sub(if self.width >= 80 { 26 } else { 2 })),
               usize::from(self.height.saturating_sub(if self.width >= 80 { 8 } else { 11 })))
.scroll_x(false).scroll_y(true).show_scrollbars(true).render()
.with_class("w-full flex-1 min-h-0")
```

Use `shrink-0 h-1` sidebar entries; reserve `flex-1` for compact horizontal entries. Replace page `gap-1` with `gap-0.25` (one cell). Use one grid column below the sidebar breakpoint and two above it; all pages have `w-full`. Give card headings `h-1 shrink-0` and text explicit normal wrapping.

- [x] Add a full-width Layout card using existing grid utilities:

```rust
div().class("grid grid-cols-4 gap-0.25 w-full")
    .child(div().class("col-span-4 h-3 bg-cyan-700 text-white").text("span 4").build())
    .child(div().class("col-span-2 h-3 bg-violet-700 text-white").text("span 2").build())
    .child(div().class("h-3 bg-amber-700 text-white").text("span 1").build())
    .child(div().class("h-3 bg-emerald-700 text-white").text("span 1").build())
    .child(div().class("col-span-3 h-3 bg-blue-700 text-white").text("span 3").build())
    .child(div().class("h-3 bg-rose-700 text-white").text("span 1").build()).build()
```

- [x] Require native terminal-cell background runs to prove spans, not just labels; check 60x24, 144x50, and 200x60. Run default and feature-enabled catalog tests and focused Clippy. Commit, then follow Cairn for refreshed evidence and review.

## Task 2: Readable default wireframe (CAT-002)

Files: `examples/widget_catalog/motion.rs`, `examples/widget_catalog/catalog.rs`, `tests/widget_catalog_behavior.rs`, `scripts/check-widget-catalog-pty.py`.

- [x] Add a failing viewport-aware wireframe test: terminal rows/columns must match the canvas, rotation must change Braille pixels, and oversized/empty viewports must stay bounded.
- [x] Add `cube_frame_sized(elapsed: Duration, width: usize, height: usize) -> String`; retain `cube_frame` as the 40x16 compatibility wrapper. Clamp to finite 240x100 cells before allocating. Project into 2x4 subcells per Braille character, maintaining the physical cell aspect. Rasterize twelve edges with the existing Bresenham algorithm and pack masks into U+2800 plus the Braille mask. Use actual available Motion dimensions, leaving title and footer rows intact. Keep elapsed-time rotation and the 80 ms clock.

```rust
const DOTS: [[u8; 2]; 4] = [[1, 8], [2, 16], [4, 32], [64, 128]];
let cell = &mut cells[y as usize / 4][x as usize / 2];
*cell |= DOTS[y as usize % 4][x as usize % 2];
```

- [x] Change the PTY cube-only hash to include Braille U+2801..U+28FF, not changing labels. Require distinct frames and clean exit for all three quit keys. Run deterministic/default/feature-enabled tests, formatting and focused lint; commit before Cairn checks.

## Task 3: Visual acceptance and closing review

- [x] Reuse the owned Kitty/Xvfb Host harness to capture Overview, Layout, and Motion at the required sizes, including grow/shrink. Use fresh directories; never attach to the developer desktop. Inspect actual PNGs for full width, complete coverage, tight spacing, correct spans, legible edges, and intact controls. Report software host glyph rendering separately from hardware cube rendering.
- [x] Update runnable documentation without claiming every host or 60 FPS. Run `cairn check` only when wake names the requirement; read and commit each receipt and its output. Record final no-code review with exact candidate/evidence/artifacts, resolve only verified findings, and run wake again. New scope remains the developer's choice.

Plan review: Both open findings have explicit tests and visual acceptance. CAT-003 remains unchanged and is rerun after shared-input changes. The optional GPU path, local logo, live widget inventory, public APIs, and release boundaries are preserved. No new dependency or framework subsystem is needed.

### CAT-001 implementation verification

The new tests first failed on the 80x24 viewport, stretched sidebar, and absent spans. Native cell tests also rejected an intermediate class override that erased the grid classes. Screenshot inspection then found cards extending the second example column outside the viewport; the new two-column visibility test failed with missing Accordion and passed after removing percentage width from grid children. This is a failure demonstration, not a claim that constructor inventory proves visible layout.

The corrected tree passed 19 default and 21 feature-enabled behavior tests, 11 validator tests, formatting, and focused Clippy with only the pre-existing wizard `collapsible_else_if` lint allowed. Owned Kitty/Xvfb captures at 60x24, 144x50, and 200x60 show wrapped coverage, compact navigation, full-width spans, and both example columns. Local diagnostic captures in `20260916T205755974203Z-860142` show the intermediate clipping; corrected captures are in `20260916T205859937742Z-870701`. Cairn evidence and final review remain separate committed actions.

### CAT-002 implementation verification

The viewport test first failed with no Braille canvas on the old configuration. The correction passes 21 default and 23 feature-enabled tests, including zero/oversized dimensions, elapsed-time rotation, preserved deadlines, real native App cell counts, and header/footer retention. The simpler DebugBackend paints only the first row of a multiline text node; it is used for controls, not as proof of cube pixels. PTY observations select only nonblank Braille cells and pass distinct-frame and three-quit cleanup checks. All fourteen Ripwire-named integration targets and the library suite pass (1051 passed, eight ignored). Formatting and focused Clippy pass with only the inherited wizard lint allowed.

Long owned-host resize captures reproduced a separate harness deadlock: Kitty mode-change warnings filled its undrained stderr pipe. Replacing just that sink in a diagnostic run allowed the identical grow/shrink capture to pass. A real child writing 110 KB to the launcher's diagnostic sink failed before the fix and passes with a private regular file. Diagnostics remain available; the native warning itself is captured in the backlog rather than claimed repaired.

The corrected host run `20260916T210812516632Z-972771` captures Overview, Layout, and Motion in one instance at 60x24, 144x50, 200x60, and back to 60x24. Actual inspected PNGs show intact controls, colored spans, and bounded readable wireframe edges after resize. Host cases are requirement-selected so CAT-001 does not depend on unfinished motion acceptance and CAT-002 does not inspect unrelated layout widgets. Fresh Cairn receipts and no-code closing review are still pending.

Ripwire quality-delta exits 2 on constructor-token similarity to an unrelated image test and short-horizon Cairn commit churn; it also reports new rasterizer complexity and graph-unreached test functions. Test-gate exits 4 while naming obligations; all fourteen named integration targets were executed successfully. These graph reports are not claimed green or used to justify unrelated abstraction or scope expansion.
