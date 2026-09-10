# Record App widget evidence while retaining the engine lifecycle requirements

Level: Judged
Decided by: Codex
Rests on: API-011,API-012,API-014,API-016,API-020
Would be wrong if: An App builder control is marked accepted without behavioral evidence, or an unfinished engine, protocol, or entry-point requirement is treated as complete or removed from final acceptance.
History: The commitment orders widget acceptance before dialog-engine lifecycle, image acceptance and entry-point integration. The retained dialog decision requires the engine to use the same controls under API-012. The matrix currently mixes verified App controls with those unfinished dedicated requirements.

## Decision

Reconcile the API-011 matrix against actual public construction routes and App workflows, including dialog editing, callbacks, removal, focus, sizing and styling. Keep all existing App, native-platform, graphics-host and Orca checks. Record verified App behavior without claiming the separate DialogEngine lifecycle works. List API-012 engine stacking, synchronous/asynchronous result delivery and lifecycle as unfinished until its own implementation and mechanism pass; likewise retain API-014 and API-016 obligations. No feature is removed, no failed check becomes a pass, and no missing App control gets a matrix exception. The current commitment remains all 54 requirements, and API-020 must reconcile these overlapping entries with their dedicated evidence before Done.

## Realized by

- b2079c401464b25ccb33747e33b5b3373f284a90 Restore widget behavior, native terminal sessions, and image output through App

The App rows in `docs/widget-acceptance.md` are reconciled with their behavioral
checks, including the image-host and Windows TerminalWidget rows. Their App
coverage is reviewed; committed Cairn acceptance receipts remain required.
API-012 engine results/stacking/lifecycle and API-014/API-016 obligations remain
explicitly unfinished. `.cairn/reviews/rust-api-remediation.md` records the
construction-route review and the limits of these editing results.
