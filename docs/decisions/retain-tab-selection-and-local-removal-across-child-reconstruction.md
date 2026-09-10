# Retain tab selection and local removal across child reconstruction

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Rebuilding a child resets tab choice, closed tabs return, parent-authored selection is ignored, or callbacks and panel identities become stale.
History: Earlier API reversals require observed native behavior rather than metadata claims. The real Orca run exposed selection reverting when fresh child Elements made supplied props unequal; existing static-root App tests missed it. This is a local ownership correction at Judged level, with explicit negative and corrected App and reader checks.

## Decision

Keep effective tab props in the retained Tabs owner. Reconcile newly authored props with the last authored selection and tab identities: replacing child Elements updates content without resetting local selection or reopening locally closed tabs. Explicitly changed authored selection takes effect. Use content keys where supplied and positional identity otherwise, matching the existing panel identity contract. Route semantic focus and activation to this same owner. Verify reconstructed child input, close, prop updates and real Orca tab delivery.

## Realized by

Implementation: `src/widgets/layout/tabs.rs`, `src/builder/widgets/layout.rs`.

Behavior checks: `tests/api_widget_behavior/tabs.rs`, `tests/api_widget_behavior/orca_tabs.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
