# Preserve AT-SPI state and focus actions without invented pixel bounds

Level: Judged
Decided by: Codex
Supersedes: retain-a-narrow-at-spi-state-translation-repair-with-upstream-provenance
Cause: an unforeseen condition occurred
Rests on: API-011, API-018
Would be wrong if: The adapter fabricates pixel geometry, focus actions activate controls, unrelated adapter behavior changes, or real Orca delivery is not verified.
History: The real App workflow reached the labelled nested entry but its Component interface was absent. Upstream exposes that interface only for a root or a node with raw pixel bounds. The previous prototype did not exercise assistive focus on a child.

## Decision

Retain the reviewed expanded and disabled state translations. Also expose the AT-SPI Component interface for nodes that support a Focus action, even when screen-pixel bounds are unavailable. The existing extents implementation returns invalid bounds in that case; preserve that behavior. This makes the existing grab_focus action reachable for real terminal controls without pretending terminal cells are pixels. Add regression cases for focus-only nodes and exercise child and composite-header focus through the real App in the isolated Orca workflow. Keep upstream provenance, licenses and the exact maintained delta.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/accessibility/platform/translation`.

Behavior checks: `src/accessibility/platform/translation/state_tests.rs`, `tests/api_widget_behavior/orca.py`, `tests/api_widget_behavior/orca_data.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
