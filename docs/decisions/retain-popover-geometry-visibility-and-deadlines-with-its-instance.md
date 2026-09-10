# Retain popover geometry visibility and deadlines with its instance

Level: Judged
Decided by: Codex
Rests on: API-011,API-005,API-006
Would be wrong if: Placement uses guessed bounds, a public visibility method cannot wake App, callbacks deadlock on reentry, or deadlines survive removal.
History: Earlier clipboard and accessibility reversals require direct App evidence for ownership and delivery. This remains Judged: private retained state and existing layout, routing, scheduler and focus owners preserve public signatures; measured frame tests and negative probes determine acceptance.

## Decision

Keep the public Popover and builder routes. Use private shared instance state for measured trigger and content geometry, authored visibility changes, and imperative visibility requests. Render the trigger while closed and position actual expanded content against the presented clipping rectangle. Capture trigger input before consuming child controls, use an outside-click shield, and invoke callbacks after releasing state locks. Own hover and animation deadlines in the component scheduler and cancel them on unmount. A retained private child supplies that lifecycle even when callers render the public Popover manually and retain its imperative handle after removal. Apply existing declarative focus traps to visible content; complete first-descendant autofocus and focus restoration through the App focus owner. Preserve existing state fields as observed values and use terminal cells for geometry.

## Realized by

Implementation: `src/widgets/display/popover`.

Behavior checks: `tests/api_widget_behavior/popover.rs`, `tests/api_widget_behavior/orca_overlays.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
