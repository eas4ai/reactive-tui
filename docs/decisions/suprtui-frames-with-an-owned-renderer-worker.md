# SuprTUI frames with an owned renderer worker

Level: Judged
Decided by: Shawn and Codex
Rests on: RND-001 RND-002 RND-004
Would be wrong if: A worker cannot preserve Backend compatibility or introduces unbounded frame buffering.

## Decision

Use SuprTUI from https://github.com/eas4ai/suprtui.git at 7793deb80c5bceecc5d8ed9fc6bc6d2530531774. Keep its Rc/RefCell pools on one owned worker and acknowledge each frame synchronously through a bounded channel. Retain Backend Send + Sync. Paint graphemes using shared Taffy layout metadata; do not convert through the legacy single-char Surface. The adapter owns checked flushes and I/O errors. libghostty-rs follows in a later commitment as agreed.

## Realized by

- 296616ee5969e467a3274366c90e5d22895dee53 feat: render complete application frames with SuprTUI
