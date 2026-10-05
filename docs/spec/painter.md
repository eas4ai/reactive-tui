# Painter and hit testing

Prefix: PNT

The painter in src/layout/paint_tree/suprtui.rs lays the element tree out
with Taffy on the render worker and paints it into the crate's cell buffer;
src/backend/suprtui.rs hands it the frame. Today every node paints through a
per-cell inverse transform, `hit_bounds` scans every cell of a masked node
to bound it, `render_frame` clones the element and `present` clones it
again, and layout runs on every present. The rasterizer is in
rasterizer.md and presentation in presentation.md.

## Observed

(none yet)

## Draft

[PNT-001] A node with an identity transform and no mask MUST fill its background by row and paint its text without a per-cell inverse transform; masked or transformed nodes keep the general path and both paths MUST produce the same cells.
Falsifier: A per-cell inverse transform runs for an identity node, or the fast path's cells differ from the general path's on any golden.
Mechanism: painter-goldens
Status: Agreed 2026-09-22

[PNT-002] The Backend MUST offer a per-cell hit query answered from the renderer's committed hit grid, which the painter fills with element indices as it paints, exact under masks, transforms and z-order, committed and rolled back with the frame; the event layer MUST prefer the query when offered and fall back to painted bounds otherwise; the per-cell mask scan in `hit_bounds` MUST be removed.
Falsifier: A click inside an element's bounding rectangle but outside its mask dispatches to that element, or a click on a cell painted by a higher z-order sibling dispatches to the lower one.
Mechanism: painter-goldens
Status: Agreed 2026-09-22

[PNT-003] One `Element` copy per present: `render_frame` MUST store a shared handle and `present` MUST send that handle.
Falsifier: Two deep copies of the frame element are observed per present.
Mechanism: render-alloc
Status: Agreed 2026-09-22

[PNT-004] When the paint spec is unchanged since the last frame at the same size, layout MUST not be recomputed.
Falsifier: Presenting an unchanged spec re-runs layout.
Mechanism: painter-goldens
Status: Agreed 2026-09-22

[PNT-005] When a frame's paint spec differs from the last frame's at the same size, every element at the same position in the tree with the same class, style, text and image MUST keep its layout node, only new or changed elements MAY get new nodes, layout MUST run again only for what the change can move, and the painter MUST paint the same cells and hit grid as a full layout of the same spec.
Falsifier: In a grid of 199 rows of 25 text elements at 700 by 200, a frame that changes one element's text builds a layout node for any other element or measures the text of an element outside that element's row, and with elements of fixed width measures the text of any other element; or, over frames that change text, classes and children, a frame's cells or hit grid differ from a full layout of the same spec.
Mechanism: painter-goldens
Rationale: Measured on 2026-09-27 at 2f6f66ef in release at 700 by 200: with 4,975 text elements and one changing per frame, layout took 7.1 ms of a 20 ms frame, and 0.9 ms when nothing changed.
Status: Agreed 2026-09-27

[PNT-006] A cell an element paints in its own explicit colors (`Element::with_cells`) MUST be painted at the element's own opacity times its ancestors' opacity, as the element's style colors are.
Falsifier: An element with the classes `w-1 h-1 opacity-50` whose one explicit cell is red, over a black parent, presents a cell whose foreground or background is not half red within one step of rounding; or the same cell at full opacity inside a parent at half opacity presents a different color.
Mechanism: review-core
Rationale: Explicit cell colors were attenuated by the ancestors' opacity only, so an element's own partial opacity left its chart, image and canvas cells at full strength (the developer's code review of 2026-10-04, C16).
Status: Agreed 2026-10-05
