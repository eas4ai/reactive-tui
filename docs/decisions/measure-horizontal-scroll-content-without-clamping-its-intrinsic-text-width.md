# Measure horizontal scroll content without clamping its intrinsic text width

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Ordinary controls grow beyond their viewport, nested explicit widths are ignored, or horizontal scroll bounds still omit content.
History: The horizontal Unicode probe showed that shared text measurement clamps nonwrapping content to available width. Removing that clamp globally broke existing Slider geometry tests, so the change is restricted to horizontal scroll content.

## Decision

Add a private inherited width-measurement flag to layout styles for the horizontal ScrollView content wrapper. In that subtree, nonwrapping text reports its intrinsic cell width unless layout gives an explicit known width. Other text retains existing constrained measurement. Preserve the flag through owned style snapshots and report unclipped layout content extents through LayoutInfo. Verify ordinary widget and styling tests alongside horizontal Unicode and nested-control scroll tests.

## Realized by

Implementation: `src/widgets/layout/scroll_view.rs`, `src/layout/paint_tree.rs`.

Behavior checks: `tests/api_widget_behavior/scroll.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
