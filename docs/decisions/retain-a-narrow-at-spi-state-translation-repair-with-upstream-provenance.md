# Retain a narrow AT-SPI state translation repair with upstream provenance

Superseded by: preserve-at-spi-state-and-focus-actions-without-invented-pixel-bounds

Level: Judged
Decided by: Codex
Rests on: API-011, API-018
Would be wrong if: The patch changes unrelated adapter behavior, source provenance is lost, published consumers miss the repair, or state assertions pass without real Orca delivery.
History: The disposable probe delivered labels and focus to Orca but no expansion event. This follows the approved screen-reader target and challenges the adapter at the actual observation boundary.

## Decision

The installed AccessKit AT-SPI translation layer and current 0.20.0 release omit expanded and expandable state. Their disabled handling also leaves ordinary disabled buttons enabled and sensitive. Retain the upstream crate with its licenses and version, changing only these state translations and adding regression tests. Use the repaired layer in the owned Linux adapter and verify state delivery with Orca in GNOME Terminal. Preserve the patch in packaged consumer builds, document its source and exact delta, and remove it only when an upstream release passes the same checks. Do not substitute changed spoken labels for missing semantic state.

## Realized by

Implementation: `src/accessibility/platform`, `Cargo.toml`.

Behavior checks: `src/accessibility/platform/translation/state_tests.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
