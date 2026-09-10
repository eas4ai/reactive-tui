# Render tables through retained measured controls

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Table builders still rely on assumed bounds, sorted or filtered input targets the wrong source row, configured controls remain descriptive, or existing public construction breaks.
History: The widget audit found descriptive builders and fixed bounds. Table currently assumes width 80 and origin zero, ignores widths during painting, and updates different selection fields for mouse and keyboard. DataTable has no event implementation.

## Decision

Preserve the public Table and DataTable unit components, props, state and builder routes. Render retained child components that own measured viewport geometry and local interaction state. Use source row IDs to retain selection across reorder and source indices for callbacks, with sorting and filtering applied before pagination or visible slicing. Lay out real column cells with bounded widths and use their presented geometry for header, row, cell and resize input. Build DataTable controls from the existing functional input and button components, retaining filters, search, pagination and column visibility locally until corresponding props change. Keep original row callback indices when passing filtered or paged rows to Table. Verify real App input, clipping, callbacks and resize at multiple terminal sizes.

## Realized by

Implementation: `src/widgets/display/table`, `src/widgets/display/data_table`.

Behavior checks: `tests/api_widget_behavior/table.rs`, `tests/api_widget_behavior/data_table.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
