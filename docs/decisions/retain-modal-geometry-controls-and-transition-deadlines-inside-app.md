# Retain modal geometry controls and transition deadlines inside App

Level: Judged
Decided by: Codex
Rests on: API-011,API-005,API-006
Would be wrong if: Modal still uses guessed dimensions, consumes child content, loses a close result, leaks a transition timer, or drags against stale bounds.
History: The earlier widget and accessibility reversals require actual App workflows and failure demonstrations. This is Judged: retain public construction and state types, use existing layout, focus and scheduler ownership, and verify each advertised control.

## Decision

Keep the public unit Modal and its prop helpers. Render through a private retained child that owns local dismissal, measured geometry, drag/resize state and a cancellable finite transition clock. Auto size follows expanded content; explicit sizes and positions use the measured clipping viewport in terminal cells. Build real header/action controls and preserve content/footer Elements, reusing ScrollView for overflow. Route drag and resize through measured targets and an overlay-wide observer until release or focus loss. Use existing App traps and nontrapping focus scopes; closing content becomes inert immediately. Changed visibility props can reopen a dismissed modal, while unrelated prop changes retain interaction state. Preserve callback reasons and custom action identifiers, deliver action notifications outside locks, and make the generic builder construct this component. Shared dialog-manager stacking/results remain obligations of API-012.

## Realized by

Implementation: `src/widgets/display/modal`.

Behavior checks: `tests/api_widget_behavior/modal.rs`, `tests/api_widget_behavior/orca_overlays.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
