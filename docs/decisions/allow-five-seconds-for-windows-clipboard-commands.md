# Allow five seconds for Windows clipboard commands

Level: Judged
Decided by: Codex
Supersedes: bound-clipboard-commands-and-tie-cancellation-to-hook-ownership
Cause: an unforeseen condition occurred
Rests on: API-008
Would be wrong if: A stalled command exceeds its platform deadline, cancellation leaves an owned child running, or normal Windows startup still exceeds the allowance.

## Decision

Keep the bounded clipboard ownership, temporary streams, transfer limit, error handling, cancellation and native evidence requirements from the earlier decision. Use a two-second deadline on Unix and a five-second deadline on Windows. The first native PowerShell startup exceeded two seconds before writing its readiness marker, while the subsequent warm timeout case passed. Five seconds permits that observed startup cost and still places a fixed bound on synchronous Windows calls. Keep independent six-second Windows and three-second Unix test limits, and retain direct-child exit checks.

## Realized by

(none yet: recorded, not built)
