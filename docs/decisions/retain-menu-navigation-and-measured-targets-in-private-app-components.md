# Retain menu navigation and measured targets in private App components

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Menus use guessed click bounds, state leaks between Apps, callback delivery duplicates, or nested menus lose keyboard navigation and focus restoration.
History: The clipboard and AT-SPI reversals require testing behavior at its real boundary. This remains Judged because public widget shapes stay intact and actual App frame and input probes govern acceptance.

## Decision

Public menu components render private retained children carrying their configuration, state seed and callbacks. Retained owners keep selection paths and checkbox/radio edits per App, render real item elements, and measure their layout for pointer targets and dropdown placement. Share menu item state and rendering logic between menu families, using the existing layout, focus and overlay machinery. Remove competing placeholder event handlers after each family moves to the retained path. Verify nested keyboard and mouse selection, scrolling, disabled and empty items, callback replacement, prop updates and viewport changes through App.

## Realized by

Implementation: `src/widgets/menu`.

Behavior checks: `tests/api_widget_behavior/menus.rs`, `tests/api_widget_behavior/orca_menus.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
