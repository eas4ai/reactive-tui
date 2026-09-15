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

- 7b045486182c6b1357c70a453dd3d2c1b4492746 fix: require async Tokio task shutdown

Implementation: Sync stop returns an error containing `stop_async` while an input task is active and preserves the handle. The public async route signals and awaits that task. The two-runtime fixture requires the sync error to remain recoverable through bounded async cleanup.

Behavior check: Both current-thread and multi-thread cases reach completion and the Cargo test process exits; timeout, panic, process failure, and missing completion remain failures.
