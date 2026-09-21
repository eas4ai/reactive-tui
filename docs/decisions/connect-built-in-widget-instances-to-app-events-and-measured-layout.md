# Connect built-in widget instances to App events and measured layout

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: Widget state resets on redraw, callbacks run twice or survive removal, registered user components are replaced, or widget input and layout use guessed bounds.
History: The API reversals concerned clipboard cancellation ownership and measured Windows startup time. They require testing real ownership and geometry here rather than assuming the old widget state works. This additive connection stays Judged because it preserves public signatures and the existing App instance owner; a required contract break will be escalated separately.

## Decision

Keep the existing Component implementations and the App-owned keyed instance tree. Add default event and measured-layout methods to the type-erased component boundary, preserving existing implementors. Give resolved component roots owned event handlers that reach their live instance and resource scope; release handlers on removal. Pass acknowledged paint bounds to controls that need viewport measurements and local mouse coordinates. Preserve event-mutated props until the caller supplies a changed value. Provide typed Element construction for generic widgets and a built-in fallback for existing named widget builders without replacing caller registry entries. Repair each builder to retain its actual configuration and each widget to honor focus, disabled state, callbacks, empty data and actual dimensions. Record per-family App workflows and challenge the shared routing with a safe violating case. This extends the existing keyed ownership and geometry decisions; it does not retire public construction routes.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/component/builtin.rs`, `src/component/runtime.rs`, `src/component/layout_info.rs`, `src/app/event_tree.rs`.

Behavior checks: `tests/api_widget_behavior.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
