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
