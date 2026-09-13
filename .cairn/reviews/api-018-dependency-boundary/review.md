# Facade example discovery and the pinned input dependency

API-018 receipt 20260913T071233883Z failed after example discovery traversed the
new separate Crossterm crate. It tried compiling a shell command as Rust and
Crossterm examples against only the Reactive-TUI facade. All other documentation,
Props, rustdoc, doctest and non-Rust example checks passed in that receipt.

The collector already excludes the separately maintained SuprTUI crate.
Apply that same exact directory boundary to src/backend/crossterm; do not
exclude other backend code, public guides, or macro examples. Dependency
renderer/input behavior retains its own checks, including native verification.
This does not claim upstream dependency documentation acceptance.

Compared examples.json from the prior pass (20260913T040057123715Z), the failed
run (20260913T071026448749Z), and the corrected examples-only run
(20260913T071425457557Z). All 62 prior public examples retain identical file,
language and content hashes. The corrected inventory has 63 entries, adding
the App-owned performance guide example. The only 39 removed entries belong
to src/backend/crossterm. The entire examples section passed locally.
The existing checker controls also passed, including zero-test and nested
failure rejection. The formal full check must run after commit.

Focused Ripwire edit-check passed. quality-delta returned 2 on broad existing
and reference findings, and test-gate returned 4; neither is a pass.
Self-audit: this is a bounded correction to crate ownership in the verification
collector. Every previously accepted public example remains checked. No
production behavior, advertised API, or failure assertion was removed.
