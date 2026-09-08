# Validate generated hook frames and refresh memo values on render

Level: Judged
Decided by: Codex
Rests on: API-003
Would be wrong if: Valid repeated renders lose state or allocate new slots, memo handles remain stale, or invalid hook ordering silently creates disconnected state.

## Decision

Generated component renders enter an RAII hook frame that resets the index and validates positional hook count, kind and stored type. Report invalid order and overlapping renders as explicit programmer errors rather than returning isolated fallback state. Preserve manual Hooks::reset for callers managing their own render boundary. The current use_memo signature has no dependency list: recompute once when called during each render, retain its output signal, and publish only changed values. State, memo, reducer and previous-value slots have distinct kinds. Identically typed calls still follow positional identity, so callers must keep their order stable. Effect/context lifecycle is the separate API-004 requirement.

## Realized by

(none yet: recorded, not built)
