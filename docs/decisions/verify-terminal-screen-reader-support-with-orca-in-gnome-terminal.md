# Verify terminal screen-reader support with Orca in GNOME Terminal

Level: Consequential
Decided by: Developer
Rests on: API-011, API-018
Would be wrong if: The implemented path cannot deliver labels, roles, focus and state changes to Orca, or documentation claims verification for other terminal and reader pairs.
History: The prior API decisions preserve public APIs and require real platform evidence. This explicit developer approval narrows only the screen-reader guarantee to a named terminal and reader pair.

## Decision

The developer approved escalation api-011-api-018 on 2026-09-09. Replace the blanket screen-reader claim with verified Orca support in GNOME Terminal on Linux. Preserve public label APIs and existing terminal rendering and input guarantees. Implement and test actual reader delivery of labels, roles, focus and state announcements, including distinct accessible labels, nested controls, hidden content and removal. Other terminal and screen-reader pairs remain explicitly unverified. Painting text or retaining metadata alone is not acceptance evidence.

## Realized by

Implementation: `src/accessibility`, `tests/api_widget_behavior/accessibility_probe.rs`.

Behavior checks: `tests/api_widget_behavior/orca.py`, `tests/api_widget_behavior/orca_data.py`, `tests/api_widget_behavior/orca_tabs.py`, `tests/api_widget_behavior/orca_menus.py`, `tests/api_widget_behavior/orca_overlays.py`, `tests/api_widget_behavior/orca_dialogs.py`, `tests/api_widget_behavior/orca_display.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
