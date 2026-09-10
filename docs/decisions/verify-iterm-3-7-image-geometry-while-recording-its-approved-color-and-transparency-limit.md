# Verify iTerm 3.7 image geometry while recording its approved color and transparency limit

Level: Judged
Decided by: agent
Rests on: API-011 API-014 API-020
Would be wrong if: The geometry check accepts absent, stale or unmoved image regions, or the evidence claims exact colors or transparency on iTerm 3.7.
History: Prior API reversals show that synthetic checks alone miss host behavior. This remains Judged because the developer already approved the exact host limitation; classification must reject violating fixtures and pass native captures before acceptance. No other platform contract changes.

## Decision

The developer approved escalation api-011-api-014-api-020. Preserve strict exact-sRGB measurements and an explicit strict diagnostic mode. For this pinned host only, independently classify red, blue, green and yellow regions by dominant channels to check presence, replacement, movement and removal; do not count that classification as color accuracy. Store both measurements and the support limitation in native evidence. Keep WezTerm inline color acceptance and all public APIs unchanged.

## Realized by

`scripts/check-iterm-host.py` records exact sRGB and dominant-channel geometry
separately. The strict color diagnostic remains available through
`--require-exact-srgb`. `scripts/check-widget-platforms.py` requires both captured
measurement files. The specification and widget inventory state the approved
host/version limitation. Local violating/corrected cases are recorded in
`.cairn/reviews/api-011-iterm-geometry-falsification.log`; fresh native acceptance
is pending run 34529569174.
