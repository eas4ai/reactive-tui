# Expose tree checkboxes as controls inside their tree items

Level: Judged
Decided by: Codex
Rests on: API-011 API-018
Would be wrong if: The tree loses hierarchy or labels, checking duplicates actions, or the real reader still does not announce checkbox changes.
History: The corrected tree focus probe delivers expansion and focus. Its checkbox test receives checked state on the TreeItem but Orca 50.1.2 generates no speech: the tree-item generator handles expansion only. The checkbox is already a distinct painted region and mouse target.

## Decision

Keep each row as a TreeItem with its label, selection and expansion. Expose its painted checkbox as a labelled CheckBox child with checked state and an explicit action routed to the retained tree owner. Preserve the existing keyboard Space and measured checkbox click behavior. A focus request must not check or activate a row. Verify actual Orca checkbox speech while navigating the tree and through its checkbox action, alongside disabled rows and collapse cleanup.

## Realized by

Implementation: `src/widgets/display/tree`.

Behavior checks: `tests/api_widget_behavior/orca_data.py`, `tests/api_widget_behavior/tree.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
