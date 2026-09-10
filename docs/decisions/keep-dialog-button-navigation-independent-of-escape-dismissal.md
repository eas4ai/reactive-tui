# Keep dialog button navigation independent of Escape dismissal

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-012 API-006
Would be wrong if: A dialog escape_closable option is intended to disable Tab navigation and keyboard button activation as well as Escape dismissal.
History: Retained dialog adapters use the shared Modal renderer. They currently map escape_closable to Modal keyboard_navigation, which removes dialog buttons from keyboard focus when Escape dismissal is disabled.

## Decision

Keep the public ModalProps keyboard_navigation contract unchanged. Add a private Escape-dismissal policy to the retained modal adapter. Confirmation, input and autocomplete dialogs keep keyboard navigation enabled and pass their escape_closable flag only to this policy. Let inner controls handle Escape before the modal decides whether to dismiss, so autocomplete can still close its suggestion list. Verify disabled Escape followed by Tab and Enter through App, along with existing modal dismissal and nested-focus tests.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/display/modal.rs`, `src/widgets/dialog/confirmation/live.rs`, `src/widgets/dialog/input/live.rs`, `src/widgets/dialog/autocomplete/live.rs`.

Behavior checks: `tests/api_widget_behavior/confirmation.rs`, `tests/api_widget_behavior/input.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
