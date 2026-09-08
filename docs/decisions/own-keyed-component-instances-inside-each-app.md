# Own keyed component instances inside each App

Level: Judged
Decided by: Codex
Rests on: API-002
Would be wrong if: Keyed redraws recreate state, component callbacks run under registry locks, App cleanup disrupts another App, or expanded output loses declared children and styling.

## Decision

Keep component factories in the existing registry and give each App a private instance tree indexed by parent-scoped keys and positional slots. Mount one instance, update its props and recursively expand its output before focus, layout and painting; remove descendants before parents. Keep public legacy render-tree conversion behavior separate and use a pure conversion for already-expanded App output. Unregistered names retain their existing container fallback and children. Component classes and focus apply to its rendered root, and explicit Element children are appended to that root. Reject duplicate sibling keys and excessive expansion depth with an error. Preserve public signatures and avoid cloning live components. This ownership will support the later hook and event requirements without putting App state in the global registry.

## Realized by

- 919afade932ec4cb034c5cb49766ce3aa38d20da Render App components through persistent keyed instances
