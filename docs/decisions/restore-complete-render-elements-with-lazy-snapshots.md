# Restore complete render elements with lazy snapshots

Level: Judged
Decided by: Codex
Rests on: API-016,API-019,API-020
Would be wrong if: Existing as_element callers cannot inspect descendants, snapshots become stale after child replacement, or snapshot construction and destruction recurse with tree depth.
History: API-019 removed descendant data from as_element; the native legacy consumer exposed this compatibility regression and Shawn approved restoring the prior complete-subtree behavior.

## Decision

Keep node-local storage for conversion. Materialize and retain the complete Element snapshot only when as_element is called, reconstructing descendants iteratively. Internal whole-tree reconstruction reads node-local data to avoid populating every subtree cache. Clear the cache when child ownership changes and destroy retained snapshots iteratively. Preserve the unchanged native consumer and add direct descendant and cache-lifecycle assertions.

## Realized by

- 13ba4cb10379e1eb8e12684b00ac5bb03a4935ee Restore complete Element subtrees through render node accessors
