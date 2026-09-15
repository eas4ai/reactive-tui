# Normalize invalid FPS bounds and empty zero-column grids

Level: Judged
Decided by: Codex
Rests on: RTR-004
Would be wrong if: Callers rely on reversed FPS bounds preserving the smaller maximum, or zero-column grids should reject construction instead of returning no placements.

## Decision

Keep the existing infallible public APIs. AdaptiveFpsManager::with_config keeps both bounds between one and one billion FPS, the largest rate with a nonzero nanosecond frame duration, and raises max_fps to the normalized minimum when the bounds are reversed. All later target changes clamp through those normalized bounds, and frame-duration calculation defensively divides by at least one. DeclarativeGrid::auto_grid returns the requested zero-column grid with no placed children before it calculates row or column positions. Document both normalization rules on the public methods.

## Realized by

- bda32a1d7a65c266afe6fc1432a04e4d8ccf3fd4 fix: normalize arithmetic boundaries

Implementation: `AdaptiveFpsManager::with_config` normalizes all stored bounds before its first clamp. `DeclarativeGrid::auto_grid` returns a responsive zero-column grid with no children before calculating positions. Both public methods document these rules.

Behavior check: RTR-004 tests a cross-product of zero, normal, reversed, and extreme FPS bounds plus zero-column grids with several row and item counts.
