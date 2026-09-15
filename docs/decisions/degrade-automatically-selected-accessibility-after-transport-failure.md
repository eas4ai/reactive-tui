# Degrade automatically selected accessibility after transport failure

Level: Judged
Decided by: Codex
Supersedes: own-a-bounded-and-cancellable-screen-reader-transport-per-app
Cause: an unforeseen condition occurred
Rests on: RTR-001
Would be wrong if: Automatic accessibility failure can corrupt App state, hide an explicit request failure, or leave the transport worker running.
History: The earlier transport decision made every bus failure fatal. Fable found that stale inherited session-bus addresses are common after desktop-session changes and make an otherwise usable terminal App exit.

## Decision

Track whether accessibility was explicitly requested. If an automatically selected connection fails during publication, focus handling, action polling, startup, or shutdown, close and remove that connection, log the reason, and continue the App. Preserve the existing error result for screen_reader(true). Keep the render and input order unchanged.

## Realized by

(none yet: recorded, not built)
