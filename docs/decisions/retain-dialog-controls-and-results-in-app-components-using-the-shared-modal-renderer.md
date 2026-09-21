# Retain dialog controls and results in App components using the shared modal renderer

Level: Judged
Decided by: Codex
Rests on: API-011,API-012
Would be wrong if: Dialog edits reset on redraw, callbacks or close results duplicate, vetoed submissions still close, geometry uses fixed hit bounds, or separate Apps share dialog state.
History: Earlier API reversals required proving lifecycle and geometry at their actual boundaries. Dialog builders currently return descriptions and several dialog types render empty elements; existing Modal and input controls already have measured App workflows.

## Decision

Make dialog construction routes produce retained App components with their authored options and initial state. Reuse the measured Modal frame, buttons, text editing and progress controls, while each dialog owns its selection, validation, completion and deadline state. Keep public DialogComponent methods and result types. Deliver callbacks after releasing state locks, preserve vetoed actions, and keep callback replacement separate from local edits. Verify each family through App at two viewport sizes, including pointer input after resize, disabled and empty states, removal and result delivery. Wire the dialog engine to these same controls under its lifecycle requirement; do not substitute descriptive builder output or an unobserved result for acceptance.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/dialog`, `src/widgets/display/modal`.

Behavior checks: `tests/api_widget_behavior/dialogs.rs`, `tests/api_widget_behavior/confirmation.rs`, `tests/api_widget_behavior/input.rs`, `tests/api_widget_behavior/autocomplete.rs`, `tests/api_widget_behavior/wizard.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.

The App controls are implemented. Wiring DialogEngine to their lifecycle and
results remains API-012 work; that portion is not yet realized.
