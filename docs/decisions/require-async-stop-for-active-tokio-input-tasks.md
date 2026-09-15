# Require async stop for active Tokio input tasks

Level: Judged
Decided by: Codex
Supersedes: run-tokio-lifecycle-routes-without-nested-block-on
Cause: the stated condition occurred
Rests on: RTR-003
Would be wrong if: Returning an error from synchronous stop prevents callers from reaching graceful async cleanup or breaks inactive-loop shutdown.
History: The superseded decision aborted an active input task from synchronous stop. The post-review RTR-003 check reached both test markers but the multi-thread runtime did not exit before the 300-second deadline because Tokio's stdin blocking worker remained active. This is the superseded decision's stated wrong condition.

## Decision

Keep synchronous task creation so sync and async start never enter a nested runtime. When no input task is active, synchronous stop succeeds. When a task is active, synchronous stop returns an error directing the caller to stop_async and leaves the task handle available for that cleanup. stop_async signals shutdown and awaits graceful task completion. No lifecycle route calls block_on, and the public async route remains the owner of bounded task joining.

## Realized by

(none yet: recorded, not built)
