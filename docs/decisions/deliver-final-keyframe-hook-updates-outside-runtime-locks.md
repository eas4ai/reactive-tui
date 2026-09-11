# Deliver final keyframe hook updates outside runtime locks

Level: Judged
Decided by: Codex
Rests on: API-013
Would be wrong if: A keyframe hook misses its final value, runtime IDs collide after cancellation, callbacks deadlock when starting or stopping animation, or unmounted keyframe hooks keep producing updates.
History: Prior API reversals concern clipboard and terminal ownership and accessibility. This repair follows the same requirement to prove cancellation and real completion; it does not change their mechanisms. Keyframe sampling compatibility is already approved in api-013.

## Decision

Repair the shared hook runtime dependency used by KeyframeHandle: assign monotonically increasing IDs, snapshot pending updates, invoke callbacks without a registry lock, and remove completed tasks only after final delivery. Give keyframe hooks a retained owner with component cleanup and weak runtime callbacks, so re-rendering keeps the active sequence and removal cancels it even when a handle escapes. Preserve play and seek and add explicit stop if needed for ownership. Demonstrate final delivery, cancellation isolation and callback reentry with bounded tests, then prove keyframe values and cleanup through App.

## Realized by

src/hooks/animation.rs retains keyframe owners, cancels playback at cleanup and delivers callbacks outside runtime locks. Unit tests cover final delivery, ID isolation, callback/drop reentry, cancellation and aborted renders; tests/api_hook_lifecycle.rs verifies actual App frames and removal.
