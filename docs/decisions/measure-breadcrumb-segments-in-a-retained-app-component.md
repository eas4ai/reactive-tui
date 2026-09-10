# Measure breadcrumb segments in a retained App component

Level: Judged
Decided by: Codex
Rests on: API-011, API-018
Would be wrong if: Overflow depends on guessed terminal bounds, a click navigates to a different painted segment, inert segments activate, public construction breaks, or hidden segments retain accessible targets.
History: The baseline App tests found no named builder rendering or keyboard callbacks; the old mouse path assumes a first-row character offset and overflow estimates byte lengths.

## Decision

Preserve the public Breadcrumb unit component, props, state and builders. Render a retained child that owns acknowledged viewport and segment geometry, focus, hover and scroll position, following the existing accordion ownership pattern. Fit overflow against measured segment widths and the actual viewport, including separators and configured maximum width. Route mouse and assistive actions through the painted segment targets and emit one deferred JSON navigation notification under the configured callback name, or breadcrumb_navigation when none is supplied. Retain accessible labels and current/nonclickable semantics; paint configured tooltips and implement wrapping and scrolling as actual layout behavior.

## Realized by

Implementation: `src/widgets/layout/breadcrumb`.

Behavior checks: `tests/api_widget_behavior/breadcrumb.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
