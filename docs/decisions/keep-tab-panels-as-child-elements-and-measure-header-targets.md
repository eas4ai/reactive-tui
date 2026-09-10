# Keep tab panels as child elements and measure header targets

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Panel controls lose events or state, inactive panels paint or accept input, or header clicks disagree with the presented frame.
History: Earlier API decisions require real App ownership and measured geometry. This extends those choices without changing public props or callbacks.

## Decision

Render tab headers, close controls and panels as real layout children. Measure each header and close target after presentation and retain geometry inside its App-owned Tabs instance. Keep automatic and manual keyboard activation, skip disabled tabs and emit callbacks only for actual changes or explicit close requests. Preserve lazy loading: mount only the active panel when enabled; retain hidden panel instances when disabled. Both builder routes construct the same typed component. No fixed screen width or label-byte hit test remains.

## Realized by

Implementation: `src/widgets/layout/tabs.rs`, `src/builder/widgets/layout.rs`.

Behavior checks: `tests/api_widget_behavior/tabs.rs`, `tests/api_widget_behavior/orca_tabs.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
