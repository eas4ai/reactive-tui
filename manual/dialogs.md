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

## Colors and sizes

Every dialog, the toast included, takes its colors from the active theme's
roles: the box is `bg-surface text-foreground` with a border in `border`,
the veil behind a modal dialog is `overlay`, the primary button (OK, Yes,
Next, Finish) is `bg-primary text-primary-foreground`, a danger button
`bg-error text-error-foreground`, every other button `bg-secondary
text-secondary-foreground`, and the button that holds the focus
`bg-selection text-selection-foreground`; a button whose variant the
theme has no look for takes the fill of the role it names (`warning`,
`success`, `info`) or the secondary look. An error line is `text-error`
and a warning `text-warning`. `DialogTheme::default()` names those roles;
`DialogTheme::of(&theme)` writes the colors of one theme, so the dialog
keeps them when the application changes its theme, and
`DialogThemes::light()`, `dark()` and `high_contrast()` do that for the
preset of that name. `DialogThemes::minimal()` draws its buttons as text.
`DialogTheme::border_style` holds classes added to the box; the border
itself is the modal's own, so a `border-<color>` class there paints the
box's background instead.

A box whose width the options do not set is as wide as its message, its
title or its row of buttons needs, plus one cell of padding at each side,
and at most half the viewport; a longer message wraps. A title and a
button have one cell of padding at each side. The field of the input and
autocomplete dialogs, the suggestion list and the progress dialog's bar
are 36 cells wide, so those boxes are 40 cells unless a longer line
widens them, and they shrink to a narrower box. A dialog is centered on
the screen, or placed where its
`DialogPosition` says, and painted whole even when the element that
renders the engine stands inside a box that clips its content. A dialog
the engine opens over another is placed one row lower than it.

## Confirmation dialog

`ConfirmationDialog` asks a question and closes on a button. Build it with
`confirmation_dialog()` (`.title()`, `.message()`, `.confirm_text()`,
`.cancel_text()`, `.danger(true)` for a red confirm button) or through
`DialogEngine::show_confirmation` with `ConfirmationDialogOptions`. It has
no icon unless `icon` names one. Enter presses the button that holds the
focus, Tab moves between the buttons, Escape cancels when
`escape_closable` is set. The screen reader hears it as an alert dialog
labeled by its title.

## Input dialog

`InputDialog` edits one field and validates it. Build it with
`InputDialog::new(id, InputDialogOptions { .. })` and render it through
`DialogComponent::render`, or open it with `DialogEngine::show_input`. The
field fills the box; an error under it is `text-error`, a warning
`text-warning`. OK is the primary button, Cancel the other. The `bg-` and
`text-` classes of the options' `input` css class reach the field's cells
after the theme's roles, so an application's colors win there.

## Autocomplete dialog

`AutocompleteDialog` filters suggestions as the user types. Build it with
`AutocompleteDialog::new(id, AutocompleteDialogOptions { .. })` and
`DialogComponent::render`, or open it with `DialogEngine::show_autocomplete`.
The selected suggestion is painted `bg-selection text-selection-foreground`.

## Progress dialog

`ProgressDialog` shows a bar and a percentage. Build it with
`progress_dialog()` (`.title()`, `.message()`, `.progress(0.64)`,
`.indeterminate(true)` for a bar that moves) or open it with
`DialogEngine::show_progress` and update it with
`DialogUpdate::Progress`. Its Cancel button, when `cancellable`, is a
secondary button.

## Toast

`Toast` shows a message for a while. Build it with `toast()` (`.success()`,
`.error()`, `.warning()`, `.info()` or `.message()` with `.toast_type()`,
`.duration(ms)` or `.persistent()`, `.position("top-right")`,
`.closable(true)`) or show it with `DialogEngine::show_toast`. A toast is
painted in the fill of its kind with that fill's text: `success`,
`warning`, `error` or `info`; `ToastType::Custom(classes)` paints the
classes its string names. It sits at the corner or edge its position
names with one cell between it and the screen's edge; toasts the engine
shows at the same position stack, each under the earlier ones (or over
them at a bottom position), one row apart. A warning or an error is an
alert to the screen reader, the rest a status, labeled by the kind and
described by the message. Escape closes a closable toast that holds the
focus.

## Wizard dialog

`WizardDialog` moves through steps. Build it with `wizard()` (`.title()`,
`.step(WizardStep::new("name").content(element))`, `.cancelable(false)`)
or open it with `DialogEngine::show_wizard`. Next and Finish are the
primary button; Back, Skip and Cancel are secondary.

## Network access

Dialogs make no network request unless the application sets an endpoint.
`ValidationConfig::async_validation_url` enables input validation requests on
submit and on any configured change or blur trigger.
`AutocompleteConfig::suggestions_url` enables suggestion requests after the
configured minimum input and debounce delay.

Remote helpers require curl 8.4 or newer. On Windows the client tries the
system's own curl (`%SystemRoot%\System32\curl.exe`) first and the first curl
on `PATH` second, and runs the first of the two that is 8.4 or newer; elsewhere
it runs `curl` from `PATH`. They send JSON with HTTP or HTTPS
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
- Dialog theme and its looks: [`src/widgets/dialog/mod.rs`](../src/widgets/dialog/mod.rs),
  [`src/widgets/dialog/dialog_types.rs`](../src/widgets/dialog/dialog_types.rs)
- The box every dialog is drawn in: [`src/widgets/display/modal.rs`](../src/widgets/display/modal.rs)
- Overlay contract tests: [`tests/overlays_contract.rs`](../tests/overlays_contract.rs)
- Input validation: [`src/widgets/dialog/input/validation.rs`](../src/widgets/dialog/input/validation.rs)
- Dialog behavior tests: [`tests/api_widget_behavior/dialogs.rs`](../tests/api_widget_behavior/dialogs.rs)
- Dialog lifecycle tests: [`tests/api_dialog_lifecycle.rs`](../tests/api_dialog_lifecycle.rs)

## Related chapters

- [Menus](menus.md)
- [Reactive state and hooks](reactive-state-and-hooks.md)
- [Animation and screens](animation-and-screens.md)

[Back to the manual](README.md)
