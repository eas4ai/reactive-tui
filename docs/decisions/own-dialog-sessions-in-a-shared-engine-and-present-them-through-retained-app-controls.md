# Own dialog sessions in a shared engine and present them through retained App controls

Level: Judged
Decided by: Codex
Rests on: API-012
Would be wrong if: A result or close event is lost, callbacks run under an engine lock, live edits reset, unread events grow without bound, or a removed host retains dialog work.
History: Earlier API reversals exposed unverified host assumptions and incomplete lifecycle boundaries. This remains Judged because it preserves the agreed public behavior and uses the established App controls; acceptance must attack result loss, reentry, bounded queues and cleanup at the real App boundary before claiming completion.

## Decision

Replace the private construction-only engine store with shared owned dialog sessions. Keep existing show and close signatures; add fallible show methods, use DialogId zero for rejected legacy show calls, and expose the rejection reason. Render sessions as keyed retained App controls so measured routing, validation, timers and focus restoration remain shared with the working widgets. Close through one idempotent path, release the engine lock before callbacks, retain synchronous Opened and Closed events, and provide awaitable one-result handles and an asynchronous event receiver without requiring Tokio. Reserve close-event capacity when accepting each dialog and reject new opens when the bounded event backlog is full. Apply engine stacking and configuration at the shared modal boundary. Removing the mounted engine host cancels its sessions; independent engines remain isolated. Verify result delivery, callback reentry, overflow and limits, z-order, resized pointer input, focus restoration, cancellation and owned-task cleanup with actual App frames and safe violating fixtures.

## Realized by

a1e2a83adb242190aa04c472521cf8dff5240e4f — Implement owned dialog sessions and App lifecycle results
