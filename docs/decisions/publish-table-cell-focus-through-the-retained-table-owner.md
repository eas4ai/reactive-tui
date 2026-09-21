# Publish table cell focus through the retained table owner

Level: Judged
Decided by: Codex
Rests on: API-011 API-018
Would be wrong if: Reader focus activates a row action, changes unrelated selection, loses stable row identity, or Orca still fails to announce keyboard navigation.
History: The real Orca table probe sees table and cell roles but does not announce movement to the next enabled row. Earlier accessibility repairs established owned virtual focus for composite controls; this applies that route to existing table cells.

## Decision

Expose the current table cell as the accessible focus while the table owns keyboard focus. Keep cell identity under the stable row ID. Route assistive cell focus to the existing retained table cursor without invoking row actions or toggling multi-selection. Publish selected and disabled state and cell labels with their original text. Preserve keyboard navigation, column scrolling and measured mouse actions. Verify real Orca announcements for navigation and assistive focus, disabled rows and removal, then rerun Table and DataTable App cases.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/display/table`, `src/widgets/display/data_table`.

Behavior checks: `tests/api_widget_behavior/orca_data.py`, `tests/api_widget_behavior/table.rs`, `tests/api_widget_behavior/data_table.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
