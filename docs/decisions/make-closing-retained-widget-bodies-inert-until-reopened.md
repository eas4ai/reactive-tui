# Make closing retained widget bodies inert until reopened

Level: Judged
Decided by: Codex
Rests on: API-005, API-006, API-011
Would be wrong if: Closing content still receives activation or Tab focus, reopening loses child state, or an unrelated accessible-hidden decoration becomes inert.
History: An App test with a two-second accordion collapse shows Tab and Enter activating the still-painted inner button after collapse starts. Accessible hidden metadata alone does not remove event targets.

## Decision

Add an internal inherited inert flag to expanded element metadata. Event registration omits focus, handlers and traps for an inert subtree while preserving layout, rendering and retained components. Accessibility snapshots omit the same subtree. Accordion sets the body inert as soon as its logical expanded state is false, independently of animation progress. Reopening restores normal registration. Verify Tab skips the closing body, pointer activation is suppressed, and retained input still survives close/reopen.

## Realized by

Implementation: `src/widgets/display/modal`, `src/widgets/layout/accordion/live.rs`.

Behavior checks: `tests/api_widget_behavior/modal.rs`, `tests/api_widget_behavior/accordion.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
