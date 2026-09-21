# Advance bounded GIF frames in the retained image worker

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014
Would be wrong if: The existing worker cannot honor GIF frame timing, disposal, repetition and cancellation without blocking App or leaving stale image placements.
History: The ImageFormat API advertises animated GIF. Shared decoding currently retains only its first frame; the existing owned image and external-renderer decisions already require bounded resources and lifecycle cleanup.

## Decision

Decode GIF frames with the existing image library disposal handling and read repetition metadata with its already locked gif dependency. Bound encoded input, cumulative decoded frames and frame count; reject excess rather than truncate silently. Keep the decoded animation and its clock in the retained image worker. Wait on its condition variable for the next frame or newest request, skip expired frames by elapsed time, reuse the clock on resize, and stop finite animations on the last frame. A zero delay uses a ten-millisecond minimum to avoid a busy loop. Publish frames through the existing wake and owned image output, including external modes, and cancel on source replacement or removal. Standalone one-shot serializers remain snapshots; App advances animated sources.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`live/animation.rs` uses the shared source reader, image GIF disposal handling,
and the already locked gif 0.13.3 decoder for frame count and repetition metadata.
The retained worker waits on its existing condition variable for frames or new
requests. Resize keeps decoded frames and the original clock; finite, hidden and
replaced animations stop waking App. Each published frame advances the existing
reactive signal and uses the normal owned frame output.

Decoder and worker tests cover disposal, timing, bounded allocation, cancellation,
finite stopping and replacement cleanup. The App test fails with frame wakeups
disabled and passes with them restored. A single unchanged GIF advances in actual
Kitty, Xterm Sixel, WezTerm inline, Chafa and Viu captures, then moves and is removed.
See the current commitment review for the retained logs and editing-check limits.
