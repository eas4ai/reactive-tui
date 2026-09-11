# Route screen input through retained component and presented event state

Level: Judged
Decided by: Codex
Rests on: API-013
Would be wrong if: Active screens still discard callbacks, hit bounds differ from presented cells, keyed screen state resets during updates, or removed screens leave runnable resources.
History: The API domain reversals concern clipboard deadlines, ConPTY ownership, terminal reflow and AT-SPI coordinates. None defines standalone screen event ownership; this decision preserves their existing mechanisms and uses the already verified App event and focus implementation.

## Decision

Give each screen a retained component runtime, scheduler, resource scope, event tree and focus manager. Reuse the App event and focus machinery with crate-only visibility instead of a second callback dispatcher. Publish targets only after the backend acknowledges the frame; route non-hotkey input only to the active screen and repaint handled changes. Keep public screen constructors, lifecycle hooks and legacy render-tree access. On screen removal, close owned resources. Use complete Element frames where supported and real tree differences for the legacy backend fallback. Prove keyboard, mouse, resize, keyed updates, inactive-screen isolation and cleanup against actual SuprTUI output before acceptance.

## Realized by

(none yet: recorded, not built)
