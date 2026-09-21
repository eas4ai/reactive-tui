# Translate current item state and value for AT-SPI readers

Level: Judged
Decided by: Codex
Rests on: API-011, API-018
Would be wrong if: Current item is conflated with keyboard focus or selection, false state is exposed as active, or Orca cannot announce the current page.
History: Breadcrumb preserves AccessKit AriaCurrent::Page, but the maintained adapter does not translate it. Installed Orca 50.1.2 reads the active state and current object attribute together. The Core AAM mapping specifies the same pair.

## Decision

Extend the maintained AT-SPI translation with State::Active for nonfalse AriaCurrent values and the exact current object attribute token. Preserve the existing window and dialog active state behavior. Cover absent, false and every supported current token, and verify the actual current-page announcement using Orca object navigation in the isolated GNOME Terminal workflow. Keep the upstream delta documented. Mapping: https://w3c.github.io/core-aam/#ariaCurrent.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/accessibility/platform/translation`.

Behavior checks: `src/accessibility/platform/translation/state_tests.rs`, `tests/api_widget_behavior/orca.py`, `tests/api_widget_behavior/orca_data.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
