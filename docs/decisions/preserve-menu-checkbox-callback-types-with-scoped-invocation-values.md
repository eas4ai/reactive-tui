# Preserve menu checkbox callback types with scoped invocation values

Level: Judged
Decided by: Codex
Rests on: API-011
Would be wrong if: A nested callback observes another item value, a panic leaves a value behind, or Apps share retained checkbox state.
History: Clipboard deadline reversals and the AT-SPI repair reversal show that local assumptions need observation at the real boundary. This remains Judged: public types stay compatible, and scoped argument behavior will be challenged with reentry and unwind tests before App acceptance.

## Decision

MenuAction exposes a public Arc<dyn Fn()> field, while MenuItem::checkbox accepts Fn(bool). Preserve both public shapes. MenuItem::execute supplies the next checkbox value through a private thread-local invocation scope; the constructor adapter consumes it before calling user code. Restore the previous scope on unwind and isolate direct MenuAction calls. This transports only a synchronous argument: retained checked state belongs to each menu instance. Compare action callback identity as well as ID so replacement callbacks reach retained components. Verify toggles, direct calls, reentry, panic cleanup, thread isolation and replacement identity before using this adapter in App menus.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

Implementation: `src/widgets/menu/invocation.rs`, `src/widgets/menu/item.rs`.

Behavior checks: `tests/api_widget_behavior/menus.rs`.

Editing results, failure demonstrations and outstanding platform limits are
recorded in `.cairn/reviews/rust-api-remediation.md` and
`docs/widget-acceptance.md`. These references do not replace committed Cairn
acceptance evidence.
