# Count complete static image decode storage

Level: Judged
Decided by: Codex
Rests on: XIS-002,render-external-image-tools-into-owned-clipped-cells-off-the-app-thread
Would be wrong if: The image decoder retains storage not counted by its allocation limit, or converting a decoded color format can exceed the reserved RGBA output without rejection.

## Decision

Keep 256 MiB as the total static image decode budget. Count the encoded bytes already held by the framework, the decoder's simultaneous allocations, and a full RGBA output copy when the decoder does not already return RGBA8. Set the image decoder allocation limit to the remaining budget before decoding and consume DynamicImage with into_rgba8 so an RGBA8 buffer moves without a second copy. Reject the image before decoding when the measured required decoder output cannot fit. Keep the separate 64 MiB encoded-input limit. Pass -- before every chafa and viu image path.

## Realized by

- 5be1de9622853ee3d8f5cbc11468a4b92c65fb4c fix: bound complete image decode memory
