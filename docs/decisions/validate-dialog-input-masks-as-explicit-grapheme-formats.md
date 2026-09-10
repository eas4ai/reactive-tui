# Validate dialog input masks as explicit grapheme formats

Level: Judged
Decided by: Codex
Rests on: API-011,API-012,API-018
Would be wrong if: A configured mask has no observable effect, invalid or incomplete input can submit, escaped literals or Unicode split incorrectly, or App and native validation disagree.
History: The retained dialog decision requires the same validation contract for App and native controls. InputFieldConfig advertises a mask/format string but defines no syntax and never applies it.

## Decision

Define the mask string as an exact input format: # requires one ASCII digit, A requires one Unicode alphabetic grapheme (including combining marks), * accepts one non-control grapheme, and backslash escapes the next grapheme. Other graphemes are literals entered by the user. Keep editing unrestricted so users can correct existing or pasted values; validate the complete format on configured change/blur validation and before submission. Empty optional fields remain allowed, malformed masks produce an actionable validation error, and the active format is visible beside the input. Preserve the authored value in callbacks and results without silent reformatting. Test invalid and corrected App submissions at both viewport sizes plus native validation, Unicode, escaped literals and malformed configuration.

## Realized by

Implementation: `src/widgets/dialog/input`.

Behavior checks: `tests/api_widget_behavior/input.rs`, `docs/dialog-input.md`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
