# Avoid repeated synchronized-update warnings in native Kitty scanout

Surfaced from: CAT-002
Outside because: The current catalog finding requires readable viewport-sized wireframe edges and clean animation/quit, which pass independently. Changing shared scanout mode transitions is separate framework work, not this example configuration repair.
Captured: 2026-09-16T21:08:36.846Z

Owned Kitty animation captures repeatedly log pending mode change to already current mode (0). The frame and quit checks still pass. The catalog host now captures stderr in a private regular file because an undrained pipe filled and stalled remote control. Investigate duplicate synchronized-update mode transitions separately, with byte-stream tests and Kitty diagnostics; do not treat the capture-harness fix as a scanout repair.
