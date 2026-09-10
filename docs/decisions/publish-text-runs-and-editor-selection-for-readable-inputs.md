# Publish text runs and editor selection for readable inputs

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Orca cannot read editable values and caret changes, Unicode positions diverge from the editor, or passwords expose their original text.
History: Earlier API reversals require real reader evidence. The tabs reader test now reaches the TextInput but fails because it exposes no AT-SPI Text interface; labels alone did not establish readable input.

## Decision

Attach private grapheme selection metadata and keyed screen-reader text runs to TextInput. App resolves these runs to its existing stable node identities when exporting AccessKit selection. Preserve actual text and hard line breaks for ordinary fields and expose only masked characters for passwords. Keep painting separate from readable content and verify keyboard editing, selection, multiline, empty values and password privacy through consumer and real Orca checks.

## Realized by

Implementation: `src/widgets/input/text_input.rs`, `src/accessibility/text.rs`.

Behavior checks: `tests/api_widget_behavior/orca_tabs.py`, `tests/api_widget_behavior/input.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
