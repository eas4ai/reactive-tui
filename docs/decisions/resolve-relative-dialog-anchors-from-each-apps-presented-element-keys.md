# Resolve relative dialog anchors from each Apps presented element keys

Level: Judged
Decided by: Codex
Rests on: API-011,API-012
Would be wrong if: A dialog uses another Apps geometry, resolves an ambiguous key silently, ignores anchor removal or movement, or positions from fixed default bounds.
History: The retained-dialog decision requires measured positioning. RelativeTo currently always returns an unresolved-anchor error; ElementBuilder::id explicitly maps to Element keys.

## Decision

Keep an App-owned snapshot of presented keyed element bounds and provide it through the existing component resource scope. Resolve RelativeTo element_id against the unique presented Element key, including ElementBuilder::id. Repeated keys in separate branches are legal but cannot be used as an unambiguous anchor; report missing and ambiguous anchors explicitly. Use the requested point on the targets measured screen rectangle plus the authored offset. Replace the snapshot after acknowledged presentation, including removal and resize; schedule a redraw only when it changes. Never retain a global registry or infer a target from text. Preserve the existing Modal viewport placement rules and native bounds override. Verify all nine anchor points, offsets, changed layout and removal, ambiguous keys and isolation between Apps.

## Realized by

Implementation: `src/component/anchors.rs`, `src/widgets/dialog/frame.rs`.

Behavior checks: `tests/api_widget_behavior/dialog_position.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
