# Keep Stack children in the shared flex layout tree

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Nested controls lose input or identity, or spacing, padding, wrapping and alignment differ from the actual viewport.
History: Earlier API decisions require real ownership and measured geometry. This uses the existing layout engine and preserves the public props and builder routes.

## Decision

Render Stack as a flex container with its original child Elements. Map direction, wrapping, reversal, alignment, justification and cell spacing to the existing StyleBuilder. Both builder routes construct the same component; caller classes override defaults through the existing style cascade. Remove the private text-flattening layout implementation and replace its private-helper tests with App observations of content positions and nested input. Keep public StackState fields compatible.

## Realized by

Implementation: `src/widgets/layout/stack.rs`.

Behavior checks: `tests/api_widget_behavior/stack.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
