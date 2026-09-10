# Publish App accessibility through an owned AccessKit Linux adapter

Level: Judged
Decided by: Codex
Rests on: API-011, API-018
Would be wrong if: Orca cannot observe the labels and state changes, inaccessible or removed nodes remain exposed, callbacks reenter borrowed widgets, or adapter resources cross App lifetimes.
History: The developer approved the named screen-reader target in api-011-api-018. Existing decisions require App ownership, exactly-once event delivery and real platform evidence.

## Decision

Use AccessKit and its Linux AT-SPI adapter for the approved GNOME Terminal and Orca integration. Keep semantic metadata attached to Elements and publish only acknowledged visible nodes with stable App event identities. Represent composite-widget active descendants explicitly, retain labels distinct from painted text, and route assistive actions back through App with bounded owned delivery. The App owns adapter and snapshot lifetimes. Use isolated D-Bus and desktop sessions for real Orca checks so test configuration does not alter the developer desktop. Prove the delivery path with a disposable probe before integrating it into the runtime. Do not claim other terminal and reader pairs.

## Realized by

Implementation: `src/accessibility/connection.rs`, `src/accessibility/platform/unix/adapter.rs`, `src/app.rs`.

Behavior checks: `tests/api_widget_behavior/accessibility_probe.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
