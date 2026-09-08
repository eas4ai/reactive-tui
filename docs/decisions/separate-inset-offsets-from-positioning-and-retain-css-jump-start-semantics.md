# Separate inset offsets from positioning and retain CSS jump-start semantics

Level: Judged
Decided by: Shawn and Codex
Rests on: DFT-001 DFT-002 DFT-003 DFT-004
Would be wrong if: Offsets still reset layer or position, explicit position ordering changes, step boundaries disagree with the documented CSS contract, or the default suite still fails.

## Decision

Route numeric side offsets through the existing inset-only parser, like other inset values. Callers select absolute or fixed positioning explicitly; explicit position classes retain their current layer defaults and order. Retain the existing jump-start implementation: CSS Easing Level 1 section 2.3 specifies the first jump at zero and at exact interval boundaries. Correct the contradictory test and extend boundary coverage. Add visible imports to the four accordion doctests without suppressing execution.

## Realized by

d44c45441993ca27aa946c43026cac78e32085b2
