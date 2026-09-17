# Report restored-session panics through the owned backend channel

Level: Judged
Decided by: Codex
Rests on: TRL-001, TRL-002, DQC-003, 20260917T115325551Z-1107399
Would be wrong if: Panic reporting occurs before restoration, changes the propagated payload, repeats a prior panic hook, prints normal diagnostics to process streams, or breaks existing backend implementations.

## Decision

The DQC-003 logging repair removed the replay that made panic text visible after alternate-screen cleanup. Preserve logging for normal diagnostics and preserve the existing panic hook exactly once. Add a defaulted Backend panic-shutdown method, with built-in terminal backends forwarding a failure-only message through their owned renderer or TTY writer after screen restoration. Keep existing custom backends source-compatible; their default shuts down and uses logging. App retains its backend until this cleanup completes, then drops its owned resources and resumes the original panic payload. Renderer workers retain their owned writer across catch_unwind so worker failures can replay there after the session guard restores output. Do not add a process-global output logger, suppress hooks, manufacture a second panic, or bypass the captured diagnostics check. Verify both actual PTY panic cases, prior-hook/cross-thread ownership cases, owned-writer failure regressions and the normal-operation diagnostics gate.

## Realized by

(none yet: recorded, not built)
