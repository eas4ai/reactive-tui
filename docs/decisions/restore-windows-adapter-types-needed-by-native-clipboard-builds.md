# Restore Windows adapter types needed by native clipboard builds

Level: Judged
Decided by: Codex
Rests on: API-008,API-019
Would be wrong if: The adapter still fails the native build, loses its public entry points, or the repaired event conversion changes tested key, mouse, resize or focus behavior.

## Decision

Repair the existing Windows Console adapter so the real library can run the clipboard hook on Windows. Retain public entry points, route PlatformTty methods to their existing implementations, use the locked windows-sys types and required features, and consolidate duplicate record converters into the platform TerminalEvent format. Preserve key press/release, character and function-key conversion; provide an actual bounded byte-read path for existing terminal queries. Verify event conversion and raw-read failure/timeout behavior on Windows. This prerequisite does not claim that all remaining legacy terminal behaviors in API-019 are accepted.

## Realized by

- 9779c16 — restore native adapter types, trait forwarding, UTF-16 event conversion and bounded character reads.
- c94d9dd — preserve inherited test handles when the private console closes.
- 04885d5 — consume native console input records with matching console modes.
- Native run 34313628354 at 09f1435 passed the Windows adapter fixture and real clipboard checks.
