# Reflow retained terminal soft wraps when the main screen changes width

Level: Judged
Decided by: Codex
Rests on: API-011 API-019
Would be wrong if: Resizing joins explicit lines, loses styled graphemes or cursor state, reflows alternate-screen applications, grows memory beyond the retained cells, or still disagrees with native ConPTY after complete output.
History: The retained Terminal decision preserves its public screen API alongside libghostty. Native App run 34510730395 fails when widening a 43-column screen to 47 columns: ConPTY joins a wrapped banner line while VirtualScreen only extends rows. A separate captured-output replay also exposed output arriving across a height change; retain that failure case and do not call reflow a complete ordering repair.

## Decision

Record soft-wrap boundaries as text is written and reflow those logical lines when the main screen changes width. Preserve explicit line breaks, cell styles, hyperlinks and whole graphemes; map the active and saved cursors through the reflow, retain bounded scrollback when the cursor would leave a smaller viewport, and keep the alternate screen as a clipped fixed grid. Use compact intermediate rows so large width changes do not allocate one full-width row per old blank line. Preserve the public Terminal and VirtualScreen interfaces. Verify both growth and shrinkage, styled wide/combining text, pending-wrap cursor state, hard breaks, scrollback and alternate-screen restoration with deterministic tests, then rerun the strict native Windows App workflow. Keep the independently observed pending-output resize issue explicit until native evidence or a separate repair resolves it.

## Realized by

(none yet: recorded, not built)
