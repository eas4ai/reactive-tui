# Keep reflowed terminal history above the visible origin

Level: Judged
Decided by: Codex
Supersedes: reflow-retained-terminal-history-together-with-the-visible-main-screen
Cause: an unforeseen condition occurred
Rests on: API-011 API-019
Would be wrong if: Retained text becomes inaccessible, resize brings a historical prefix into view and shifts native cursor-addressed updates, or the history limit is exceeded.
History: The earlier history reflow passed local text-preservation tests but native Windows 34517617898 exposed a visible-origin mismatch. Existing API reversals reinforce keeping native cursor behavior as evidence. This remains Judged: preserve text in history and change only viewport selection, with a captured Windows replay and platform rerun.

## Decision

Reflow retained history and the main grid together, preserving all bounded historical text. If the old first visible cell maps inside a newly packed row, keep that entire row in history and start the visible grid at the following row. A row beginning exactly at the old visible origin remains visible. Keep the active cursor visible and preserve saved cursor mapping. This avoids pulling a historical prefix back into the viewport, which shifted the screen relative to ConPTY cursor coordinates. Height growth alone keeps the origin. Alternate-screen cells remain fixed.

## Realized by

(none yet: recorded, not built)
