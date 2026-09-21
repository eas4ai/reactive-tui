# Use radians consistently for TransformProperty rotation

Level: Judged
Decided by: Shawn
Rests on: API-010,API-013
Would be wrong if: TransformProperty::Rotate uses different units between direct interpolation, wrapped interpolation, and App painting, or the degree-based Rotation and CSS utilities change units.
History: The recorded reversals concern clipboard process deadlines, not angle units. This contract correction has explicit developer approval in api-010-api-013; no additional platform assumption is introduced.

## Decision

The developer approved .cairn/escalations/api-010-api-013.md. TransformProperty::Rotate and rotate_animation accept radians in every path. AnimatedProperty::Rotation and CSS rotation utilities retain degrees. Convert built-in spin and degree-based convenience APIs at their boundary. Callers that relied on the inconsistent degree interpretation of TransformProperty::Rotate must convert to radians. Verify quarter-turn interpolation and equivalent captured frames.

## Realized by

- 4603bdc Paint gradients and owned animation frames through App
