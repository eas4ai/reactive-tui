# Input dialog formats and updates

`InputFieldConfig::mask` describes the complete input format. Validation checks it
on configured change/blur events and before submission. The dialog displays the
authored format beside the field.

| Token | Meaning |
| --- | --- |
| `#` | One ASCII digit (`0` through `9`) |
| `A` | One Unicode alphabetic grapheme, including combining marks |
| `*` | One grapheme without control characters, including emoji |
| Backslash followed by a grapheme | That literal grapheme |
| Any other grapheme | That literal grapheme |

For example, `AA-##` accepts `AB-42` or `界é-42`, and rejects `12-AB`, `AB42`
and incomplete input. The user enters separators. Editing and pasting preserve
the authored text so it can be corrected; callbacks and results receive that
same text. A trailing backslash or a control character in the mask is a
configuration error. Empty values are permitted unless the field is required.

Retained App inputs keep edits across ordinary redraws. Changing the initial
value replaces the field's value. New callbacks and read-only attributes take
effect on the retained field. Change validation is debounced; replacing the
delay reschedules pending validation. Disabling change validation, closing or
removing the dialog cancels its pending timer.

App tests in `tests/api_widget_behavior/dialogs.rs` cover mask rejection and
correction, callback/read-only/seed updates, and pointer editing and submission
after resize. Tests in `dialog_input_lifecycle.rs` cover debounce coalescing and
cancellation. These workflows run at 32×12 and 60×20 cells. Native event tests
also check mask literals, Unicode and invalid configuration.

These checks do not establish complete dialog-engine lifecycle or screen-reader
acceptance. Remaining work is listed in [the widget inventory](widget-acceptance.md).
Remote validation has a separate [HTTP contract and evidence limits](dialog-http.md).
