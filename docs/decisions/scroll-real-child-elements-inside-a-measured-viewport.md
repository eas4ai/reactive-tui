# Scroll real child elements inside a measured viewport

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Content or child events disappear, offsets exceed measured content, clipped children receive pointer input, or smooth scrolling keeps work alive after unmount.
History: The existing API recovery decisions require App-owned state and presented geometry. This extends those mechanisms without changing public builder or props signatures.

## Decision

Keep ScrollView content as an Element under a clipped viewport. Measure the content wrapper and viewport through existing layout callbacks, and position content using the shared layout engine so Unicode painting, nested controls and hit bounds remain aligned. Clamp offsets against those measurements; use signed wheel deltas and focus-aware keyboard input. Paint scrollbars from the same dimensions. Smooth scrolling interpolates cell offsets for a bounded duration using the App scheduler, cancelling its timer on unmount. Both builders retain their content and classes.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/layout/scroll_view.rs`, `src/layout/paint_tree.rs`.

Behavior checks: `tests/api_widget_behavior/scroll.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
