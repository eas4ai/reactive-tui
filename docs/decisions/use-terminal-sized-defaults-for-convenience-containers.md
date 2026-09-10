# Use terminal-sized defaults for convenience containers

Level: Judged
Decided by: Shawn and Codex
Rests on: API-011
Would be wrong if: Default helper content is still invisible at ordinary terminal sizes, or caller-supplied classes stop overriding the defaults.
History: Prior API reversals require direct runtime proof. This remains Judged because it repairs helper defaults without changing CSS utility interpretation or public signatures.

## Decision

Use modest terminal padding for the content and card convenience helpers. Their current p-6 padding consumes 24 rows on each side and hides all content in a 40x16 or 80x24 terminal. Keep classes overridable and preserve the global spacing scale. Verify default content through App at both sizes and cover nested composition and input helpers before catalog closure.

## Realized by

Implementation: `src/builder/layout.rs`, `src/builder/core.rs`, `src/layout/css/variants.rs`.

Behavior checks: `tests/api_widget_behavior/core_builders.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
