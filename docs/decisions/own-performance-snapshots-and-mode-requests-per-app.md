# Own performance snapshots and mode requests per App

Level: Judged
Decided by: Shawn and Codex
Rests on: API-004 API-016 API-019 API-020
Would be wrong if: Apps share metrics or mode requests, callbacks schedule after owner closure, context changes alter hook order, or retained public provider literals stop compiling.
History: The developer approved api-004-api-016-api-019-api-020 after the two-App probe demonstrated shared signals and unowned mode requests.

## Decision

Construct a private performance owner with each App and expose its existing public PerformanceContext through App::performance_context. Supply this context after entering each root render scope, preserving descendant overrides. Keep a bounded latest-mode request per owner, wake that App when requested, and make escaped setters inert after owner closure through weak ownership. Publish actual mode, render cost and elapsed presentation cadence. Completed measurements notify other subscribed Apps while excluding their own App from redraw. Drain coalesced mode requests before deciding whether the net mode change requires a render. Fixed modes retain their selected target; only Auto adapts. Reserve stable fallback hook slots regardless of context availability. Retain global context and request functions only for standalone callers; Apps never read or write them. Preserve public context fields and verify sequential, nested, concurrent and closed-owner behavior.

## Realized by

(none yet: recorded, not built)
