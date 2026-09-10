# Measure progress bars in a retained child and own their animation deadlines

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Progress builders still paint descriptions, vertical or segmented bars ignore their measured dimensions, animation depends on unrelated input, completion callbacks duplicate, or removal leaves a timer active.
History: The widget ownership decision preserves public unit components through retained App children. Chart and Accordion already use component-owned Scheduler timeouts; no shared global animation owner is needed.

## Decision

Keep ProgressBar, its props, builders and state helpers. Route both builder families to the actual component. Let the public component retain completion edge detection and let an internal child own measured bar cells and animation deadlines. Interpolate determinate changes from the displayed fraction over 200 milliseconds; schedule indeterminate movement and pulse updates while mounted, and cancel owned deadlines on stop or removal. Render horizontal thickness and vertical height, bounded segments, stripes, foreground/background colors and text styles inside the measured parent. Honor formatter/callback replacement, reject invalid numeric ranges and zero dimensions/segments visibly, and never report completion for invalid or indeterminate input. Verify App frames, completion transitions, changed props, resize, animation and cleanup, with a safe violating case before acceptance.

## Realized by

Implementation: `src/widgets/display/progress_bar`.

Behavior checks: `tests/api_widget_behavior/progress.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
