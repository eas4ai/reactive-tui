# API-019 local scopes and mapping checkpoint

Status: implemented and verified locally; full API-019 remains incomplete.

The developer approved api-004-api-019-api-020. The local ownership decision was
recorded before implementation. `with_local_hooks` owns the non-Send arena;
Hooks stores only thread-safe scope and slot metadata. The arena owns local shares,
TLS holds weak bindings, and resource cleanup sweeps active arenas. Upgraded
resource owners and removed values drop after all arena borrows are released.
The whole Hooks owner binds to one scope, including before allocating another slot.
App.run moves App into the scoped closure, reusing an existing scope when present.
No unsafe Send promise, dependency or Rc field in App/generated state was added.

## Executed checks and controls

`api-019-local-validation/20260913T021058996371Z/` records the reference group,
hook state/lifecycle integration, full library, strict all-target Clippy, formatting,
and executable migration guide all passing. The full library has 1011 passing
tests and two existing ignored fixtures. The reference group requires all ten
reference cases, thirteen local-scope cases, and the external consumer. Exact
discovery and execution checks reject missing tests. The consumer verifies
retention through foreign owner drop until scope exit and requires E0277 when a
LocalRef is sent to another thread. Historical unscoped failures are preserved.

`api-019-local-controls/20260913T020654707415Z/` independently removes retention,
resource-triggered cleanup, and the App scope wrapper. Each targeted test fails;
each exact restored source passes. `api-019-mapping-control/20260913T021218153333Z/`
removes paste conversion: its now-discovered test fails at the expected mapping
assertion. After exact restoration, all five mapping tests are discovered and pass.
The mapping repair only relocates tests out of a nested render function and removes
the obsolete dead-code allowance; original assertions and production logic remain.

Development failures were used to correct owner-wide binding and test fixtures:
a scope mismatch could otherwise append a new slot; a test initially touched real
Crossterm input, then spun before the render interval. The bounded virtual backend
now waits for App wakeups. A generated-props fixture Default warning was corrected
with an explicit missing-prop sentinel. These failures were never acceptance passes.

## Independent review and production audit

The read-only ownership reviewer found no concrete correctness defect. Reviewed
weak bindings, scope identity reuse, thread confinement, nested borrowing, owner
cleanup, destructor reentry, and App success/error/unwind ordering. Added the
suggested inner-scope cleanup of an outer owner. Scope-exit destructors cannot
reenter the closing scope; a cleanup racing with an in-flight sweep may wait for
a later sweep or scope exit. The migration guide states both boundaries.

Rules 1–4: scoped ownership follows the explicit approval and preserves generic
bounds and supported non-Send values. Changes and documentation are cohesive.
Rules 5–8: ownership and failure boundaries are explicit; no new dependency,
unsafe thread promise or user-destructor execution under an arena borrow. Checks
are bounded and builds/tests remain capped at twelve.
Rules 9–12: API-019 remains the sole in-progress todo. Real failure controls and
corrected cases are preserved. No native-scope or whole-catalog readiness claim.
Rules 13–14: local repairs are reviewable; commitment completion is withheld.
Remaining performance ownership needs the separate choice documented in
`api-019-performance-owner/review.md`. Gesture, themes, Markdown/syntax, editor,
legacy terminal/platform concerns and API-020 still require work and current proof.

Final Ripwire edit, quality-delta and test-gate outputs are in `api-019-local-static/`.
Their results and focused findings are reviewed separately; static nonpasses are
not replaced with success claims. Raw captured output remains unchanged.

Static review: all three focused edit checks passed; quality-delta exited 2 and
test-gate exited 4. Focused non-dead-code findings concern unchanged backend
conversion/render similarities, a small Arc/Atomic test-sentinel constructor, and
minor churn in the App wrapper and hook metadata. The relocated backend tests
execute and their mutant fails; these findings do not justify changing unrelated
backend algorithms or extracting a generic constructor. Newly dead reports include
test/trait entry points exercised by the recorded checks. Unrestricted whitespace
checks may flag captured command output; those bytes remain intact. Authored-file
whitespace and both external Rust fixture formatting checks passed.
