# Resolve animation endpoints through owner-scoped presented targets

Level: Judged
Decided by: Codex
Rests on: API-013
Would be wrong if: A relative endpoint uses a guessed value, an escaped handle binds a replacement target, owners share IDs, or playback samples candidate properties before successful presentation.
History: Prior API reversals concerned clipboard bounds, ConPTY ownership, terminal reflow and accessibility coordinates. This design follows the approved target-handle choice, avoids a global ID registry, uses acknowledged styles, and tests removal, duplicate IDs and independent owners.

## Decision

Implement approved api-013-2 using a per-App and per-screen target registry. Publish resolved property snapshots only after presentation. Handles hold weak references to one target generation and fail after removal or owner shutdown. Resolve current-value endpoints on playback start, retain them while paused, and resolve again on restart. Add checked construction/playback errors; preserve existing explicit from-to ID construction. Apply bound numeric animation samples through the existing style painter and request redraw through the owner waker. Custom numeric current values must be explicitly declared by the element, never inferred from erased component props.

## Realized by

(none yet: recorded, not built)
