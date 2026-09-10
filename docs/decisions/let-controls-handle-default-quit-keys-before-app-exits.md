# Let controls handle default quit keys before App exits

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Escape cannot close a menu, copy terminates a focused editor, or an unhandled default quit key stops exiting the application.
History: Prior API reversals required measured clipboard deadlines and explicit cancellation ownership. This choice likewise follows actual App input probes and stays Judged: it restores advertised control input without removing application quit support.

## Decision

Route input to the focused component and then the root before applying the default Escape and Ctrl-C quit fallback. A control that consumes Escape can close its popup; a text input can consume Ctrl-C for copy. Unhandled default quit keys still exit. Preserve explicitly configured quit keys as global bindings. Add App acceptance coverage for consumed Escape and fallback exit.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/app.rs`.

Behavior checks: `tests/api_widget_behavior/table.rs`, `tests/api_widget_behavior/data_table.rs`, `tests/api_widget_behavior/terminal.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
