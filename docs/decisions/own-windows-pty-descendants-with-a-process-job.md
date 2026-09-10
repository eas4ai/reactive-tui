# Own Windows PTY descendants with a process job

Level: Judged
Decided by: Codex
Rests on: API-011 API-019
Would be wrong if: A child can escape before assignment, normal exit codes change, unrelated processes are terminated, or descendant and output-worker shutdown remain unbounded.
History: Native SDK run 34480201771 proves console closure returns, the direct child exits and the input worker joins, but the output reader blocks when a descendant exists. The previous direct-process owner depended on console closure to terminate all descendants; that dependency failed.

## Decision

Create a private Windows job with kill-on-close for each retained PTY session. Launch the direct child suspended, assign it to that job before resuming its primary thread, and fail launch with complete cleanup if assignment or resume fails. Terminate the owned job during shutdown before joining output; retain the directly observed normal exit code and wait boundedly for owned process termination. Do not enable breakaway or enumerate unrelated processes. Keep the existing bounded IO queues and SDK console ownership. Verify the existing native descendant, full-output, idle, failed-launch and handle-count cases without extending their deadlines.

## Realized by

`src/terminal/pty/windows/job.rs` owns the private job; `native.rs` creates the child suspended, assigns it before resume and terminates the owned tree before console closure and output join. Native run 34481972388 passes all PTY behavior cases, including descendants and stable failed-launch handles. Its later negative build failed because the test source copy omitted benches; that harness correction is being rechecked.
