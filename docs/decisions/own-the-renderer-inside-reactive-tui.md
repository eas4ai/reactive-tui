# Own the renderer inside Reactive-TUI

Level: Judged
Decided by: Shawn and Codex
Rests on: EMB-002 RND-001 RND-002 RND-004
Would be wrong if: The local renderer cannot retain its upstream behavior or full terminal styles without breaking application output.

## Decision

Bring the renderer from SuprTUI revision 7793deb80c5bceecc5d8ed9fc6bc6d2530531774 into src/backend/engine as an internal path crate. Preserve source provenance and applicable MIT notices. Keep SuprTuiBackend as the compatibility adapter and retain the owned renderer worker. Extend terminal cell styles locally for EMB-002. This implements the developer response to emb-002; the existing embedded-terminal commitment remains the scope.

## Realized by

(none yet: recorded, not built)
