# Connect App producers with coalesced wake notifications

Level: Judged
Decided by: Shawn and Codex
Rests on: WAK-001 WAK-002 WAK-003 WAK-004 WAK-005
Would be wrong if: Notifications are lost at wait boundaries, idle roots stop receiving required work, or the existing input and cleanup contracts regress.

## Decision

Use a per-App wake handle with bounded pending flags, weak render-observed signal subscriptions, and an App-owned shared scheduler. Add defaulted root and backend compatibility hooks. Use the existing Crossterm EventStream with a standard task waker for interruptible host waiting, retaining its decoder instead of adopting Termwiz input types. Borrow the WezTerm wake-and-invalidate pattern. Timer callbacks run outside scheduler locks. Embedded publications wake their attached App. Preserve periodic updates for existing roots unless they opt into wake-driven behavior.

## Realized by

(none yet: recorded, not built)
