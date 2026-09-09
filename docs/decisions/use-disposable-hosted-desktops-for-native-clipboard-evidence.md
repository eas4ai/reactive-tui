# Use disposable hosted desktops for native clipboard evidence

Level: Judged
Decided by: Codex
Rests on: API-008
Would be wrong if: A hosted job cannot use the native clipboard or its evidence does not match the committed implementation.

## Decision

Run the existing macOS and Windows clipboard probes on disposable GitHub-hosted desktops from a separate CI branch. Keep workflow permissions read-only, pin action commits, bound job runtime, and collect evidence artifacts without modifying the default remote branch. Add native build tools required by the existing locked dependencies. Platform acceptance still requires the real hook round trips and process lifecycle checks; compilation alone is insufficient.

## Realized by

- 81bfe9953b1b4a082b584fd3851b014d79753615 Run clipboard evidence on disposable native CI desktops
- 000fa6b5f1523a1a67a61bc561723eec6abefdad Preserve complete native build diagnostics in CI artifacts
