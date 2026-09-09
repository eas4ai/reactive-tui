# Bound clipboard commands and tie cancellation to hook ownership

Level: Judged
Decided by: Codex
Rests on: API-008
Would be wrong if: A stalled tool blocks past the operation deadline, cleanup leaves an owned process running, nonzero exit reports success, or a platform claim lacks real integration evidence.

## Decision

Keep synchronous clipboard hook signatures and cached error state. Detect executables directly without spawning which. Run each command with a two-second deadline and cancellation from the existing weak hook resource token. Use private anonymous temporary files for standard streams so full or inherited pipes cannot block the caller; promote the already locked tempfile dependency and cap text transfers at 64 MiB with explicit errors. Own, kill and reap the direct child on failure, timeout or cancellation, and stop its Unix process group on those failures. Successful clipboard tools may retain their own clipboard-serving background process. Preserve text and trailing newlines; pass Windows text through UTF-8 stdin to a fixed PowerShell command rather than interpolating it. Keep backend choices and test Linux tools against isolated desktop servers. Require actual macOS and Windows evidence for their existing claims; fixtures alone are not platform acceptance.

## Realized by

(none yet: recorded, not built)
