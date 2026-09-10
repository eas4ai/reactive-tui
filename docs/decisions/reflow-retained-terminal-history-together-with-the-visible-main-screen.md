# Reflow retained terminal history together with the visible main screen

Superseded by: keep-reflowed-terminal-history-above-the-visible-origin

Level: Judged
Decided by: Codex
Rests on: API-011 API-019
Would be wrong if: Resizing hides retained text, breaks a soft-wrapped line at the history boundary, moves the active cursor off screen, or exceeds the configured history limit.
History: Earlier API reversals concerned native process and accessibility boundaries. This extends the recorded owned screen reflow after two local failures demonstrated missing history handling; it remains Judged because it changes no public API or platform claim. Native verification remains required.

## Decision

Extend the existing terminal reflow to the retained history and main grid as one sequence. Track the first visible cell alongside the active and saved cursors, then choose the visible origin from those remapped positions. Preserve complete historical lines above that origin; a logical line crossing it may bring its prefix into the first visible row when it becomes wider. Apply the configured history row limit after reflow. Height growth alone keeps the visible origin. Alternate-screen cells remain a fixed grid. Regression probes first demonstrate hidden columns and a broken soft wrap at the history boundary.

## Realized by

(none yet: recorded, not built)
