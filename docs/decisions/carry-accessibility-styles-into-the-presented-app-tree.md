# Carry accessibility styles into the presented App tree

Level: Judged
Decided by: Codex
Rests on: API-011, API-018, API-020
Would be wrong if: Accepted accessibility styles remain paint-only, explicit label values disappear, focus and reader state disagree with the presented frame, or incomplete label references silently succeed.
History: Earlier API reversals exposed insufficient platform evidence and adapter assumptions. This remains Judged because it restores existing semantics without narrowing the approved platform contract; real reader verification and failure cases are required before acceptance.

## Decision

Retain accessibility attributes in StyleBuilder snapshots and apply them to the same resolved Element candidate used for event registration and screen-reader publication after presentation succeeds. Preserve existing visual effects. Connect the existing value-taking focus::apply_aria_attribute and apply_role APIs to this metadata. Bare label and reference tokens use explicitly supplied attributes; report missing values instead of inventing labels. Provide explicit per-App accessibility IDs for label and description relationships, validate duplicate and missing references, and preserve existing Element and widget label APIs. Verify actual Orca delivery alongside App keyboard, mouse, disabled state, style snapshot and reduced-motion behavior. No reader acceptance is inferred from metadata alone.

## Realized by

Implementation: `src/accessibility/style.rs`, `src/app/event_tree/accessibility.rs`.

Behavior checks: `tests/api_widget_behavior/accessibility_styles.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
