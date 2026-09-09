# Own hook resources and inherited context in component scopes

Level: Judged
Decided by: Codex
Rests on: API-004
Would be wrong if: Effects or timers survive component removal, unchanged timer dependencies postpone deadlines, context leaks between siblings or Apps, or valid standalone hooks lose cleanup.

## Decision

Give each App root and keyed component an owned resource scope, with a temporary thread-local binding during rendering and lifecycle callbacks. The scope supplies the App scheduler and inherited context, tracks hook cleanup, and closes before removal. Hooks retain cleanup in positional effect slots and run pending effects after a successful generated render; use_effect runs each render because it has no dependency list. Add an explicit dependency-aware effect hook for callers and built-in timers. Timer duration changes restart timers; unchanged duration preserves deadlines while callbacks refresh. Timer handles capture their scheduler, and removed owners reject further scheduling. Context providers apply to the rendered descendants, with nearest-provider overrides and fresh per-render bindings. Standalone Hooks retain cleanup until cleanup or final drop and keep the existing standalone scheduler fallback. Keep the public standalone ReactiveRuntime API separate from App-owned hook resources.

## Realized by

- 0e6d00d46d8f4310d29c0eb7d8fa6d987ab82e00 Own component effects timers and inherited context in App scopes
