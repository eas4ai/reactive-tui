# Deliver widget callback IDs to the App root after input dispatch

Level: Judged
Decided by: Codex
Rests on: API-005, API-011
Would be wrong if: A notification is lost, duplicated, leaks between Apps, or recursively delivers input while a widget instance is borrowed.
History: The existing callback ownership decision requires exactly-once App delivery and isolation. This additive route completes retained string IDs without changing public prop types.

## Decision

Preserve string callback IDs in widget props. Collect synchronous widget notifications in a scoped input-dispatch frame, then deliver each as a CustomEvent to RootComponent after routing releases widget borrows. The event name is the configured callback ID and its payload is documented per widget. Scope entry and exit restore nested App isolation and discard undelivered notifications on errors. Do not retain an asynchronous global event queue or recursively dispatch from a widget callback.

## Realized by

Implementation: `src/event/notifications.rs`, `src/app.rs`.

Behavior checks: `tests/api_widget_behavior/accordion.rs`, `tests/api_widget_behavior/breadcrumb.rs`, `tests/api_widget_behavior/file_explorer.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
