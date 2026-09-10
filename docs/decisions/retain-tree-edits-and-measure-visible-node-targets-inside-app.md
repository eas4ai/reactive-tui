# Retain tree edits and measure visible node targets inside App

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Tree redraws discard expansion or loaded children, node IDs become ambiguous, a drop creates a cycle, or input reaches a node outside the painted viewport.
History: The existing widget ownership decision preserves public unit components through retained child instances. Earlier reversals concerned measured behavior and resource ownership; this change uses the same App owner and measured layout without changing public Tree signatures.

## Decision

Keep Tree, TreeProps and TreeState public construction compatible. Render an App-owned child component that retains expansion, checking, selection, cursor and a local root for lazy-loaded children and drag/drop moves. Caller changes replace the corresponding authored state; ordinary redraws retain user changes by node ID. Measure painted expander, checkbox and label targets after layout and clip scrolling to the actual terminal viewport. Search reveals matching paths and optionally hides nonmatches. A drop moves a node under an expandable target, rejects the root and descendant cycles, and reports the source node action as drop:<target-id>; it changes only the in-memory tree, never the filesystem. The synchronous lazy-load callback supplies children once per lazy node until the caller replaces the root. Invalid duplicate IDs or missing lazy-load callbacks produce visible errors. Preserve the named and typed routes, replace the descriptive specialized builder, and test input, resize, lazy loading, drop rejection and state replacement through App.

## Realized by

Implementation: `src/widgets/display/tree`.

Behavior checks: `tests/api_widget_behavior/tree.rs`, `tests/api_widget_behavior/orca_data.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
