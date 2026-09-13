# Pin an isolated libatspi state-set lifetime repair

Level: Judged
Decided by: Codex
Rests on: API-011, API-019, API-020, escalation api-011-api-020
Would be wrong if: The pinned repair does not reproduce and eliminate the confirmed state-set use-after-free, changes the installed desktop libraries, or fails unchanged full Orca workflows.
History: Earlier API reversals showed that successful local behavior is insufficient when ownership, native hosts, or screen-reader delivery can diverge. They keep this decision at Judged level and require an isolated host library, an independent memory-failure demonstration, unchanged full workflows, and no system installation.

## Decision

Build libatspi 2.60.6 from the official GNOME source archive with its published SHA-256, patch only the state-set lifetime used across reentrant D-Bus refresh, and load it only inside the repository-owned accessibility fixture. Prove the original failure and corrected case under memory checking, verify public ABI and reference balance, then run the unchanged full Orca workflows. Keep the desktop installation untouched and retain the patch until an upstream release passes the same checks.

## Realized by

- 41e680882a5cce0011954ed7b21511ead62b8272 Repair isolated libatspi state-set lifetime
