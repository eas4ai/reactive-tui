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

## Network access

Dialogs make no network request unless the application sets an endpoint.
`ValidationConfig::async_validation_url` enables input validation requests on
submit and on any configured change or blur trigger.
`AutocompleteConfig::suggestions_url` enables suggestion requests after the
configured minimum input and debounce delay.

Remote helpers require curl 8.4 or newer. They send JSON with HTTP or HTTPS
POST, reject redirects and other protocols, limit one request to 1 MiB, limit
one response to 64 KiB, and stop after five seconds. Replacing or closing the
dialog cancels and reaps the request process. URLs, headers, and bodies travel
through curl's private standard input. Certificate verification remains on.

The curl process receives only variables used to find curl, configure an HTTP
or HTTPS proxy, or choose a certificate store. These are `PATH`, Windows
`PATHEXT`/`SystemRoot`/`WINDIR`, lowercase `http_proxy`/`https_proxy`/
`all_proxy`/`no_proxy`, uppercase `HTTPS_PROXY`/`ALL_PROXY`/`NO_PROXY`,
`CURL_CA_BUNDLE`, `SSL_CERT_FILE`, and `SSL_CERT_DIR`. It does not receive the
rest of the application environment or user curl configuration.

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
