# Restore nontrapping overlay focus through the App focus owner

Level: Judged
Decided by: Codex
Rests on: API-011,API-006
Would be wrong if: Autofocus targets a guessed or unexpanded child, closing steals focus after the user left the overlay, or a nested trap is bypassed.
History: Previous API reversals require direct behavioral evidence. This is Judged because the existing App focus owner remains authoritative and public focus fields do not change; test nontrapping traversal, removal, keyed redraw and nesting.

## Decision

Add private focus-scope metadata for overlays that request autofocus without trapping Tab. Collect eligible descendants after component expansion. The App focus owner saves the current target on scope entry, chooses the first eligible descendant, and restores the saved target on close only if focus was still inside that scope. Preserve trap confinement and keyed scope identity. Record focus before event-tree removal so closing a focused descendant does not lose the restoration decision.

## Realized by

Implementation: `src/app/focus_manager.rs`, `src/widgets/display/popover`.

Behavior checks: `tests/api_widget_behavior/popover.rs`, `tests/api_widget_behavior/modal.rs`, `tests/api_widget_behavior/menus.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
