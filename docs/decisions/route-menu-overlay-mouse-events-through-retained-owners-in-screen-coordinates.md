# Route menu overlay mouse events through retained owners in screen coordinates

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Mouse events disagree with painted overlay bounds, callbacks lose their App scope, an owner survives unmount, or keyboard routing changes.
History: The three API reversals showed that platform timing and accessibility geometry need real boundary evidence. This change follows failing App mouse probes and the observed local-cell rejection in ComponentRuntime; it does not broaden platform claims.

## Decision

Wrap the private menu runtimes in one shared retained adapter. Attach a raw mouse listener to their rendered root so bubbling events from measured overlays reach the owner even outside its natural layout rectangle. Keep original screen coordinates for measured hit testing, enter the captured component scope for callbacks, and retain normal component routing for keyboard and focus events. Share this adapter between menubar, popup and dialog menus; context-menu triggers already fill their allocated region. Verify outside dismissal, nested hover, measured item clicks, resize, unmount and callback counts through App.

## Realized by

Implementation: `src/widgets/menu`.

Behavior checks: `tests/api_widget_behavior/menus.rs`, `tests/api_widget_behavior/orca_menus.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
