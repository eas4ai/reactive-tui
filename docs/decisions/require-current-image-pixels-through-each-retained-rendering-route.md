# Require current image pixels through each retained rendering route

Level: Judged
Decided by: Codex
Rests on: API-014
Would be wrong if: The selected host cases omit a claimed protocol or a passing capture can substitute fallback for real graphics.
History: The approved iTerm2 3.7 color and transparency exception remains limited to that host; all Linux protocol color checks remain exact. The recorded local URL decision remains unchanged.

## Decision

Reuse the existing decoded-pixel, App, platform and Surface tests, then execute the existing isolated X11 image capture driver for App, standalone and Surface routes on their claimed hosts, including external renderers and animated GIFs. Reject a forced ASCII run with the same exact-color assertions. Verify committed macOS and Windows native records and retain fresh Linux captures with the acceptance receipt. Do not count historical defect-confirming tests as acceptance or widen the approved iTerm limitation.

## Realized by

- 245767b94c59310e0ea302494a0f3c09fd21633f Declare image acceptance with decoded pixels and real host captures
