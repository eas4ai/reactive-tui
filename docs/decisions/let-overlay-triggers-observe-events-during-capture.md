# Let overlay triggers observe events during capture

Level: Judged
Decided by: Codex
Rests on: API-011,API-005
Would be wrong if: A trigger observes activation twice, suppresses the child callback, or keeps an event handler after removal.
History: Earlier clipboard deadline and AT-SPI translation reversals require direct behavioral evidence rather than inferred support. This change is Judged because it uses the existing event phase and private metadata without changing public contracts; test actual capture ordering and cleanup.

## Decision

Add crate-private capture handlers to owned Element metadata and register them in the existing router capture phase. Preserve them through component expansion. Popover trigger wrappers can observe activation before child controls consume it, returning Handled so the child callback still runs. Outside-click shields may consume events. Keep public APIs unchanged and verify ordering, redraw replacement, disabled and inert suppression, and removal before using this path for overlays.

## Realized by

Implementation: `src/app/event_tree.rs`, `src/widgets/display/popover`.

Behavior checks: `tests/api_widget_behavior/popover.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
