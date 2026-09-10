# Expose ScrollView as an AT-SPI scroll pane

Level: Judged
Decided by: Codex
Rests on: API-011,API-018
Would be wrong if: A scroll control is still reported as a generic section or panel, its public label is lost, or reader focus cannot reach its scrolled child.
History: The maintained AccessKit translation already carries reviewed role, state and focus repairs. ScrollView currently lacks a semantic role; the inherited ScrollView mapping also collapses it into Panel.

## Decision

Give the existing ScrollView element its AccessKit ScrollView role. Translate that role to the dedicated AT-SPI ScrollPane role while preserving Pane as Panel. Retain existing keyboard ownership, clipping and child controls. Verify the baseline role failure and corrected real Orca workflow with scroll-to-end, nested editable content and public labels at two viewport sizes; record the adapter delta with its existing provenance.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/layout/scroll_view.rs`, `src/accessibility/platform/translation`.

Behavior checks: `tests/api_widget_behavior/orca_display.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
