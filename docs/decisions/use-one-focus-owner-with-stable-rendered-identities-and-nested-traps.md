# Use one focus owner with stable rendered identities and nested traps

Level: Judged
Decided by: Codex
Rests on: API-006
Would be wrong if: A redraw changes focus without an input or lifecycle cause, reverse traversal differs from forward order, or closing a nested trap restores a removed or outside node.

## Decision

Use the event router focus manager for both public App focus methods and routed input. Replace the private duplicate focus state with a declarative frame plan built from the stable event-tree IDs. Keep rendered order as the tie breaker for tab indices, skip negative indices during Tab navigation, and ignore release events. Retain trap creation order and restore targets across redraws; update membership without reopening a live trap. The most recently opened live trap confines programmatic, mouse and keyboard focus; removing it restores a valid target in the remaining trap or the document. Register declarative focus and blur callbacks on the matching event nodes and emit transitions only when identity changes. Preserve public Rust signatures and C ABI.

## Realized by

- 07bfeec54339eb77121cf9312af656906a38084c Unify stable App focus and nested trap lifecycle
