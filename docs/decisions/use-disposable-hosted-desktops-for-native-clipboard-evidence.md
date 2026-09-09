# Use disposable hosted desktops for native clipboard evidence

Level: Judged
Decided by: Codex
Rests on: API-008
Would be wrong if: A hosted job cannot use the native clipboard or its evidence does not match the committed implementation.

## Decision

Run the existing macOS and Windows clipboard probes on disposable GitHub-hosted desktops from a separate CI branch. Keep workflow permissions read-only, pin action commits, bound job runtime, and collect evidence artifacts without modifying the default remote branch. Add native build tools required by the existing locked dependencies. Platform acceptance still requires the real hook round trips and process lifecycle checks; compilation alone is insufficient.

## Realized by

(none yet: recorded, not built)
