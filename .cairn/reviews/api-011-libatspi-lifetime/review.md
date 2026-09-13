# API-011 isolated libatspi lifetime review

Date: 2026-09-13

## Scope examined

I examined the confirmed state-set lifetime failure in the reader-side AT-SPI
client, the isolated repair, and the API-011 Orca workflows that consume it. The
fixture downloads the official at-spi2-core 2.60.6 archive and requires SHA-256
`a89b64a8b217a8042bdf0e35cbfab629ceee35640dba75df578afde9aa789d57` before
extracting or building it. The patch changes only `atspi/atspi-stateset.c`; it is
applied with zero fuzz inside `target/libatspi-lifetime`. The installed desktop
library is never replaced.

## Violating and corrected cases

`python3 -B scripts/check-libatspi-lifetime.py --record-review` built unpatched
and patched AddressSanitizer libraries plus an ordinary patched runtime. A
reentrant `GetState` fixture released the accessible owner's state-set reference
while the synchronous refresh was in progress.

- The unpatched `atspi_state_set_contains` case aborted with an AddressSanitizer
  heap-use-after-free in `refresh_states`.
- The unpatched `atspi_state_set_get_states` case aborted with the same
  heap-use-after-free.
- Both patched AddressSanitizer cases completed, returned the focused state, and
  observed exactly one finalization after the retained reference was released.
- Both ordinary-runtime cases loaded libatspi from the isolated repaired build
  and completed the same reentrant release fixture.

The captured process results and sanitizer diagnostics are in `result.json`.
The patched and unpatched libraries expose the same dynamic symbol set after
excluding AddressSanitizer bookkeeping symbols. The installed
`/usr/lib/x86_64-linux-gnu/libatspi.so.0.0.1` hash was
`af7f57ce45eb98b59e0ae77dd3c22ee497785df2bba0ab7b82d6735d068f6876` before
and after the run.

## Consumer behavior

`scripts/check-api-widget-behavior.py` ran the 401-test API widget suite, the
focused accessibility and transport suites, both bus failure cases, and the
unchanged Orca command matrix with the repaired library directory supplied only
to the reader processes. Orca reported AT-SPI2 2.60.6. Negative semantic-label
controls failed for their intended reason, and both supported viewports passed
the base, table, tabs, overlay, display, and dialog catalogs. The existing
ConPTY receipt and the macOS/Windows native widget receipts also validated after
generated Python bytecode from a separate focused run was removed.

## Findings and limits

The repair closes the demonstrated reentrant lifetime gap for both public
state-set readers without changing the library's exported ABI or the machine's
installed library. The proof targets the confirmed state-set failure path; it
does not claim that unrelated code in libatspi is free of other defects. The
full formal API-011 result still belongs to `cairn check API-011` after these
inputs are committed.
