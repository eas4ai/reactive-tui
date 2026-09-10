# Repair iTerm 3.7 inline image color and transparency behavior in the host

Surfaced from: API-014
Captured: 2026-09-10T19:31:55.017Z

Native diagnostics show iTerm2 3.7.0 shifts pure red/blue pixels and paints white behind transparent PNG pixels. Identical PNGs render exact colors and preserve black background in an independent AppKit window. Explicit sRGB/ICC profiles and UseMetal=NO do not fix the host output. Evidence and standalone repros are in .cairn/reviews/api-011-iterm-native-reference, api-011-iterm-color-diagnostic, api-011-iterm-renderer-diagnostic, and api-011-native-image-reference.{py,swift}. A host patch or upstream resolution needs separately agreed scope; do not weaken library acceptance or silently claim support.
