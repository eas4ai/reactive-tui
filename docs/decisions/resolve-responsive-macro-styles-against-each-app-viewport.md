# Resolve responsive macro styles against each App viewport

Level: Judged
Decided by: Agent
Rests on: API-009, API-018
Would be wrong if: Breakpoint values are discarded, a resize keeps stale styles, utility classes lose precedence, numeric validation is bypassed, or style closures cross the renderer thread boundary.
History: The public responsive_css macro has no compiling implementation: it calls an absent StyleBuilder::class method and discards every breakpoint value. API-018 standalone documentation compilation reproduced the failure. API-009 already defines App breakpoints and owned style snapshots.

## Decision

Keep responsive_css returning StyleBuilder. Apply breakpoint blocks in ascending minimum terminal-column order using the existing sm=40, md=80, lg=120 and xl=160 thresholds; repeated thresholds retain declaration order. Materialize cumulative owned styles during construction, evaluating each block once, and serialize only style data. Resolve the selected profile against the current App or ScreenManager viewport before utility classes, motion, accessibility and painting. Keep explicit at_width resolution for standalone StyleBuilder users. Reject unknown macro breakpoint names at compile time. Verify boundary widths, reverse resize, class precedence, independent Apps, invalid values and facade-only macro compilation.

## Realized by

(none yet: recorded, not built)
