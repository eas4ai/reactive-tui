# Compose screen transitions through the complete element painter

Level: Judged
Decided by: agent
Rests on: API-013
Would be wrong if: Intermediate captured frames do not change with progress, Unicode is damaged, or active screen input disagrees with presented geometry.
History: Earlier API reversals concerned clipboard deadlines, ConPTY ownership, terminal reflow and accessibility coordinates. This change reuses acknowledged painter geometry instead of inventing hit coordinates, and verifies actual captured frames.

## Decision

Prepare both retained screen component trees and compose full-size layers through the existing element painter. Fade blends opacity; directional transitions translate layers; scale and terminal flip/cube approximations use cell transforms. Keep the source screen active until completion and remap its acknowledged geometry from the composed tree so callbacks retain stable identities. Preserve legacy backend patch fallback. Verify intermediate colors, directional placement, endpoints and input before completion.

## Realized by

(none yet: recorded, not built)
