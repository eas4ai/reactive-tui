# Run Tokio lifecycle routes without nested block_on

Level: Judged
Decided by: Codex
Rests on: RTR-003
Would be wrong if: Aborting the synchronous stop path leaves an input task active or a shared launch path changes async lifecycle behavior.

## Decision

Make input-task creation synchronous because it only validates state and calls tokio::spawn. The sync and async start routes use that shared launch path, so neither needs block_on. The sync start route returns an error when no runtime is active. The async stop route signals shutdown and awaits graceful completion. The sync stop route signals shutdown, aborts the task, clears the running flag, and returns without blocking. This keeps every public lifecycle route safe inside current-thread and multi-thread Tokio runtimes.

## Realized by

- 790ebb2d2228c04f62b0cac37939a3437505e08a fix: avoid nested Tokio runtimes

Implementation: Task creation is synchronous and shared by both start routes. A running-state guard clears the flag when the task completes or is aborted. Both stop routes share shutdown signaling and task ownership; sync stop aborts without blocking, while async stop awaits the task.

Behavior check: The RTR-003 fixture exercises sync and async start and stop inside current-thread and multi-thread Tokio runtimes, enforces bounded completion, and fails on any panic.
