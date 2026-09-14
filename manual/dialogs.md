# Dialogs

Crate modules: `widgets`

Widget module: `dialog`

## Purpose

Dialogs manage short interactions that sit above normal application content and
produce an explicit result.

## Main API

- `DialogEngine` owns active dialog entries and completion events.
- `DialogEngineConfig` sets engine behavior.
- `DialogId`, `DialogEvent`, and `DialogResult` identify and report dialog work.
- `ConfirmationDialog` asks the user to choose a button.
- `InputDialog` edits and validates fields.
- `AutocompleteDialog` filters and selects suggestions.
- `ProgressDialog` shows ongoing work and optional cancellation.
- `Toast` shows a timed notification.
- `WizardDialog` moves through ordered steps.
- `DialogTheme` and dialog animation values control presentation.

## Basic use

Create a shared dialog engine, open a configured dialog, render the engine's
element in the root tree, and consume completion events. Use live dialog paths
when the dialog must update while the app waits for input.

## Behavior

The engine assigns an ID, mounts the dialog, routes updates and events, and
retires it after completion. Completion is delivered once. Toast expiry and
other scheduled changes wake an idle application. Input validation runs before
an accepted result is published.

The engine maintains dialog focus and stacking. Close, cancel, submit, timeout,
and transport failure are distinguishable outcomes where the dialog type
supports them.

## Limits

- The engine must remain reachable while dialogs are active.
- Remote input helpers use configured HTTP behavior and can fail independently
  of local dialog rendering.
- Progress updates must use the live update route to appear while the app is
  idle.
- Validation callbacks should avoid blocking the event loop.

## Source map

- Dialog exports and common values: [`src/widgets/dialog/mod.rs`](../src/widgets/dialog/mod.rs)
- Dialog engine: [`src/widgets/dialog/engine.rs`](../src/widgets/dialog/engine.rs)
- Input validation: [`src/widgets/dialog/input/validation.rs`](../src/widgets/dialog/input/validation.rs)
- Dialog behavior tests: [`tests/api_widget_behavior/dialogs.rs`](../tests/api_widget_behavior/dialogs.rs)
- Dialog lifecycle tests: [`tests/api_dialog_lifecycle.rs`](../tests/api_dialog_lifecycle.rs)

## Related chapters

- [Menus](menus.md)
- [Reactive state and hooks](reactive-state-and-hooks.md)
- [Animation and screens](animation-and-screens.md)

[Back to the manual](README.md)
