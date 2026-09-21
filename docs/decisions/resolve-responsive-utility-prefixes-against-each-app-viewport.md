# Resolve responsive utility prefixes against each App viewport

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-009
Would be wrong if: A responsive grid uses the same column count on both sides of its breakpoint, variants leak across Apps, resize leaves stale geometry, or search input remains invisible at 40 columns.
History: Earlier styling recovery established per-App state resolution before layout. Responsive prefixes currently ignore their breakpoint and must use the same owned path.

## Decision

Resolve sm, md, lg and xl class prefixes from each App terminal width before layout, at 40, 80, 120 and 160 columns respectively. Prefixes without an App viewport remain inactive in standalone class parsing. Preserve their ordering with focus, hover and disabled variants. Keep responsive_grid returning ElementBuilder and verify one column below md and the requested count at or above md, including resize. Reduce the search_input left/right padding to four/two cells and place its icon within that inset so the default helper has editable text at 40 columns. Do not change the global spacing scale.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/builder/layout.rs`, `src/builder/core.rs`, `src/layout/css/variants.rs`.

Behavior checks: `tests/api_widget_behavior/core_builders.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
