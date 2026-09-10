# Keep noncancelable wizard completion reachable by keyboard

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-012 API-006
Would be wrong if: Disabling wizard cancellation is intended to disable keyboard completion and navigation.
History: The earlier dialog Escape-policy repair separated dismissal from keyboard navigation for confirmation, input and autocomplete. The same coupling remains in the wizard; a real App test leaves Finish visible but unreachable with Enter when cancellation is disabled.

## Decision

Use the existing private Modal Escape policy for WizardDialog. Keep button keyboard navigation enabled and let cancelable control only cancellation and close affordances. Verify that Escape preserves a noncancelable wizard and Enter finishes it at both viewport sizes; retain existing wizard navigation, validation and cancellation checks.

## Realized by

Implementation: `src/widgets/dialog/wizard.rs`, `src/widgets/dialog/wizard/live.rs`, `src/builder/dialog_builders.rs`.

Behavior checks: `tests/api_widget_behavior/wizard.rs`, `tests/api_widget_behavior/orca_dialogs.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
