# Use viewport-sized Braille for the default catalog wireframe

Level: Judged
Decided by: Sol
Rests on: CAT-001 CAT-002
Would be wrong if: Owned Kitty captures show disconnected or clipped edges, or resizing changes the animation deadline or quit lifecycle.

## Decision

Render the existing twelve cube edges into two-by-four Braille subcells per terminal cell, using the available Motion viewport. Clamp the canvas to 240 by 100 cells before allocation. Keep elapsed-time rotation, the 80 ms cadence, the fixed-size compatibility helper, and normal App shutdown. This improves screenshot quality without a new dependency or changing the optional shaded wgpu renderer. Native cell tests, distinct pixel hashes, and owned Kitty captures must verify the result.

## Realized by

- 0ad385ee0eef6e9e8f25a042172aafbbf78fce28 fix: scale catalog wireframe with Braille subpixels
