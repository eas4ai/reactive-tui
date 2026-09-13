# Dispatch owned Updater callbacks on the App thread

Level: Judged
Decided by: Codex
Rests on: API-019 API-020
Would be wrong if: Requests are lost, callbacks cross Apps, reentry runs in the same batch, or cleanup leaves a live request target.
History: The developer explicitly approved the breaking Updater method migration in api-019-api-020-2; retain all stated ownership and dispatch guarantees.

## Decision

Implement the approved required update method returning Result<()>. App registration owns Send + Sync updaters; a separate registration token cancels delivery on drop and cloneable weak request handles coalesce pending work. Snapshot pending flags before invoking callbacks, without holding state locks during user code. Dispatch before initial render and on subsequent loop turns; prune cancelled registrations and close all states before dropping callbacks.

## Realized by

- 99c4c0f358dcb58c6e1a4010107592d42bb6f27a Dispatch owned Updater callbacks through App wake requests
