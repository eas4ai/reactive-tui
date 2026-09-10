# Own accordion layout and motion in a retained child component

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Accordion construction breaks, state resets on ordinary redraw, nested content loses identity, measured headers disagree with input, or animation timers survive unmount.
History: Prior API decisions require preserving construction and testing real geometry and resource lifetime. This uses the existing App-owned component tree without changing public field construction.

## Decision

Preserve the public Accordion unit struct and its public props and state fields. Render a private retained component that owns measured header targets and bounded animation resources. Keep content as keyed child elements and clip its animated height. Use existing App component layout delivery and scheduler ownership. Reconcile changed section props and modes, skip disabled headers, and preserve expansion on redraw when persistence is enabled.

## Realized by

Implementation: `src/widgets/layout/accordion/live.rs`, `src/widgets/layout/accordion/live/motion.rs`.

Behavior checks: `tests/api_widget_behavior/accordion.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
