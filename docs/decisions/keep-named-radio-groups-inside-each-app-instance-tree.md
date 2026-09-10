# Keep named radio groups inside each App instance tree

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Selecting one radio leaves another selected in the same group, duplicate values select two owners, removed controls stay registered, or Apps share a group.
History: Earlier API ownership decisions require resources to belong to the live App or component, including cancellation and cleanup. This registry follows the existing owner rather than adding global mutable selection.

## Decision

Give each App component runtime its own private named-radio group registry and supply it through the existing component resource scope. A radio owns a selection token; groups keep weak references and release empty entries when controls leave. Selecting a member clears its peers before App redraws. Preserve keyed selection across redraws. The builder documents checked as the initial state, so initialize it once; when several initial members are checked, the first mounted checked member wins. A changed group name moves the existing selection into the new group only when that group has no selection. Ungrouped radios keep independent selection. Keep all public builder signatures and preserve labels, values, disabled state and classes. Verify grouped keyboard and mouse selection, duplicate values, removal, keyed reordering and separate Apps.

## Realized by

Implementation: `src/widgets/input/named_radio.rs`, `src/builder/specialized.rs`.

Behavior checks: `tests/api_widget_behavior/radio.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
