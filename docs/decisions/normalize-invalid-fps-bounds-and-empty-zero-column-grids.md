# Normalize invalid FPS bounds and empty zero-column grids

Level: Judged
Decided by: Codex
Rests on: RTR-004
Would be wrong if: Callers rely on reversed FPS bounds preserving the smaller maximum, or zero-column grids should reject construction instead of returning no placements.

## Decision

Keep the existing infallible public APIs. AdaptiveFpsManager::with_config raises min_fps to at least one and raises max_fps to the normalized minimum when the bounds are reversed. All later target changes clamp through those normalized bounds, and frame-duration calculation defensively divides by at least one. DeclarativeGrid::auto_grid returns the requested zero-column grid with no placed children before it calculates row or column positions. Document both normalization rules on the public methods.

## Realized by

(none yet: recorded, not built)
