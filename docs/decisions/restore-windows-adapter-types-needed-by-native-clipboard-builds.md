# Restore Windows adapter types needed by native clipboard builds

Level: Judged
Decided by: Codex
Rests on: API-008,API-019
Would be wrong if: The adapter still fails the native build, loses its public entry points, or the repaired event conversion changes tested key, mouse, resize or focus behavior.

## Decision

Repair the existing Windows Console adapter so the real library can run the clipboard hook on Windows. Retain public entry points, route PlatformTty methods to their existing implementations, use the locked windows-sys types and required features, and consolidate duplicate record converters into the platform TerminalEvent format. Preserve key press/release, character and function-key conversion; provide an actual bounded byte-read path for existing terminal queries. Verify event conversion and raw-read failure/timeout behavior on Windows. This prerequisite does not claim that all remaining legacy terminal behaviors in API-019 are accepted.

## Realized by

(none yet: recorded, not built)
