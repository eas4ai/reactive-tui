# Retain component instance identity in event and accessibility paths

Level: Judged
Decided by: Codex
Rests on: API-002, API-006, API-011
Would be wrong if: Keyed redraw or reorder loses focus, replacement reuses the removed control identity, or event and accessibility paths disagree.
History: The Orca breadcrumb replacement workflow fails keyboard navigation after assistive focus. An App regression reproduces a click on Root followed by Enter activating Docs: the replaced retained child never receives focus because its flattened layout slot keeps the old router identity.

## Decision

Assign each App-owned component mount a monotonically increasing identity and retain the identity chain on its expanded root metadata. Include that chain in event, styling and accessibility paths. Stable instances keep their identity across redraw and keyed reorder; a replacement gets new router nodes, removes the old targets, and receives normal autofocus delivery. Preserve user keys and layout geometry. Verify the click-to-keyboard replacement failure and the real Orca workflow, then rerun existing component and focus acceptance checks.

## Realized by

Implementation: `src/component/instance.rs`, `src/app/event_tree.rs`, `src/app/event_tree/accessibility.rs`.

Behavior checks: `tests/api_widget_behavior/accessibility_styles.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
