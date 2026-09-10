# Paint menu outlines with the shared measured cell border renderer

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011
Would be wrong if: An enabled outline is absent, border colors replace the panel background, items overlap the outline, or resize leaves stale pointer targets.
History: Prior API reversals require observing real output and preserving ownership contracts. This local renderer reuse remains Judged because it introduces no platform dependency or public interface change; captured cells and resized input must demonstrate it.

## Decision

Use the existing table and modal cell-border renderer for popup, context and dialog menu panels. Reserve at least one cell of padding when the outline is enabled, derive its dimensions from the measured panel, and apply border classes only to its glyphs. Keep base panel styles on the panel. Verify enabled and disabled outlines, colors, clipping and resized activation through App.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`src/widgets/menu/panels.rs` reserves a Taffy border and reuses
`src/widgets/display/table/border.rs` for measured glyphs. The internal
`StyleBuilder::cell_border` keeps the edge independent of CSS padding. Border
classes apply to the glyphs; base classes remain on the panel. App regressions
in `tests/api_widget_behavior/menus.rs` cover outline toggles, colors, zero
padding, and resized dropdown input. The absent-outline baseline fails in
`.cairn/reviews/api-011-menu-border-negative.log`.
