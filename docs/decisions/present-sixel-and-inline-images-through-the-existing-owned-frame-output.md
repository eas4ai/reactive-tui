# Present Sixel and inline images through the existing owned frame output

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011 API-014 API-016
Would be wrong if: Clearing and repainting the owned application screen cannot remove Sixel or inline image placements on a claimed host, or compositing cannot preserve clipping and paint order.
History: This extends the recorded owned-image frame decision to its remaining graphics protocols. Kitty and Ghostty App pixels now pass real-host checks, and corrected standalone Sixel and inline encoders pass Xterm and WezTerm pixel checks. Public Image fields remain unchanged.

## Decision

Keep the SuprTUI worker and common image planes. Select Kitty, Sixel or inline output from the requested mode and known capabilities, with ASCII for unavailable modes. Add explicit image options for interactive hosts whose capabilities the caller knows. Kitty keeps owned-ID deletion. Since Sixel and inline protocols lack equivalent placement deletion, clear the owned application screen and repaint all cells and image planes when these placements change, on resize, and on cleanup, inside the same checked frame operation. Preserve masks and paint order; composite partial-alpha Sixel pixels over preceding image planes and the actual cell background before encoding its binary transparency. Retain cached unchanged images when unrelated cell updates do not touch them. Keep bounded frame allocations, checked writes, failure retries, and real-host pixel tests for each route.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

`src/backend/suprtui/graphics.rs` selects shared Kitty, Sixel and inline
encoders and composites preceding pixels into legacy placements. Sixel uses
saved/restored DECSDM and bounded origin padding to avoid Xterm scrolling at
the bottom edge. The worker clears and repaints owned legacy placements on
change, resize and shutdown; unchanged frames retain their placements.

The review records decoded-output regressions and real Xterm and WezTerm pixel
captures for update, movement and removal, plus an Xterm full-height negative
and corrected pair. The full commitment and remaining image paths are still
under implementation; this record does not claim catalog completion.
