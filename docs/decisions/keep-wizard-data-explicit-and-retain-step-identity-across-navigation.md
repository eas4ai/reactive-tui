# Keep wizard data explicit and retain step identity across navigation

Level: Judged
Decided by: Codex
Rests on: API-011,API-012
Would be wrong if: Step validators cannot observe caller data, Back loses child state, reordered steps change identity, or callbacks run twice or under a lock.
History: The retained dialog decision requires real App controls and preserves existing native methods; the wizard currently keeps an inaccessible empty data map and skips validation.

## Decision

Expose explicit set_data and data access on WizardDialog. Callers own the named string values and update them through their normal child callbacks; do not infer field names from arbitrary Element content. Pass authored data to the retained App control so validators and completion callbacks receive current values. Retain visited step children under stable step IDs while only the active panel receives layout and input. Next and Finish validate the active step; Skip bypasses validation only when that step permits it. Reject empty or duplicate step IDs and empty step lists with a visible configuration error and a working cancellation route. Preserve current public options and native methods, including completion veto. Builders use the same retained control and enforce their can_proceed, initial step, progress, cancellation and styling settings.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/dialog/wizard.rs`, `src/widgets/dialog/wizard/live.rs`, `src/builder/dialog_builders.rs`.

Behavior checks: `tests/api_widget_behavior/wizard.rs`, `tests/api_widget_behavior/orca_dialogs.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
