# Route explicit assistive activation to the retained control owner

Level: Judged
Decided by: Codex
Rests on: API-005 API-011
Would be wrong if: Assistive activation bypasses focus traps or disabled state, fires a control twice, or activates a geometrically unrelated child.
History: The expanded Select has a header and option rows under one retained owner. Its semantic combo action currently synthesizes a mouse press at the center of all rows, which can activate an option instead of closing the dropdown. The real Orca workflow reproduces the failure. Prior accessibility ownership decisions already route semantic focus through an explicit private event.

## Decision

Add an optional private click event alongside the existing accessibility focus event. Preserve it during component expansion and carry it in the presented action target. After the normal owner focus, disabled and trap checks, deliver that event through the existing router instead of synthesizing a mouse press. Keep the measured mouse fallback for controls without an explicit semantic activation. Select uses this route to toggle its dropdown; option actions retain their existing measured selection route. Verify expanded combo close/reopen through Orca, both viewport sizes, and the existing App input and focus tests.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/accessibility/connection.rs`, `src/app/event_tree/accessibility.rs`.

Behavior checks: `tests/api_widget_behavior/orca.py`, `tests/api_widget_behavior/orca_menus.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
