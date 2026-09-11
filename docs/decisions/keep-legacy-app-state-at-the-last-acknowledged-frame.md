# Keep legacy App state at the last acknowledged frame

Level: Judged
Decided by: Codex
Rests on: API-016,API-002,API-004,RND-004
Would be wrong if: A failed presentation advances the diff baseline, unchanged frames produce insertions, or constructing the candidate duplicates a component lifecycle.
History: The five API reversals named by Cairn concern clipboard timing, ConPTY runtime behavior, terminal reflow and reader state delivery. They require checks at the actual consumer boundary, not assumed success. This private tree-state repair and additive native selector remain Judged; they change none of those approved platform choices or deadlines.

## Decision

Use one retained RenderTree as the last successfully presented Element frame. Build the next candidate separately, obtain real reconciler patches including the initial insertion, pass that candidate to the backend, and retain it only after presentation succeeds. Preserve stable scoped node keys and complete Element children for legacy consumers without constructing component instances. Add the SuprTUI selector to the C App builder and the generated native TypeScript schema; preserve all existing exported signatures. The legacy rendering adapters remain a separate part of API-016 and acceptance stays failing until every declared route passes.

## Realized by

(none yet: recorded, not built)
