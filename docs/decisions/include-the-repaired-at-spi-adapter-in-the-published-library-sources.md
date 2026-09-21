# Include the repaired AT-SPI adapter in the published library sources

Level: Judged
Decided by: Codex
Rests on: API-011, API-018
Would be wrong if: A consumer can resolve the unpatched adapter, unrelated upstream behavior changes during namespacing, or the included sources cannot be traced to their releases.
History: The narrow state repair proved actual collapsed and expanded announcements in Orca. A Cargo patch table would not reliably carry that repair into downstream packages, so private included sources preserve the approved behavior.

## Decision

Include the upstream Unix adapter and AT-SPI translation modules privately inside the library so Cargo consumers cannot lose the state repair by ignoring a workspace patch table. Preserve release identifiers, source archive hashes and license notices. Mechanically qualify crate-local imports for the private module location and select the upstream async-io executor independently of the library tokio feature. Keep the semantic delta limited to expanded/expandable and disabled state translation, with dedicated regression checks and real Orca evidence. Do not expose the vendored adapter as a second public API.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/accessibility/platform`, `Cargo.toml`.

Behavior checks: `src/accessibility/platform/translation/state_tests.rs`, `tests/api_widget_behavior/orca.py`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
