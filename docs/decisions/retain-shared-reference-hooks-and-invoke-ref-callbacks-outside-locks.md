# Retain shared reference hooks and invoke ref callbacks outside locks

Level: Judged
Decided by: Shawn and Codex
Rests on: API-019 API-004
Would be wrong if: Shared references lose identity across renders, callback replacement is stale, callback reentry deadlocks, or the change adds bounds to working reference APIs.
History: This follows the recorded positional hook storage and owned component-scope decisions. Local non-Send reference ownership remains a separate unresolved compatibility choice.

## Decision

Use distinct positional HookKind slots for shared Ref, CallbackRef and MultiRef, retaining their existing shared state without adding Clone bounds to use_ref. Refresh CallbackRef callback captures on render through shared callback storage so earlier handles invoke the current callback. Snapshot the callback and release internal guards before invocation; release replaced values outside the current-value lock. Keep forwarded references following the current parent. Preserve current generic bounds and add state retention, handle-loss, callback freshness/reentry and separate-slot coverage. Do not change LocalRef or its hook signature under this decision.

## Realized by

- 789d5c0ac4485bddf1f8769b71c930d69b1c60ac Own Unix input workers and retain shared reference hooks
