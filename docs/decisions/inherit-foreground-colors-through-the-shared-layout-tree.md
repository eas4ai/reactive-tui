# Inherit foreground colors through the shared layout tree

Level: Judged
Decided by: Codex
Rests on: API-011,API-009
Would be wrong if: An explicit child color is overwritten, an ancestor color affects an unrelated sibling, or rendered color still differs from the styled region.
History: The Modal cell-color tests exposed a shared missing inheritance path despite earlier styling passes. This is Judged because the existing CSS-like tree and explicit overrides define the behavior; real cell-color assertions and inherited regressions will verify it.

## Decision

Carry the resolved foreground RGBA value down the shared layout builder alongside inherited typography. Apply it only when a node has no explicit foreground. Keep backgrounds local and preserve child overrides. Modal region classes must color their actual text descendants, and its default black foreground must remain visible on the white background. Verify sibling isolation and explicit overrides as well as the Modal workflow.

## Realized by

Implementation: `src/layout/paint_tree.rs`.

Behavior checks: `tests/api_widget_behavior/core_builders.rs`, `tests/api_widget_behavior/menus.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
